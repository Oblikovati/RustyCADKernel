// Rust translation of IntCurve_IntConicConic_1.cxx (the parts shared by the line-circle
// and line-ellipse Perform overloads: ProjectOnLAndIntersectWithLDomain and the
// solution loop) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The line-circle and line-ellipse intersections share their structure: arcs of
//! the closed conic found geometrically, clipped by both domains, then classified.
//! [`ClosedConic`] is the conic side of that structure.
use super::conic_conic_tool::{PI_P_PI, determine_transition_lc, normalize_on_circle_domain};
use super::{IntConicConic, Interval, PeriodicInterval};
use crate::int_imp_par_gen::determine_position;
use crate::int_res2d::{Domain, IntersectionPoint, IntersectionSegment, Position, Transition};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2d, Line2, Point2, Vector2};
use std::cmp::Ordering;
use std::f64::consts::PI;

/// The closed conic of a line-conic intersection: its value, D1, D2, and the
/// radius that converts its parameter to a length (the minor radius of an ellipse).
pub(super) trait ClosedConic {
    /// The point of parameter `u`.
    fn point_at(&self, u: f64) -> Point2;
    /// `D1` (`ElCLib::CircleD1` or `EllipseD1`).
    fn d1_at(&self, u: f64) -> (Point2, Vector2);
    /// `D2`.
    fn d2_at(&self, u: f64) -> (Point2, Vector2, Vector2);
    /// `R`: the radius, the minor radius of an ellipse.
    fn length_radius(&self) -> f64;
    /// Whether the native loop of this conic has the slips of the line-ellipse
    /// overload (see [`LineConicContext`]).
    const NATIVE_ELLIPSE_SLIPS: bool;
}

/// `ProjectOnLAndIntersectWithLDomain(Conic, Line, CDomainAndRes, LDomain,
/// ConicSolution, LineSolution, NbSolTotal, RefLineDomain, ConicDomain)`: the arc of
/// the conic projected on the line, intersected with the line domain and clamped to
/// its bounds (without tolerance).
pub(super) fn project_on_l_and_intersect_with_l_domain(
    conic: &impl ClosedConic,
    line: &Line2,
    c_domain_and_res: &mut PeriodicInterval,
    l_domain: &mut Interval,
    solutions: &mut LineConicSolutions,
    ref_line_domain: &Domain,
) -> Result<(), Failure> {
    if c_domain_and_res.is_null() {
        return Ok(());
    }
    let mut linf: f64 =
        elclib::line_parameter_2d(line.position(), conic.point_at(c_domain_and_res.binf));
    let mut lsup: f64 =
        elclib::line_parameter_2d(line.position(), conic.point_at(c_domain_and_res.bsup));
    let l_inter: Interval = Interval::new(linf, lsup);
    let mut l_inter_and_domain: Interval = l_domain.intersection_with_bounded(&l_inter);
    if l_inter_and_domain.is_null {
        return Ok(());
    }
    let dom_linf: f64 = if ref_line_domain.has_first_point() {
        ref_line_domain.first_parameter()?
    } else {
        -precision::infinite()
    };
    let dom_lsup: f64 = if ref_line_domain.has_last_point() {
        ref_line_domain.last_parameter()?
    } else {
        precision::infinite()
    };
    (linf, lsup) = (l_inter_and_domain.binf, l_inter_and_domain.bsup);
    linf = linf.max_native(dom_linf).min_native(dom_lsup);
    lsup = lsup.max_native(dom_linf).min_native(dom_lsup);
    (l_inter_and_domain.binf, l_inter_and_domain.bsup) = (linf, lsup);
    let mut conic_solution = PeriodicInterval::new(c_domain_and_res.binf, c_domain_and_res.bsup);
    if conic_solution.length() > PI {
        conic_solution.complement();
    }
    solutions.on_conic.push(conic_solution);
    solutions.on_line.push(l_inter_and_domain);
    Ok(())
}

// The clamps of ProjectOnLAndIntersectWithLDomain: `if (x < lo) x = lo` then
// `if (x > hi) x = hi`, applied in the native order.
trait NativeClamp {
    fn max_native(self, lo: f64) -> f64;
    fn min_native(self, hi: f64) -> f64;
}

impl NativeClamp for f64 {
    fn max_native(self, lo: f64) -> f64 {
        if self < lo { lo } else { self }
    }
    fn min_native(self, hi: f64) -> f64 {
        if self > hi { hi } else { self }
    }
}

