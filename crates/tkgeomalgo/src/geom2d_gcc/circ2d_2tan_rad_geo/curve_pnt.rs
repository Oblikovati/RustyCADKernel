// Rust translation of the curve-point constructor of Geom2dGcc_Circ2d2TanRadGeo.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles of given radius tangent to a curve and through a point.
use super::{Circ2d2TanRadGeo, NB_SOL_MAX, offset_of};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::Position;
use crate::geom2d_gcc::domains::{clamped_domain, closed_domain};
use crate::geom2d_gcc::{CurveTool, QCurve};
use crate::geom2d_int::TheIntConicCurveOfGInter;
use crate::int_res2d::Domain;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

impl Circ2d2TanRadGeo {
    /// `Geom2dGcc_Circ2d2TanRadGeo(Qualified1, Point2, Radius, Tolerance)`: the circle
    /// of radius `Radius` about the point intersected with the curve offset by the
    /// radius; an enclosing curve gives no solution (and is not done);
    /// `Circ2d2TanRadGeo::curve_pnt(&q1, point, 1.0, 1e-9)?`.
    pub fn curve_pnt(
        qualified1: &QCurve,
        point2: Point2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let qualifiers_ok = check_qualifier(qualified1.qualifier(), "Geom2dGcc_Circ2d2TanRadGeo");
        let mut circ = Self::start(qualifiers_ok, radius)?;
        let tol: f64 = tolerance.abs();
        let cote1: Vec<f64> = match qualified1.qualifier() {
            Position::Enclosed => vec![radius],
            Position::Outside => vec![-radius],
            Position::Unqualified => vec![radius, -radius],
            _ => Vec::new(),
        };
        let circle = circle_at(point2, radius)?;
        let d1: Domain = closed_domain(&circle, PI + PI, (tol, tol))?;
        let mut intp = TheIntConicCurveOfGInter::new();
        for &side in &cote1 {
            if circ.solutions.nbr_sol >= NB_SOL_MAX {
                break;
            }
            let offset = offset_of(qualified1, side)?;
            let d2: Domain = clamped_domain(&offset, tol)?;
            intp.perform_circle(circle, &d1, &offset, &d2, tol, tol)?;
            circ.take_point_centres(&intp, qualified1, point2, radius)?;
        }
        Ok(circ)
    }

    // The centres of one circle-offset intersection; the curve parameter is the
    // second one.
    fn take_point_centres(
        &mut self,
        intp: &TheIntConicCurveOfGInter,
        qualified1: &QCurve,
        point2: Point2,
        radius: f64,
    ) -> Result<(), Failure> {
        if !intp.is_done() {
            return Ok(());
        }
        if !intp.is_empty()? {
            let mut i: i32 = 1;
            while i <= intp.nb_points()? && self.solutions.nbr_sol < NB_SOL_MAX {
                let point = intp.point(i)?;
                let pararg1: f64 = point.param_on_second();
                let pnttg1sol: Point2 = CurveTool::value(qualified1.curve(), pararg1)?;
                let qualifiers = (qualified1.qualifier(), Position::NoQualifier);
                let tangencies = [(pnttg1sol, pararg1), (point2, 0.0)];
                self.push(point.value(), radius, qualifiers, tangencies)?;
                i += 1;
            }
        }
        self.solutions.well_done = true;
        Ok(())
    }
}
