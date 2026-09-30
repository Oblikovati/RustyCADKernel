// Rust translation of GccAna_Circ2d3Tan_6.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to two lines and through a point: the intersections of the
//! bisectors of (L1, L2) with the bisector of (L1, Point3). Only two solution
//! slots exist natively; a third solution raises Standard_OutOfRange.
use super::{Argument, Circ2d3Tan, check_line_qualifier, line_qualifier, normal_of, side_of};
use crate::gcc_ana::{Lin2dBisec, LinPnt2dBisec};
use crate::gcc_ent::{Position, QualifiedLin};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::gp::{Line2, Point2};

struct Args {
    q1: QualifiedLin,
    q2: QualifiedLin,
    point3: Point2,
    tolerance: f64,
}

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Qualified1, Qualified2, Point3, Tolerance) with two lines
    /// and a point; `Circ2d3Tan::lin_lin_pnt(l1, l2, point, tol)?`.
    pub fn lin_lin_pnt(
        qualified1: QualifiedLin,
        qualified2: QualifiedLin,
        point3: Point2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        check_line_qualifier(qualified1)?;
        check_line_qualifier(qualified2)?;
        for index in 1..=2 {
            circ.tangency3.point.set(index, point3)?;
        }
        let args = Args {
            q1: qualified1,
            q2: qualified2,
            point3,
            tolerance,
        };
        let (l1, l2) = (qualified1.qualified(), qualified2.qualified());
        let bis1 = Lin2dBisec::new(l1, l2)?;
        let bis2 = LinPnt2dBisec::new(l1, point3)?;
        if !(bis1.is_done() && bis2.is_done()) {
            return Ok(circ);
        }
        let sol2: Handle<dyn Bisec> = bis2.this_solution()?;
        for i in 1..=bis1.nb_solutions()? {
            let sol1: Line2 = bis1.this_solution(i)?;
            let intp = match sol2.arc_type() {
                IType::Lin => AnaIntersection::lines(sol1, sol2.line()?),
                IType::Par => {
                    AnaIntersection::line_conic(sol1, &Conic::from_parabola(sol2.parabola()?))?
                }
                _ => AnaIntersection::new(),
            };
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                for j in 1..=intp.nb_points()? {
                    circ.centre_6(&args, intp.point(j)?.value())?;
                }
            }
            circ.well_done = true;
        }
        Ok(circ)
    }

    fn centre_6(&mut self, a: &Args, center: Point2) -> Result<(), Failure> {
        let (l1, l2) = (a.q1.qualified(), a.q2.qualified());
        let tol: f64 = a.tolerance.abs();
        let dist1: f64 = l1.distance(center);
        let dist2: f64 = l2.distance(center);
        let dist3: f64 = center.distance(a.point3);
        let side1: f64 = side_of(l1, center);
        let mut radius: f64 = 0.0;
        let mut ok = false;
        if (a.q1.is_enclosed() && side1 <= 0.0)
            || (a.q1.is_outside() && side1 >= 0.0)
            || a.q1.is_unqualified()
        {
            ok = true;
            radius = dist1;
        }
        // Native tests an enclosed second line whatever `ok` is.
        let side2: f64 = side_of(l2, center);
        let checks2: bool = if a.q2.is_enclosed() {
            side2 <= 0.0
        } else if a.q2.is_outside() && ok {
            side2 >= 0.0
        } else {
            a.q2.is_unqualified() && ok
        };
        let touches2: bool = (dist2 - radius).abs() < tol;
        if checks2 && !touches2 {
            ok = false;
        }
        let touches3: bool = (dist3 - radius).abs() < tol;
        if ok && !touches3 {
            ok = false;
        }
        if ok {
            self.solution_6(a, center, radius)?;
        }
        Ok(())
    }

    fn solution_6(&mut self, a: &Args, center: Point2, r: f64) -> Result<(), Failure> {
        let (l1, l2) = (a.q1.qualified(), a.q2.qualified());
        let index: i32 = self.push_circle(center, r)?;
        self.set_qualifier(1, index, line_qualifier(a.q1, center, normal_of(l1)?)?)?;
        self.set_qualifier(2, index, line_qualifier(a.q2, center, normal_of(l2)?)?)?;
        self.set_qualifier(3, index, Position::NoQualifier)?;
        self.touch_line(1, index, l1)?;
        self.touch_line(2, index, l2)?;
        let point3: Point2 = self.tangency3.point.get(index)?;
        self.touch(3, index, point3, Argument::Point)
    }
}
