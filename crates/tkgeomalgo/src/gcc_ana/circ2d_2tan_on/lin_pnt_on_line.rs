// Rust translation of GccAna_Circ2d2TanOn_4.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to a line and through a point with their centre on a line: a
//! boundary case centred at the foot of the point on the locus, else the
//! intersections of the locus with the line-point bisector.
use super::Circ2d2TanOn;
use super::circ_lin_on_line::{check_line, side_of};
use super::circ_pnt_on_line::foot_on_line;
use crate::gcc_ana::LinPnt2dBisec;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::{Position, QualifiedLin};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Point2, OnLine, Tolerance): circles tangent
    /// to a qualified line and through a point, centred on `on_line` (none when
    /// the line is the locus); `Circ2d2TanOn::lin_pnt_on_line(l1, point, line, tol)?`.
    pub fn lin_pnt_on_line(
        qualified1: QualifiedLin,
        point2: Point2,
        on_line: Line2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        check_line(qualified1)?;
        let tol: f64 = tolerance.abs();
        let l1: Line2 = qualified1.qualified();
        let dir_l1: Direction2 = l1.direction();
        let normal = Direction2::new(-dir_l1.y(), dir_l1.x())?;
        // POP: the two lines are identical, no solution.
        if dir_l1.is_equal(on_line.direction(), precision::confusion())
            && on_line.distance(l1.location()) < precision::confusion()
        {
            return Ok(circ);
        }
        let (pinterm, dp2l) = foot_on_line(point2, on_line, tol);
        if (l1.distance(pinterm) - dp2l).abs() <= tol {
            let dirbid = Direction2::from_xy(l1.location().xy() - pinterm.xy())?;
            circ.well_done = (qualified1.is_enclosed() && dirbid.dot(normal) < 0.0)
                || (qualified1.is_outside() && dirbid.dot(normal) > 0.0)
                || qualified1.is_unqualified();
            if circ.well_done {
                // The boundary case scales the raw (-dy, dx), not the renormalized normal.
                let raw = rustycad_tkmath::gp::Xy::new(-dir_l1.y(), dir_l1.x());
                circ.foot_solution_4(qualified1, point2, on_line, (pinterm, dp2l, normal), raw)?;
                return Ok(circ);
            }
        }
        let bis = LinPnt2dBisec::new(l1, point2)?;
        if !bis.is_done() {
            return Ok(circ);
        }
        let sol: Handle<dyn Bisec> = bis.this_solution()?;
        let intp = match sol.arc_type() {
            IType::Lin => AnaIntersection::lines(on_line, sol.line()?),
            IType::Par => {
                AnaIntersection::line_conic(on_line, &Conic::from_parabola(sol.parabola()?))?
            }
            _ => AnaIntersection::new(),
        };
        if !intp.is_done() {
            return Ok(circ);
        }
        if !intp.is_empty()? {
            for j in 1..=intp.nb_points()? {
                let center: Point2 = intp.point(j)?.value();
                let side: f64 = side_of(l1, center);
                let ok: bool = (qualified1.is_enclosed() && side <= 0.0)
                    || (qualified1.is_outside() && side >= 0.0)
                    || qualified1.is_unqualified();
                if ok {
                    circ.foot_solution_4(
                        qualified1,
                        point2,
                        on_line,
                        (center, l1.distance(center), normal),
                        normal.xy(),
                    )?;
                }
            }
        }
        circ.well_done = true;
        Ok(circ)
    }

    // A circle centred at `center` (radius `radius`) tangent to the line through
    // the foot Center + radius * Dir(sign * normal) and through the point.
    fn foot_solution_4(
        &mut self,
        qualified1: QualifiedLin,
        point2: Point2,
        on_line: Line2,
        (center, radius, normal): (Point2, f64, Direction2),
        scaled: rustycad_tkmath::gp::Xy,
    ) -> Result<(), Failure> {
        let l1: Line2 = qualified1.qualified();
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(index, solution)?;
        let dc2 = Direction2::from_xy(l1.location().xy() - center.xy())?;
        let q1 = if !qualified1.is_unqualified() {
            qualified1.qualifier()
        } else if dc2.dot(normal) > 0.0 {
            Position::Outside
        } else {
            Position::Enclosed
        };
        self.set_qualifiers(index, q1, Position::NoQualifier)?;
        let sign: f64 = dc2.dot(normal);
        let foot = Direction2::from_xy(sign * scaled)?;
        let p1 = Point2::from(center.xy() + radius * foot.xy());
        self.tangency1.record(
            index,
            p1,
            elclib::parameter(&solution, p1)?,
            elclib::parameter(&l1, p1)?,
        )?;
        self.tangency2
            .record(index, point2, elclib::parameter(&solution, point2)?, 0.0)?;
        self.centre_on(index, center, elclib::parameter(&on_line, center)?)
    }
}
