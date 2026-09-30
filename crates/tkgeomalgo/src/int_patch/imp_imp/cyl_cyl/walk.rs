// Rust translation of CyCyNoGeometric (the setup and the loops over the ranges) of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The numerical intersection of two cylinders: U1 walks over each range where the
//! intersection exists, the two lines `U2 = FI2 +/- f(U1)` are sampled, and each line is
//! broken at the critical points and at the V boundaries.
use super::super::super::IntPatchImpImpStatus;
use super::super::super::wline::{IntPatchWLType, IntPatchWLine};
use super::super::plane::{Lines, Points};
use super::boundaries::WorkWithBoundaries;
use super::critical::{NB_CRIT_POINTS_MAX, critical_points_computing};
use super::inscribe::{inscribe_interval, inscribe_point, insertion_sort};
use super::wline::{SurfBounds, WlContext};
use crate::int_patch::point_line::IntPatchPointLine;
use crate::int_surf::IntSurfLineOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::{is_equal, real_to_int};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::bnd::Range1;
use rustycad_tkmath::gp::{Cylinder3, Line3};
use std::f64::consts::{FRAC_PI_2, PI};

/// The number of walking lines (the two signs of the arccosine).
pub(super) const NB_WLINES: usize = 2;

/// `WLFStatus`: the state of a walking line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WlfStatus {
    /// No point has been added.
    Absent,
    /// The line holds at least one point.
    Exist,
    /// The line ended at a critical point: a new line starts.
    Broken,
}

/// The fixed data of the walking.
pub(super) struct WalkSetup<'a> {
    pub(super) bw: &'a WorkWithBoundaries<'a>,
    pub(super) ctx: WlContext<'a>,
    pub(super) b: SurfBounds,
    pub(super) range_s: (Range1, Range1),
    pub(super) is_good: bool,
    pub(super) nb_max_points: i32,
    pub(super) nb_min_points: i32,
    pub(super) nb_points: i32,
    pub(super) step_min: f64,
    pub(super) step_max: f64,
    pub(super) period: f64,
}

/// The state of the lines of one step of the outer walking loop.
pub(super) struct LinesState {
    pub(super) u2: [f64; NB_WLINES],
    pub(super) v1: [f64; NB_WLINES],
    pub(super) v2: [f64; NB_WLINES],
    pub(super) status: [WlfStatus; NB_WLINES],
    pub(super) v1_prev: [f64; NB_WLINES],
    pub(super) v2_prev: [f64; NB_WLINES],
    pub(super) u_expect: [f64; NB_WLINES],
    pub(super) adding_enabled: [bool; NB_WLINES],
    pub(super) wline: [Handle<IntPatchWLine>; NB_WLINES],
}

impl LinesState {
    fn new(uf: f64) -> Self {
        let new_line = || {
            let wl = IntPatchWLine::new_undecided(&Handle::new(IntSurfLineOn2S::new(None)), false);
            wl.set_creating_way_info(IntPatchWLType::ImpImp);
            Handle::new(wl)
        };
        Self {
            u2: [0.0; NB_WLINES],
            v1: [0.0; NB_WLINES],
            v2: [0.0; NB_WLINES],
            status: [WlfStatus::Absent; NB_WLINES],
            v1_prev: [0.0; NB_WLINES],
            v2_prev: [0.0; NB_WLINES],
            u_expect: [uf; NB_WLINES],
            adding_enabled: [true; NB_WLINES],
            wline: [new_line(), new_line()],
        }
    }

    /// The point set of the line `i`.
    pub(super) fn curve(&self, i: usize) -> Handle<IntSurfLineOn2S> {
        self.wline[i].curve()
    }
}

