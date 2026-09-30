// Rust translation of the circle-circle constructor of Geom2dGcc_Circ2d2TanOnGeo.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to two circles with their centre on a curve.
use super::{Circ2d2TanOnGeo, NB_SOL_MAX, circle_touch, on_curve_of, perform_bisector};
use crate::gcc_ana::Circ2dBisec;
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::{Slots, qualify};
use crate::gcc_ent::{Position, QualifiedCirc};
use crate::gcc_int::IType;
use crate::geom2d_int::TheIntConicCurveOfGInter;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Point2};

const MEMBER: &str = "Geom2dGcc_Circ2d2TanOnGeo";

// The candidate radii of one centre (`Rbid`, `RBid` and `Radius`, arrays of 2).
struct Radii {
    rbid: Slots<f64>,
    r_bid: Slots<f64>,
    radius: Slots<f64>,
}

impl Circ2d2TanOnGeo {
    /// `Geom2dGcc_Circ2d2TanOnGeo(GccEnt_QualifiedCirc Qualified1, GccEnt_QualifiedCirc
    /// Qualified2, OnCurv, Tolerance)`: the circle bisectors intersected with the
    /// curve, and at each centre the radii that fit both circles within the
    /// tolerance. The native slips are kept: an unqualified first circle writes its
    /// first radius twice (the second slot is never written: 0 here), the
    /// unqualified qualifiers are measured with the radius at the bisector index
    /// (Standard_OutOfRange past the second bisector), and a third fitting radius
    /// overruns its array (Standard_OutOfRange);
    /// `Circ2d2TanOnGeo::circ_circ(qc1, qc2, &on, 1e-9)?`.
    pub fn circ_circ(
        qualified1: QualifiedCirc,
        qualified2: QualifiedCirc,
        on_curv: &Geom2dAdaptorCurve,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty();
        let tol: f64 = tolerance.abs();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let (c1, c2): (Circle2, Circle2) = (qualified1.qualified(), qualified2.qualified());
        let bis = Circ2dBisec::new(c1, c2);
        if !bis.is_done() {
            return Ok(circ);
        }
        let mut intp = TheIntConicCurveOfGInter::new();
        let nbsolution: i32 = bis.nb_solutions()?;
        let (cu2, d2) = on_curve_of(on_curv, tol)?;
        let mut radii = Radii {
            rbid: Slots::new(2),
            r_bid: Slots::new(2),
            radius: Slots::new(2),
        };
        let kinds = [IType::Cir, IType::Ell, IType::Hpr, IType::Lin];
        for i in 1..=nbsolution {
            let sol = bis.this_solution(i)?;
            perform_bisector(&mut intp, &sol, &kinds, (&cu2, &d2), (tol, tol))?;
            if intp.is_done() && !intp.is_empty()? {
                for j in 1..=intp.nb_points()? {
                    let point = intp.point(j)?;
                    circ.centre_radii(
                        [qualified1, qualified2],
                        point.value(),
                        point.param_on_second(),
                        i,
                        &mut radii,
                        tol,
                    )?;
                }
            }
        }
        Ok(circ)
    }

    // The solutions at one centre: the radii fitting both circles, then one
    // solution per fitting radius.
    fn centre_radii(
        &mut self,
        [q1, q2]: [QualifiedCirc; 2],
        center: Point2,
        parcen3: f64,
        i: i32,
        radii: &mut Radii,
        tol: f64,
    ) -> Result<(), Failure> {
        let (c1, c2): (Circle2, Circle2) = (q1.qualified(), q2.qualified());
        let dist1: f64 = center.distance(c1.location());
        let dist2: f64 = center.distance(c2.location());
        let nbsol: i32 = first_radii(q1.qualifier(), dist1, c1.radius(), tol, &mut radii.rbid)?;
        if nbsol != 0 {
            second_radii(q2.qualifier(), dist2, c2.radius(), tol, &mut radii.r_bid)?;
        }
        let mut nnsol: i32 = 0;
        for isol in 1..=nbsol {
            for jsol in 1..=nbsol {
                let (rb1, rb2) = (radii.rbid.get(isol)?, radii.r_bid.get(jsol)?);
                if (rb1 - rb2).abs() <= tol {
                    nnsol += 1;
                    radii.radius.set(nnsol, (rb2 + rb1) / 2.0)?;
                }
            }
        }
        if nnsol == 0 {
            return Ok(());
        }
        let mut k: i32 = 1;
        while k <= nnsol && self.solutions.nbr_sol < NB_SOL_MAX {
            let radius: f64 = radii.radius.get(k)?;
            self.circles_solution([q1, q2], (center, parcen3), radius, (&radii.radius, i), tol)?;
            k += 1;
        }
        self.solutions.well_done = true;
        Ok(())
    }

