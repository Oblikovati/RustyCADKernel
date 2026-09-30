// Rust translation of GccAna_Circ2d3Tan_2.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to a circle and two lines: the intersections of the
//! circle-line bisectors of (C1, L2) with the bisectors of (L2, L3).
use super::candidates::RadiusCandidates;
use super::{Argument, Circ2d3Tan, check_line_qualifier, line_qualifier, normal_of, side_of};
use crate::gcc_ana::solutions::qualify;
use crate::gcc_ana::{CircLin2dBisec, Lin2dBisec, check_qualifier};
use crate::gcc_ent::{QualifiedCirc, QualifiedLin};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::gp::{Circle2, Direction2, Point2};

struct Args {
    q1: QualifiedCirc,
    q2: QualifiedLin,
    q3: QualifiedLin,
    tolerance: f64,
}

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Qualified1, Qualified2, Qualified3, Tolerance) with a
    /// circle and two lines, up to 8 solutions;
    /// `Circ2d3Tan::circ_lin_lin(q1, l2, l3, tol)?`.
    pub fn circ_lin_lin(
        qualified1: QualifiedCirc,
        qualified2: QualifiedLin,
        qualified3: QualifiedLin,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(8);
        check_qualifier(qualified1.qualifier(), super::NAME)?;
        check_line_qualifier(qualified2)?;
        check_line_qualifier(qualified3)?;
        let args = Args {
            q1: qualified1,
            q2: qualified2,
            q3: qualified3,
            tolerance,
        };
        let (l2, l3) = (qualified2.qualified(), qualified3.qualified());
        let bis1 = CircLin2dBisec::new(qualified1.qualified(), l2);
        let bis2 = Lin2dBisec::new(l2, l3)?;
        let mut radius = RadiusCandidates::new();
        if !(bis1.is_done() && bis2.is_done()) {
            return Ok(circ);
        }
        for i in 1..=bis1.nb_solutions() {
            let sol1: Handle<dyn Bisec> = bis1.this_solution(i)?;
            let mut intp = AnaIntersection::new();
            for k in 1..=bis2.nb_solutions()? {
                match sol1.arc_type() {
                    IType::Lin => {
                        intp = AnaIntersection::lines(sol1.line()?, bis2.this_solution(k)?)
                    }
                    IType::Par => {
                        let conic = Conic::from_parabola(sol1.parabola()?);
                        intp = AnaIntersection::line_conic(bis2.this_solution(k)?, &conic)?;
                    }
                    _ => {}
                }
                if !intp.is_done() {
                    continue;
                }
                if !intp.is_empty()? && !intp.parallel_elements()? && !intp.identical_elements()? {
                    for j in 1..=intp.nb_points()? {
                        circ.centre_2(&args, intp.point(j)?.value(), &mut radius)?;
                    }
                }
                circ.well_done = true;
            }
        }
        Ok(circ)
    }

    fn centre_2(
        &mut self,
        a: &Args,
        center: Point2,
        radius: &mut RadiusCandidates,
    ) -> Result<(), Failure> {
        let (c1, l2, l3) = (a.q1.qualified(), a.q2.qualified(), a.q3.qualified());
        let tol: f64 = a.tolerance.abs();
        let dist1: f64 = center.distance(c1.location());
        let dist2: f64 = l2.distance(center);
        let dist3: f64 = l3.distance(center);
        let nbsol1: i32 = radius.first_circle(a.q1, dist1, a.tolerance, false);
        radius.line_radius(a.q2, side_of(l2, center), dist2, nbsol1, tol);
        let side3: f64 = side_of(l3, center);
        let allowed: bool = if a.q3.is_enclosed() {
            side3 <= 0.0
        } else if a.q3.is_outside() {
            side3 >= 0.0
        } else {
            a.q3.is_unqualified()
        };
        let nbsol3: i32 = i32::from(radius.ok && allowed && (dist3 - radius.at(1)).abs() < tol);
        if !radius.ok {
            return Ok(());
        }
        for m in 1..=nbsol3 {
            self.solution_2(a, center, radius.at(m))?;
        }
        Ok(())
    }

    fn solution_2(&mut self, a: &Args, center: Point2, r: f64) -> Result<(), Failure> {
        let (c1, l2, l3) = (a.q1.qualified(), a.q2.qualified(), a.q3.qualified());
        let tol: f64 = a.tolerance.abs();
        let index: i32 = self.push_circle(center, r)?;
        let distcc1: f64 = center.distance(c1.location());
        self.set_qualifier(
            1,
            index,
            qualify(a.q1.qualifier(), distcc1, r, c1.radius(), tol, false),
        )?;
        self.set_qualifier(2, index, line_qualifier(a.q2, center, normal_of(l2)?)?)?;
        self.set_qualifier(3, index, line_qualifier(a.q3, center, normal_of(l3)?)?)?;
        self.circle_touch_plain(1, index, center, r, c1, a.tolerance)?;
        self.touch_line(2, index, l2)?;
        self.touch_line(3, index, l3)
    }

    // The tangency on a circle argument without POP guards: TheSame when the
    // solution is that circle, else Center + r * Dir(circle centre - Center).
    pub(super) fn circle_touch_plain(
        &mut self,
        which: usize,
        index: i32,
        center: Point2,
        r: f64,
        circle: Circle2,
        tolerance: f64,
    ) -> Result<(), Failure> {
        if center.distance(circle.location()) <= tolerance
            && (r - circle.radius()).abs() <= tolerance
        {
            return self.mark_same(which, index);
        }
        let dc = Direction2::from_xy(circle.location().xy() - center.xy())?;
        let point = Point2::from(center.xy() + r * dc.xy());
        self.touch(which, index, point, Argument::Circle(circle))
    }
}
