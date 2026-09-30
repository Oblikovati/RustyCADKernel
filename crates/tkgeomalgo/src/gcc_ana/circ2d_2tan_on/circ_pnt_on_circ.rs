// Rust translation of GccAna_Circ2d2TanOn_8.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to a circle and through a point with their centre on a circle:
//! a boundary case on the line of the centres (skipped for concentric circles),
//! else the intersections of the locus with the circle-point bisectors, at most 4.
//! The native `SameCenter` flag is never set, so its branches reduce to the
//! regular ones.
use super::{Circ2d2TanOn, NAME};
use crate::gcc_ana::solutions::{Slots, circle_at, qualify};
use crate::gcc_ana::{CircPnt2dBisec, check_qualifier};
use crate::gcc_ent::{Position, QualifiedCirc};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Point2};

struct Args {
    q1: QualifiedCirc,
    point2: Point2,
    on_circ: Circle2,
    tolerance: f64,
}

// The defects of a centre at `pinterm`: both must be within Tol.
fn fits(a: &Args, pinterm: Point2, tol: f64) -> (bool, f64) {
    let c1: Circle2 = a.q1.qualified();
    let distcc2: f64 = pinterm.distance(a.point2);
    let distcc1: f64 = pinterm.distance(c1.location());
    let d1: f64 = (distcc2 - (distcc1 - c1.radius()).abs()).abs();
    let d2: f64 = (distcc2 - (distcc1 + c1.radius())).abs();
    (!(d1 > tol || d2 > tol), distcc1)
}

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Point2, OnCirc, Tolerance): circles tangent
    /// to a qualified circle and through a point, centred on `on_circ`;
    /// `Circ2d2TanOn::circ_pnt_on_circ(q1, point, circle, tol)?`.
    pub fn circ_pnt_on_circ(
        qualified1: QualifiedCirc,
        point2: Point2,
        on_circ: Circle2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        check_qualifier(qualified1.qualifier(), NAME)?;
        let a = Args {
            q1: qualified1,
            point2,
            on_circ,
            tolerance,
        };
        let mut radius: Slots<f64> = Slots::new(2);
        let c1: Circle2 = qualified1.qualified();
        if !on_circ
            .location()
            .is_equal(c1.location(), precision::confusion())
            && circ.boundary_8(&a, &mut radius)?
        {
            return Ok(circ);
        }
        let bis = CircPnt2dBisec::new(c1, point2);
        if !bis.is_done() {
            return Ok(circ);
        }
        for i in 1..=bis.nb_solutions() {
            let sol: Handle<dyn Bisec> = bis.this_solution(i)?;
            let intp = match sol.arc_type() {
                IType::Lin => AnaIntersection::line_circle(sol.line()?, on_circ)?,
                IType::Cir => AnaIntersection::circles(on_circ, sol.circle()?)?,
                IType::Hpr => AnaIntersection::circle_conic(
                    on_circ,
                    &Conic::from_hyperbola(sol.hyperbola()?),
                )?,
                IType::Ell => {
                    AnaIntersection::circle_conic(on_circ, &Conic::from_ellipse(sol.ellipse()?))?
                }
                _ => AnaIntersection::new(),
            };
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? && !intp.parallel_elements()? && !intp.identical_elements()? {
                for j in 1..=intp.nb_points()? {
                    circ.candidate_8(&a, intp.point(j)?.value(), &mut radius)?;
                }
            }
            circ.well_done = true;
        }
        Ok(circ)
    }

    fn boundary_8(&mut self, a: &Args, radius: &mut Slots<f64>) -> Result<bool, Failure> {
        let (c1, tol) = (a.q1.qualified(), a.tolerance.abs());
        let (center1, r1) = (c1.location(), c1.radius());
        let ron: f64 = a.on_circ.radius();
        let distcco: f64 = a.on_circ.location().distance(center1);
        let dircc = Direction2::from_xy(a.on_circ.location().xy() - center1.xy())?;
        let mut pinterm = Point2::from(center1.xy() + (distcco - ron) * dircc.xy());
        let (mut ok, mut distcc1) = fits(a, pinterm, tol);
        if !ok {
            pinterm = Point2::from(center1.xy() + (distcco + ron) * dircc.xy());
            (ok, distcc1) = fits(a, pinterm, tol);
            if !ok {
                return Ok(false);
            }
        }
        let count: i32 = if a.q1.is_enclosed() || a.q1.is_outside() {
            radius.set(1, (distcc1 - r1).abs())?;
            1
        } else if a.q1.is_enclosing() {
            radius.set(1, r1 + distcc1)?;
            1
        } else {
            radius.set(1, (distcc1 - r1).abs())?;
            radius.set(2, r1 + distcc1)?;
            2
        };
        for i in 1..=count {
            self.well_done = true;
            self.boundary_solution_8(a, pinterm, radius.get(i)?, distcc1)?;
        }
        Ok(self.well_done)
    }

    fn boundary_solution_8(
        &mut self,
        a: &Args,
        pinterm: Point2,
        r: f64,
        distcc1: f64,
    ) -> Result<(), Failure> {
        let (c1, tol) = (a.q1.qualified(), a.tolerance.abs());
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(pinterm, r)?;
        self.cirsol.set(index, solution)?;
        let dc1 = Direction2::from_xy(c1.location().xy() - pinterm.xy())?;
        self.set_qualifiers(
            index,
            qualify(a.q1.qualifier(), distcc1, r, c1.radius(), tol, false),
            Position::NoQualifier,
        )?;
        let p1 = Point2::from(pinterm.xy() + r * dc1.xy());
        let centre: Point2 = solution.location();
        self.tangency1.record(
            index,
            p1,
            elclib::parameter(&solution, p1)?,
            elclib::parameter(&c1, p1)?,
        )?;
        self.tangency2.record(
            index,
            a.point2,
            elclib::parameter(&solution, a.point2)?,
            0.0,
        )?;
        self.centre_on(index, centre, elclib::parameter(&a.on_circ, centre)?)
    }

    fn candidate_8(
        &mut self,
        a: &Args,
        center: Point2,
        radius: &mut Slots<f64>,
    ) -> Result<(), Failure> {
        let (c1, tol) = (a.q1.qualified(), a.tolerance.abs());
        let r1: f64 = c1.radius();
        let dist1: f64 = c1.location().distance(center);
        let mut nbsol: i32 = 1;
        let mut ok: bool = false;
        if a.q1.is_enclosed() || a.q1.is_outside() {
            let inside: bool = if a.q1.is_enclosed() {
                dist1 - r1 <= tol
            } else {
                r1 - dist1 <= tol
            };
            if inside {
                ok = true;
                radius.set(1, (r1 - dist1).abs())?;
            }
        } else if a.q1.is_enclosing() {
            ok = true;
            radius.set(1, r1 + dist1)?;
        } else if a.q1.is_unqualified() {
            ok = true;
            nbsol = 2;
            radius.set(1, (r1 - dist1).abs())?;
            radius.set(2, r1 + dist1)?;
        }
        if !ok {
            return Ok(());
        }
        for k in 1..=nbsol {
            // pop: protection against a centre at the centre of C1.
            if c1.location().is_equal(center, precision::confusion())
                || a.on_circ.distance(center) > tol
            {
                continue;
            }
            if self.nbr_sol == 4 {
                break;
            }
            self.solution_8(a, center, radius.get(k)?)?;
        }
        Ok(())
    }

    fn solution_8(&mut self, a: &Args, center: Point2, r: f64) -> Result<(), Failure> {
        let (c1, tol) = (a.q1.qualified(), a.tolerance.abs());
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, r)?;
        self.cirsol.set(index, solution)?;
        let distcc1: f64 = center.distance(c1.location());
        self.set_qualifiers(
            index,
            qualify(a.q1.qualifier(), distcc1, r, c1.radius(), tol, false),
            Position::NoQualifier,
        )?;
        if distcc1 <= tol && (r - c1.radius()).abs() <= tol {
            self.tangency1.same.set(index, 1)?;
        } else {
            let dc1 = Direction2::from_xy(c1.location().xy() - center.xy())?;
            let p1 = Point2::from(center.xy() + r * dc1.xy());
            self.tangency1.record(
                index,
                p1,
                elclib::parameter(&solution, p1)?,
                elclib::parameter(&c1, p1)?,
            )?;
        }
        self.tangency2.record(
            index,
            a.point2,
            elclib::parameter(&solution, a.point2)?,
            0.0,
        )?;
        self.centre_on(index, center, elclib::parameter(&a.on_circ, center)?)
    }
}