/// The solution arcs of a line-conic intersection (the native arrays of 4, which
/// the port lets grow instead of overflowing).
#[derive(Default)]
pub(super) struct LineConicSolutions {
    pub(super) on_conic: Vec<PeriodicInterval>,
    pub(super) on_line: Vec<Interval>,
}

/// The domain of a closed conic moved to `[0, 2PI)` and widened by its tolerances
/// divided by `radius`, kept below one period (`Ajout : Jeudi 28 mars 96`).
pub(super) fn widened_periodic_domain(
    domain: &Domain,
    radius: f64,
) -> Result<PeriodicInterval, Failure> {
    let mut c_domain: PeriodicInterval = PeriodicInterval::from_domain(domain)?;
    shift_to_first_period(&mut c_domain);
    let mut binf_modif: f64 = c_domain.binf;
    let mut bsup_modif: f64 = c_domain.bsup;
    binf_modif -= domain.first_tolerance()? / radius;
    bsup_modif += domain.last_tolerance()? / radius;
    let deltat: f64 = bsup_modif - binf_modif;
    if deltat <= PI_P_PI {
        (c_domain.binf, c_domain.bsup) = (binf_modif, bsup_modif);
    } else {
        let mut t: f64 = PI_P_PI - deltat;
        t *= 0.5;
        (c_domain.binf, c_domain.bsup) = (binf_modif + t, bsup_modif - t);
    }
    shift_to_first_period(&mut c_domain);
    Ok(c_domain)
}

// Moves `Binf` into `[0, 2PI)` and keeps the length.
fn shift_to_first_period(interval: &mut PeriodicInterval) {
    let deltat: f64 = interval.bsup - interval.binf;
    while interval.binf >= PI_P_PI {
        interval.binf -= PI_P_PI;
    }
    while interval.binf < 0.0 {
        interval.binf += PI_P_PI;
    }
    interval.bsup = interval.binf + deltat;
}

/// `Modified by Sergey KHROMOV - Mon Dec 18 2000`: two geometric arcs that are the
/// two halves of one arc across 0 are reduced to the half that meets the domain.
pub(super) fn drop_split_arc(
    nbsol: &mut i32,
    int1: &mut PeriodicInterval,
    int2: &PeriodicInterval,
    domain: &Domain,
) -> Result<(), Failure> {
    if !(*nbsol == 2 && int2.bsup == int1.binf + PI_P_PI) {
        return Ok(());
    }
    let (first_bound, last_bound) = (domain.first_parameter()?, domain.last_parameter()?);
    let (first_tol, last_tol) = (domain.first_tolerance()?, domain.last_tolerance()?);
    if int1.binf == 0.0 && first_bound - first_tol > int1.bsup {
        *nbsol = 1;
        int1.set_values(int2.binf, int2.bsup);
    } else if int2.bsup == PI_P_PI && last_bound + last_tol < int2.binf {
        *nbsol = 1;
    }
    Ok(())
}

/// The arcs of the conic within its (widened) domain and the line domain.
pub(super) fn line_conic_solutions(
    conic: &impl ClosedConic,
    line: &Line2,
    (mut int1, mut int2, nbsol): (PeriodicInterval, PeriodicInterval, i32),
    (line_domain, conic_domain): (&Domain, &Domain),
) -> Result<LineConicSolutions, Failure> {
    let mut c_domain: PeriodicInterval =
        widened_periodic_domain(conic_domain, conic.length_radius())?;
    let mut l_domain: Interval = Interval::from_domain(line_domain)?;
    let mut solutions = LineConicSolutions::default();
    let mut part = c_domain.first_intersection(&mut int1);
    project_on_l_and_intersect_with_l_domain(
        conic,
        line,
        &mut part,
        &mut l_domain,
        &mut solutions,
        line_domain,
    )?;
    let mut part = c_domain.second_intersection(&mut int1);
    project_on_l_and_intersect_with_l_domain(
        conic,
        line,
        &mut part,
        &mut l_domain,
        &mut solutions,
        line_domain,
    )?;
    if nbsol == 2 {
        let mut part = c_domain.first_intersection(&mut int2);
        project_on_l_and_intersect_with_l_domain(
            conic,
            line,
            &mut part,
            &mut l_domain,
            &mut solutions,
            line_domain,
        )?;
        let mut part = c_domain.second_intersection(&mut int2);
        project_on_l_and_intersect_with_l_domain(
            conic,
            line,
            &mut part,
            &mut l_domain,
            &mut solutions,
            line_domain,
        )?;
    }
    Ok(solutions)
}

