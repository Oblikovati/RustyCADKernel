// Rust translation of GccAna_Circ2d3Tan_8.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to a line and through two points: the intersections of the
//! bisector of the points with the bisector of (L1, Point2).
use super::{Argument, Circ2d3Tan, check_line_qualifier, line_qualifier, normal_of, side_of};
use crate::gcc_ana::{LinPnt2dBisec, Pnt2dBisec};
use crate::gcc_ent::{Position, QualifiedLin};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::gp::{Line2, Point2};

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Qualified1, Point2, Point3, Tolerance): the circles
    /// tangent to a line and through two points, at most two (not done for
    /// coincident points); `Circ2d3Tan::lin_pnt_pnt(l1, p2, p3, tol)?`.
    pub fn lin_pnt_pnt(
        qualified1: QualifiedLin,
        point2: Point2,
        point3: Point2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        check_line_qualifier(qualified1)?;
        let l1: Line2 = qualified1.qualified();
        if point2.is_equal(point3, precision::confusion()) {
            return Ok(circ);
        }
        let bis1 = Pnt2dBisec::new(point2, point3)?;
        let bis2 = LinPnt2dBisec::new(l1, point2)?;
        if !(bis1.is_done() && bis2.is_done()) {
            return Ok(circ);
        }
        let linint1: Line2 = bis1.this_solution()?;
        let sol2: Handle<dyn Bisec> = bis2.this_solution()?;
        let intp = match sol2.arc_type() {
            IType::Lin => AnaIntersection::lines(linint1, sol2.line()?),
            IType::Par => {
                AnaIntersection::line_conic(linint1, &Conic::from_parabola(sol2.parabola()?))?
            }
            _ => AnaIntersection::new(),
        };
        if !intp.is_done() {
            return Ok(circ);
        }
        if !intp.is_empty()? && !intp.parallel_elements()? && !intp.identical_elements()? {
            for j in 1..=intp.nb_points()? {
                let center: Point2 = intp.point(j)?.value();
                circ.centre_8(qualified1, center, (point2, point3), tolerance.abs())?;
            }
        }
        circ.well_done = true;
        Ok(circ)
    }

    fn centre_8(
        &mut self,
        qualified1: QualifiedLin,
        center: Point2,
        (point2, point3): (Point2, Point2),
        tol: f64,
    ) -> Result<(), Failure> {
        let l1: Line2 = qualified1.qualified();
        let dist1: f64 = l1.distance(center);
        let dist2: f64 = center.distance(point2);
        let difference: f64 = side_of(l1, center);
        let accepted: bool = (qualified1.is_enclosed() && difference <= 0.0)
            || (qualified1.is_outside() && difference >= 0.0)
            || qualified1.is_unqualified();
        let touches: bool = (dist2 - dist1).abs() <= tol;
        if !accepted || !touches {
            return Ok(());
        }
        let index: i32 = self.push_circle(center, dist1)?;
        self.set_qualifier(
            1,
            index,
            line_qualifier(qualified1, center, normal_of(l1)?)?,
        )?;
        self.set_qualifier(2, index, Position::NoQualifier)?;
        self.set_qualifier(3, index, Position::NoQualifier)?;
        self.touch_line(1, index, l1)?;
        self.touch(2, index, point2, Argument::Point)?;
        self.touch(3, index, point3, Argument::Point)
    }
}
