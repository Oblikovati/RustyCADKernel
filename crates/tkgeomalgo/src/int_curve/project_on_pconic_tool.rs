// Rust translation of IntCurve_ProjectOnPConicTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_ProjectOnPConicTool`: the parameter of a point near a [`PConic`].
use super::PConic;
use super::iconic_tool::positive_angle;
use crate::int_imp_par_gen::ProjectOnPCurveTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis22d, Point2};

/// `IntCurve_ProjectOnPConicTool`: the ElCLib parameter of the point on the conic
/// (in `[0, 2PI)` on a circle or an ellipse, a gp exception at their centre);
/// `ProjectOnPConicTool::find_parameter(&pconic, point, 1e-7)?`.
pub struct ProjectOnPConicTool;

impl ProjectOnPConicTool {
    /// `FindParameter(C, Pnt, Tol)`: on the natural domain of the conic (the
    /// tolerance is unused). `ProjectOnPConicTool::find_parameter(&pconic, point, 1e-7)?`.
    pub fn find_parameter(the_pconic: &PConic, p: Point2, _tol: f64) -> Result<f64, Failure> {
        let axe: Axis22d = the_pconic.axis2();
        let (prm1, prm2) = (the_pconic.param1(), the_pconic.param2());
        let param: f64 = match the_pconic.type_curve() {
            CurveType::Line => elclib::line_parameter_2d(axe.x_axis(), p),
            CurveType::Circle => positive_angle(elclib::circle_parameter_2d(axe, p)?),
            CurveType::Ellipse => positive_angle(elclib::ellipse_parameter_2d(axe, prm1, prm2, p)?),
            CurveType::Parabola => elclib::parabola_parameter_2d(axe, p),
            CurveType::Hyperbola => elclib::hyperbola_parameter_2d(axe, prm1, prm2, p),
            _ => 0.0,
        };
        Ok(param)
    }
    /// `FindParameter(C, Pnt, LowParameter, HighParameter, Tol)`: the same
    /// parameter clamped to the bounds (in either order) unless they are equal.
    /// `ProjectOnPConicTool::find_parameter_in(&pconic, point, 0.0, 1.0, 1e-7)?`.
    pub fn find_parameter_in(
        the_pconic: &PConic,
        p: Point2,
        low_parameter: f64,
        high_parameter: f64,
        tol: f64,
    ) -> Result<f64, Failure> {
        let (param_inf, param_sup) = if low_parameter > high_parameter {
            (high_parameter, low_parameter)
        } else {
            (low_parameter, high_parameter)
        };
        let param: f64 = Self::find_parameter(the_pconic, p, tol)?;
        if param_inf != param_sup {
            if param < param_inf {
                return Ok(param_inf);
            }
            if param > param_sup {
                return Ok(param_sup);
            }
        }
        Ok(param)
    }
}

impl ProjectOnPCurveTool for ProjectOnPConicTool {
    type Curve = PConic;
    fn find_parameter(curve: &PConic, pnt: Point2, tol: f64) -> Result<f64, Failure> {
        ProjectOnPConicTool::find_parameter(curve, pnt, tol)
    }
    fn find_parameter_in(
        curve: &PConic,
        pnt: Point2,
        low_parameter: f64,
        high_parameter: f64,
        tol: f64,
    ) -> Result<f64, Failure> {
        ProjectOnPConicTool::find_parameter_in(curve, pnt, low_parameter, high_parameter, tol)
    }
}