/// `MaxTol`: the largest of TolConf, Tol and 1e-10.
pub(super) fn max_tolerance(tol_conf: f64, tol: f64) -> f64 {
    let mut max_tol: f64 = tol_conf;
    if max_tol < tol {
        max_tol = tol;
    }
    if max_tol < 1.0e-10 {
        max_tol = 1.0e-10;
    }
    max_tol
}

/// Arcs shorter than `max_tol` on both curves become their mid points.
pub(super) fn collapse_line_conic_arcs(
    solutions: &mut LineConicSolutions,
    radius: f64,
    max_tol: f64,
) {
    for (on_conic, on_line) in solutions
        .on_conic
        .iter_mut()
        .zip(solutions.on_line.iter_mut())
    {
        if (radius * on_conic.length()) < max_tol && on_line.length() < max_tol {
            let t: f64 = (on_conic.binf + on_conic.bsup) * 0.5;
            (on_conic.binf, on_conic.bsup) = (t, t);
            let t: f64 = (on_line.binf + on_line.bsup) * 0.5;
            (on_line.binf, on_line.bsup) = (t, t);
        }
    }
}

/// The context of the classification loop of a line-conic intersection.
pub(super) struct LineConicContext<'a, C: ClosedConic> {
    pub(super) conic: &'a C,
    pub(super) line: &'a Line2,
    pub(super) line_domain: &'a Domain,
    pub(super) conic_domain: &'a Domain,
    pub(super) is_opposite: bool,
    pub(super) max_tol: f64,
    pub(super) tol: f64,
    /// The native line-ellipse loop evaluates the conic at the first bound of the
    /// arc for its second point and reads the LINE domain's last parameter when
    /// that point is at the end of the ellipse domain; kept as flags.
    pub(super) ellipse_quirks: bool,
}

/// One classified bound: the point on the conic, parameters and transitions.
struct ClassifiedBound {
    pnt: Point2,
    l: f64,
    c: f64,
    pos1: Position,
    pos2: Position,
    t1: Transition,
    t2: Transition,
}

impl<C: ClosedConic> LineConicContext<'_, C> {
    // `D2` of the conic at `u`, D1 of the line at `l`, positions and transitions.
    fn classify(&self, (u_eval, u_pos): (f64, f64), l: f64) -> Result<ClassifiedBound, Failure> {
        let (pnt, mut tan1, norm1) = self.conic.d2_at(u_eval);
        let line_axis: Axis2d = self.line.position();
        let (p2, mut tan2) = elclib::line_d1_2d(l, line_axis);
        let pos1: Position = determine_position(self.conic_domain, pnt, u_pos)?;
        let pos2: Position = determine_position(self.line_domain, p2, l)?;
        let (mut t1, mut t2) = (Transition::default(), Transition::default());
        let norm2: Vector2 = Vector2::new(0.0, 0.0);
        determine_transition_lc(
            (pos1, &mut tan1, norm1, &mut t1),
            (pos2, &mut tan2, norm2, &mut t2),
            self.tol,
        )?;
        Ok(ClassifiedBound {
            pnt,
            l,
            c: u_pos,
            pos1,
            pos2,
            t1,
            t2,
        })
    }

    // A bound at an end of the conic domain is moved exactly onto that end (point,
    // parameters, positions and transitions recomputed); `c_end` is the parameter
    // the native code takes for the end.
    fn snap_to_end(&self, bound: ClassifiedBound, c_end: f64) -> Result<ClassifiedBound, Failure> {
        let end_point: Point2 = if bound.pos1 == Position::End {
            self.conic_domain.last_point()?
        } else {
            self.conic_domain.first_point()?
        };
        let l: f64 = elclib::line_parameter_2d(self.line.position(), end_point);
        self.classify((c_end, c_end), l)
    }

    // The conic parameter of a bound: snapped to the domain end it lies at,
    // otherwise normalized on the conic domain.
    fn settle(
        &self,
        bound: ClassifiedBound,
        raw: f64,
        is_last: bool,
    ) -> Result<ClassifiedBound, Failure> {
        match bound.pos1 {
            Position::End => {
                let c_end: f64 = if is_last && self.ellipse_quirks {
                    self.line_domain.last_parameter()?
                } else {
                    self.conic_domain.last_parameter()?
                };
                self.snap_to_end(bound, c_end)
            }
            Position::Head => {
                let c_head: f64 = self.conic_domain.first_parameter()?;
                self.snap_to_end(bound, c_head)
            }
            Position::Middle => {
                let c: f64 = normalize_on_circle_domain(raw, self.conic_domain)?;
                Ok(ClassifiedBound { c, ..bound })
            }
        }
    }
}