/// The good intersection (axes almost perpendicular, one radius more than 3 times the
/// other and the small cylinder inside the big one) and its optimal step of U.
fn good_intersection(cyl1: &Cylinder3, cyl2: &Cylinder3) -> Result<Option<f64>, Failure> {
    let to_much_coeff: f64 = 3.0;
    let crit_angle: f64 = PI / 18.0; // 10 degree
    let (r1, r2) = (cyl1.radius(), cyl2.radius());
    let (rmin, rmax) = if r1 > to_much_coeff * r2 {
        (r2, r1)
    } else if r2 > to_much_coeff * r1 {
        (r1, r2)
    } else {
        return Ok(None);
    };
    let (ax1, ax2) = (cyl1.axis(), cyl2.axis());
    if !ax1.is_normal(ax2, crit_angle) {
        return Ok(None);
    }
    let dist: f64 = Line3::from_axis(ax1).distance_line(Line3::from_axis(ax2))?;
    if dist > rmax / 2.0 {
        return Ok(None);
    }
    // The relative deflection; the absolute one is Rmin*aDeflection.
    let deflection: f64 = 0.001;
    let mut nb_p: i32 = 3;
    if rmin * deflection > 1.0e-3 {
        let angle: f64 = 2.0e0 * (1.0e0 - deflection).acos();
        nb_p = (2.0 * PI / angle) as i32 + 1;
    }
    Ok(Some(2.0 * FRAC_PI_2 / f64::from(nb_p - 1)))
}

impl<'a> WalkSetup<'a> {
    /// The setup of `CyCyNoGeometric`: `Err(status)` for an early end (OK when a V
    /// range is void, InfiniteSectionCurve for a curve longer than 1e5 radius, #26894).
    fn new(
        cyl1: &Cylinder3,
        cyl2: &Cylinder3,
        bw: &'a WorkWithBoundaries<'a>,
    ) -> Result<Result<Self, IntPatchImpImpStatus>, Failure> {
        let b: SurfBounds = bw.bounds()?;
        let (range_s1, range_s2) = bw.boundary_estimation(cyl1, cyl2)?;
        if range_s1.is_void() || range_s2.is_void() {
            return Ok(Err(IntPatchImpImpStatus::Ok));
        }
        // "We should return fail status from intersector if the result should be an
        // infinite curve of non-analytical type... the limit for the extent is the
        // radius divided by 1e+2 and multiplied by 1e+7" (#26894, MSV).
        let f: f64 = 1.0e+5;
        if range_s1.delta() > f * cyl1.radius() || range_s2.delta() > f * cyl2.radius() {
            return Ok(Err(IntPatchImpImpStatus::InfiniteSectionCurve));
        }
        let opt_du: Option<f64> = good_intersection(cyl1, cyl2)?;
        Ok(Ok(Self::with_steps(
            cyl1,
            bw,
            b,
            (range_s1, range_s2),
            opt_du,
        )?))
    }

    fn with_steps(
        cyl1: &Cylinder3,
        bw: &'a WorkWithBoundaries<'a>,
        b: SurfBounds,
        range_s: (Range1, Range1),
        opt_du: Option<f64>,
    ) -> Result<Self, Failure> {
        let period: f64 = 2.0 * PI;
        let (nb_max_points, nb_min_points, du) = match opt_du {
            Some(du) => (200, 50, du),
            None => (1000, 200, 2.0 * PI / 1000.0),
        };
        let nb_pts: i32 = std::cmp::min(
            real_to_int((b.u1l - b.u1f) / du) + 1,
            real_to_int(20.0 * cyl1.radius()),
        );
        let nb_points: i32 = std::cmp::min(std::cmp::max(nb_min_points, nb_pts), nb_max_points);
        let step_min: f64 = std_max(bw.tol_2d, precision::p_confusion());
        let step_max: f64 = if b.u1l - b.u1f > PI / 100.0 {
            (b.u1l - b.u1f) / f64::from(nb_points)
        } else {
            b.u1l - b.u1f
        };
        Ok(Self {
            bw,
            ctx: bw.context()?,
            b,
            range_s,
            is_good: opt_du.is_some(),
            nb_max_points,
            nb_min_points,
            nb_points,
            step_min,
            step_max,
            period,
        })
    }
}