    // One solution of radius `radius` at `center`; an unqualified circle is
    // qualified with `Radius(i)`, the radius at the bisector index, as natively.
    fn circles_solution(
        &mut self,
        [q1, q2]: [QualifiedCirc; 2],
        (center, parcen3): (Point2, f64),
        radius: f64,
        (radius_slots, i): (&Slots<f64>, i32),
        tol: f64,
    ) -> Result<(), Failure> {
        let (index, circle) = self.open(center, radius, parcen3)?;
        for (arg, q) in [q1, q2].into_iter().enumerate() {
            let qualifier: Position = if q.is_unqualified() {
                let c: Circle2 = q.qualified();
                let distcc: f64 = center.distance(c.location());
                qualify(
                    Position::Unqualified,
                    distcc,
                    radius_slots.get(i)?,
                    c.radius(),
                    tol,
                    false,
                )
            } else {
                q.qualifier()
            };
            self.solutions.qualifier[arg].set(index, qualifier)?;
        }
        for (arg, q) in [q1, q2].into_iter().enumerate() {
            let c: Circle2 = q.qualified();
            let dist: f64 = center.distance(c.location());
            if dist <= tol && (radius - c.radius()).abs() <= tol {
                self.solutions.tangency[arg].same.set(index, 1)?;
                continue;
            }
            let point: Point2 = circle_touch(c.location(), center, radius)?;
            let par_arg: f64 = elclib::parameter(&c, point)?;
            self.touch(arg, index, &circle, point, par_arg)?;
        }
        Ok(())
    }
}

// The radii fitting the first circle (`Rbid`), their count `nbsol`: an unqualified
// circle writes its first slot twice.
fn first_radii(
    q: Position,
    dist1: f64,
    r1: f64,
    tol: f64,
    rbid: &mut Slots<f64>,
) -> Result<i32, Failure> {
    match q {
        Position::Enclosed if dist1 - r1 < tol => rbid.set(1, (r1 - dist1).abs()).map(|()| 1),
        Position::Outside if r1 - dist1 < tol => rbid.set(1, (dist1 - r1).abs()).map(|()| 1),
        Position::Enclosing => rbid.set(1, dist1 + r1).map(|()| 1),
        Position::Unqualified => {
            rbid.set(1, dist1 + r1)?;
            rbid.set(1, (dist1 - r1).abs())?;
            Ok(2)
        }
        _ => Ok(0),
    }
}

// The radii fitting the second circle (`RBid`).
fn second_radii(
    q: Position,
    dist2: f64,
    r2: f64,
    tol: f64,
    r_bid: &mut Slots<f64>,
) -> Result<(), Failure> {
    match q {
        Position::Enclosed if dist2 - r2 < tol => r_bid.set(1, (r2 - dist2).abs()),
        Position::Outside if r2 - dist2 < tol => r_bid.set(1, (r2 - dist2).abs()),
        Position::Enclosing => r_bid.set(1, dist2 + r2),
        Position::Unqualified => {
            r_bid.set(1, dist2 + r2)?;
            r_bid.set(2, (r2 - dist2).abs())
        }
        _ => Ok(()),
    }
}