// The solution arc moved by periods to share a part with the conic domain, then
// clipped to it (`7 aout 97`).
fn recenter(on_conic: &mut PeriodicInterval, conic_domain: &Domain) -> Result<(), Failure> {
    let (mut p1, mut p2) = (on_conic.binf, on_conic.bsup);
    let (q1, q2) = (
        conic_domain.first_parameter()?,
        conic_domain.last_parameter()?,
    );
    if p1 > q2 {
        loop {
            p1 -= PI_P_PI;
            p2 -= PI_P_PI;
            if p1 <= q2 {
                break;
            }
        }
    } else if p2 < q1 {
        loop {
            p1 += PI_P_PI;
            p2 += PI_P_PI;
            if p2 >= q1 {
                break;
            }
        }
    }
    if p1 < q1 && p2 > q1 {
        p1 = q1;
    }
    if p1 < q2 && p2 > q2 {
        p2 = q2;
    }
    (on_conic.binf, on_conic.bsup) = (p1, p2);
    Ok(())
}

impl IntConicConic {
    /// The classification loop of a line-conic intersection: each arc becomes a
    /// segment, two end points or one point (the line is the first curve).
    pub(super) fn add_line_conic_solutions<C: ClosedConic>(
        &mut self,
        ctx: &LineConicContext<'_, C>,
        solutions: &mut LineConicSolutions,
    ) -> Result<(), Failure> {
        for i in 0..solutions.on_conic.len() {
            recenter(&mut solutions.on_conic[i], ctx.conic_domain)?;
            let on_line: Interval = solutions.on_line[i];
            let (mut linf, mut lsup) = if ctx.is_opposite {
                (on_line.bsup, on_line.binf)
            } else {
                (on_line.binf, on_line.bsup)
            };
            // Parameters on the conic come first: they are returned in increasing order.
            if linf > lsup {
                let arc: &mut PeriodicInterval = &mut solutions.on_conic[i];
                (arc.binf, arc.bsup) = (arc.bsup, arc.binf);
                (linf, lsup) = (lsup, linf);
            }
            self.add_line_conic_arc(ctx, solutions, i, (linf, lsup))?;
        }
        Ok(())
    }

    fn add_line_conic_arc<C: ClosedConic>(
        &mut self,
        ctx: &LineConicContext<'_, C>,
        solutions: &mut LineConicSolutions,
        i: usize,
        (linf, lsup): (f64, f64),
    ) -> Result<(), Failure> {
        let arc: PeriodicInterval = solutions.on_conic[i];
        let a: ClassifiedBound = ctx.classify((arc.binf, arc.binf), linf)?;
        let a: ClassifiedBound = ctx.settle(a, arc.binf, false)?;
        let reversed: bool = self.base.reversed_parameters();
        let new_point1 = IntersectionPoint::new(a.pnt, a.l, a.c, a.t2, a.t1, reversed);
        let mut line_arc: Interval = solutions.on_line[i];
        let mut conic_arc: PeriodicInterval = arc;
        let total: f64 = line_arc.length() + conic_arc.length();
        if !matches!(total.partial_cmp(&0.0), Some(Ordering::Greater)) {
            return self.base.insert(new_point1);
        }
        let first_eval: f64 = if ctx.ellipse_quirks {
            arc.binf
        } else {
            arc.bsup
        };
        let b: ClassifiedBound = ctx.classify((first_eval, arc.bsup), lsup)?;
        let b: ClassifiedBound = ctx.settle(b, arc.bsup, true)?;
        let new_point2 = IntersectionPoint::new(b.pnt, b.l, b.c, b.t2, b.t1, reversed);
        let long_enough: bool = (b.c - a.c).abs() * ctx.conic.length_radius() > ctx.max_tol
            && (b.l - a.l).abs() > ctx.max_tol;
        if long_enough || a.t1.transition_type() != a.t2.transition_type() {
            // Verifier egalement les transitions
            self.base.append_segment(IntersectionSegment::bounded(
                new_point1,
                new_point2,
                ctx.is_opposite,
                reversed,
            ));
            return Ok(());
        }
        if a.pos1 != Position::Middle || a.pos2 != Position::Middle {
            self.base.insert(new_point1)?;
        }
        if b.pos1 != Position::Middle || b.pos2 != Position::Middle {
            self.base.insert(new_point2)?;
        }
        Ok(())
    }
}