/// `CyCyNoGeometric(theCyl1, theCyl2, theBW, theRange, theNbOfRanges, isTheEmpty,
/// theSlin, theSPnt)`: the walking lines of two cylinders over the U1 ranges, then the
/// isolated points not on a line, and new short lines near the remaining ones.
pub(in super::super) fn cycy_no_geometric(
    cyl1: &Cylinder3,
    cyl2: &Cylinder3,
    bw: &WorkWithBoundaries<'_>,
    range: &mut [Range1; 2],
    empty: &mut bool,
    (slin, spnt): (&mut Lines, &mut Points),
) -> Result<IntPatchImpImpStatus, Failure> {
    let setup = match WalkSetup::new(cyl1, cyl2, bw)? {
        Ok(setup) => setup,
        Err(status) => return Ok(status),
    };
    let b = setup.b;
    for r in range.iter_mut() {
        if !r.is_void() {
            inscribe_interval(b.u1f, b.u1l, r, bw.tol_2d, setup.period);
        }
    }
    let other: Range1 = range[1];
    if range[0].union(other) {
        // Works only for two ranges.
        range[1].set_void();
    }
    // The critical points are the U1 values where the lines must be broken.
    let mut u1crit: [f64; NB_CRIT_POINTS_MAX] = [precision::infinite(); NB_CRIT_POINTS_MAX];
    let bounds = (b.u1f, b.u1l, b.u2f, b.u2l);
    let nb_crit: usize =
        critical_points_computing(bw.coeffs, bounds, setup.period, bw.tol_2d, &mut u1crit);
    for r in range.iter() {
        let mut walk = IntervalWalk::new(&setup, (&mut u1crit, nb_crit), empty, (slin, spnt));
        walk.run(*r)?;
    }
    setup.remove_points_on_lines(slin, spnt)?;
    setup.points_neighborhood(slin, spnt)?;
    Ok(IntPatchImpImpStatus::Ok)
}

/// The walking over one continuous range of U1.
pub(super) struct IntervalWalk<'s, 'o> {
    pub(super) setup: &'s WalkSetup<'s>,
    pub(super) u1crit: &'o mut [f64; NB_CRIT_POINTS_MAX],
    pub(super) nb_crit: usize,
    pub(super) empty: &'o mut bool,
    pub(super) slin: &'o mut Lines,
    pub(super) spnt: &'o mut Points,
    pub(super) is_added_into_wl: [bool; NB_WLINES],
    pub(super) uf: f64,
    pub(super) ul: f64,
    pub(super) is_delta_period: bool,
}

impl<'s, 'o> IntervalWalk<'s, 'o> {
    fn new(
        setup: &'s WalkSetup<'s>,
        (u1crit, nb_crit): (&'o mut [f64; NB_CRIT_POINTS_MAX], usize),
        empty: &'o mut bool,
        (slin, spnt): (&'o mut Lines, &'o mut Points),
    ) -> Self {
        Self {
            setup,
            u1crit,
            nb_crit,
            empty,
            slin,
            spnt,
            is_added_into_wl: [false; NB_WLINES],
            uf: 1.0,
            ul: 0.0,
            is_delta_period: false,
        }
    }

    /// The range: its critical points inscribed and sorted, then the lines one after
    /// the other while U1 has not reached the end.
    fn run(&mut self, range: Range1) -> Result<(), Failure> {
        let (mut uf, mut ul) = (1.0, 0.0);
        if !range.get_bounds(&mut uf, &mut ul) {
            return Ok(());
        }
        (self.uf, self.ul) = (uf, ul);
        self.is_delta_period = is_equal(ul - uf, self.setup.period);
        let period: f64 = self.setup.period;
        for u in self.u1crit[..self.nb_crit].iter_mut() {
            inscribe_point(uf, ul, u, 0.0, period, false);
        }
        insertion_sort(&mut self.u1crit[..self.nb_crit], |a, b| a < b);
        while self.uf < self.ul {
            let mut lines = LinesState::new(self.uf);
            self.walk_lines(&mut lines)?;
            self.output_lines(&lines)?;
        }
        Ok(())
    }

    /// `aCriticalDelta`: the side of each critical point at the start of the lines.
    pub(super) fn critical_delta(&self) -> [f64; NB_CRIT_POINTS_MAX] {
        let mut delta = [0.0; NB_CRIT_POINTS_MAX];
        for (i, d) in delta.iter_mut().enumerate().take(self.nb_crit) {
            *d = self.uf - self.u1crit[i];
        }
        delta
    }
}
