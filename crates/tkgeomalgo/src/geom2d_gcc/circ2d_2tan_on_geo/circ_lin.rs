// Rust translation of the circle-line constructor of Geom2dGcc_Circ2d2TanOnGeo.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to a circle and a line with their centre on a curve.
use super::{
    Circ2d2TanOnGeo, NB_SOL_MAX, circle_touch, line_qualifier, line_side_value, line_touch,
    on_curve_of, perform_bisector,
};
use crate::gcc_ana::solutions::qualify;
use crate::gcc_ana::{CircLin2dBisec, check_line_position, check_qualifier};
use crate::gcc_ent::{Position, QualifiedCirc, QualifiedLin};
use crate::gcc_int::IType;
use crate::geom2d_int::TheIntConicCurveOfGInter;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};

const MEMBER: &str = "Geom2dGcc_Circ2d2TanOnGeo";

impl Circ2d2TanOnGeo {
    /// `Geom2dGcc_Circ2d2TanOnGeo(GccEnt_QualifiedCirc Qualified1, GccEnt_QualifiedLin
    /// Qualified2, OnCurv, Tolerance)`: the circle-line bisectors (lines and
    /// parabolas) intersected with the curve; the radius is the distance to the
    /// line; `Circ2d2TanOnGeo::circ_lin(qc, ql, &on, 1e-9)?`.
    pub fn circ_lin(
        qualified1: QualifiedCirc,
        qualified2: QualifiedLin,
        on_curv: &Geom2dAdaptorCurve,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty();
        let tol: f64 = tolerance.abs();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_line_position(qualified2.qualifier(), MEMBER)?;
        let (c1, l2): (Circle2, Line2) = (qualified1.qualified(), qualified2.qualified());
        let normal: Direction2 = Direction2::new(-l2.direction().y(), l2.direction().x())?;
        let bis = CircLin2dBisec::new(c1, l2);
        if !bis.is_done() {
            return Ok(circ);
        }
        let mut intp = TheIntConicCurveOfGInter::new();
        let (c2, d2) = on_curve_of(on_curv, tol)?;
        for i in 1..=bis.nb_solutions() {
            let sol = bis.this_solution(i)?;
            perform_bisector(
                &mut intp,
                &sol,
                &[IType::Lin, IType::Par],
                (&c2, &d2),
                (tol, tol),
            )?;
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                let mut j: i32 = 1;
                while j <= intp.nb_points()? && circ.solutions.nbr_sol < NB_SOL_MAX {
                    let point = intp.point(j)?;
                    circ.circle_line_centre(
                        (qualified1, qualified2),
                        normal,
                        point.value(),
                        point.param_on_second(),
                        tol,
                    )?;
                    j += 1;
                }
            }
            circ.solutions.well_done = true;
        }
        Ok(circ)
    }

    // The solution at one centre, when it lies on the qualified sides.
    fn circle_line_centre(
        &mut self,
        (qualified1, qualified2): (QualifiedCirc, QualifiedLin),
        norm_l2: Direction2,
        center: Point2,
        parcen3: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let (c1, l2): (Circle2, Line2) = (qualified1.qualified(), qualified2.qualified());
        let r1: f64 = c1.radius();
        let dist1: f64 = center.distance(c1.location());
        let mut ok: bool = match qualified1.qualifier() {
            Position::Enclosed => dist1 - r1 < tol,
            Position::Outside => r1 - dist1 < tol,
            Position::Enclosing | Position::Unqualified => true,
            Position::NoQualifier => false,
        };
        let radius: f64 = l2.distance(center);
        if ok && qualified2.is_enclosed() {
            ok = line_side_value(&l2, center) <= 0.0;
        } else if ok && qualified2.is_outside() {
            ok = line_side_value(&l2, center) >= 0.0;
        }
        if qualified1.is_enclosing() && dist1 > radius {
            ok = false;
        }
        if !ok {
            return Ok(());
        }
        let (index, circle) = self.open(center, radius, parcen3)?;
        let dc2 = Direction2::from_xy(l2.location().xy() - center.xy())?;
        let q1 = qualify(qualified1.qualifier(), dist1, radius, r1, tol, false);
        self.solutions.qualifier[0].set(index, q1)?;
        self.solutions.qualifier[1]
            .set(index, line_qualifier(qualified2.qualifier(), dc2, norm_l2))?;
        if dist1 <= tol && (radius - r1).abs() <= tol {
            self.solutions.tangency[0].same.set(index, 1)?;
        } else {
            let point: Point2 = circle_touch(c1.location(), center, radius)?;
            self.touch(0, index, &circle, point, elclib::parameter(&c1, point)?)?;
        }
        let point2: Point2 = line_touch(&l2, center, radius)?;
        self.touch(1, index, &circle, point2, elclib::parameter(&l2, point2)?)
    }
}
