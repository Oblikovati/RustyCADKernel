// Rust translation of the circle-curve constructor of
// Geom2dGcc_Circ2d2TanRadGeo.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles of given radius tangent to a circle and a curve.
use super::{Circ2d2TanRadGeo, NB_SOL_MAX, cotes, offset_of};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::qualify;
use crate::gcc_ent::QualifiedCirc;
use crate::geom2d_gcc::domains::{clamped_domain, closed_domain};
use crate::geom2d_gcc::{CurveTool, QCurve};
use crate::geom2d_int::TheIntConicCurveOfGInter;
use crate::int_res2d::Domain;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Point2};
use std::f64::consts::PI;

const MEMBER: &str = "Geom2dGcc_Circ2d2TanRadGeo";

impl Circ2d2TanRadGeo {
    /// `Geom2dGcc_Circ2d2TanRadGeo(GccEnt_QualifiedCirc Qualified1, Qualified2, Radius,
    /// Tolerance)`: the circle offset by the radius intersected with the offset curve;
    /// an unqualified circle qualifies each solution by the distance of its centre.
    /// A negative offset radius raises Standard_ConstructionError, as natively;
    /// `Circ2d2TanRadGeo::circ_curve(qc, &q2, 1.0, 1e-9)?`.
    pub fn circ_curve(
        qualified1: QualifiedCirc,
        qualified2: &QCurve,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let qualifiers_ok = check_qualifier(qualified1.qualifier(), MEMBER)
            .and_then(|()| check_qualifier(qualified2.qualifier(), MEMBER));
        let mut circ = Self::start(qualifiers_ok, radius)?;
        let tol: f64 = tolerance.abs();
        let c1: Circle2 = qualified1.qualified();
        let r1: f64 = c1.radius();
        let (cote1, cote2) = cotes(qualified1.qualifier(), qualified2.qualifier(), radius);
        let mut intp = TheIntConicCurveOfGInter::new();
        for &side1 in &cote1 {
            if circ.solutions.nbr_sol >= NB_SOL_MAX {
                break;
            }
            let circle = Circle2::from_x_axis(c1.x_axis(), r1 + side1, true)?;
            let d1: Domain = closed_domain(&circle, 2.0 * PI, (tol, tol))?;
            for &side2 in &cote2 {
                let offset = offset_of(qualified2, side2)?;
                let d2: Domain = clamped_domain(&offset, tol)?;
                intp.perform_circle(circle, &d1, &offset, &d2, tol, tol)?;
                circ.take_circle_points(&intp, qualified1, qualified2, radius, tol)?;
            }
        }
        Ok(circ)
    }

    // The centres of one circle-offset intersection.
    fn take_circle_points(
        &mut self,
        intp: &TheIntConicCurveOfGInter,
        qualified1: QualifiedCirc,
        qualified2: &QCurve,
        radius: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        if !intp.is_done() {
            return Ok(());
        }
        let c1: Circle2 = qualified1.qualified();
        if !intp.is_empty()? {
            let mut i: i32 = 1;
            while i <= intp.nb_points()? && self.solutions.nbr_sol < NB_SOL_MAX {
                let point = intp.point(i)?;
                let center: Point2 = point.value();
                let distcc1: f64 = center.distance(c1.location());
                let q1 = qualify(
                    qualified1.qualifier(),
                    distcc1,
                    radius,
                    c1.radius(),
                    tol,
                    false,
                );
                let (pararg1, pararg2) = (point.param_on_first(), point.param_on_second());
                let pnttg1sol: Point2 = elclib::value(pararg1, &c1);
                let pnttg2sol: Point2 = CurveTool::value(qualified2.curve(), pararg2)?;
                let tangencies = [(pnttg1sol, pararg1), (pnttg2sol, pararg2)];
                self.push(center, radius, (q1, qualified2.qualifier()), tangencies)?;
                i += 1;
            }
        }
        self.solutions.well_done = true;
        Ok(())
    }
}
