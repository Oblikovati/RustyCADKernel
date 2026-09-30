// ShapeAnalysis_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Curve analysis with native extrema, endpoint and sampling decisions.
mod periodic;
mod projection_act;
mod projection_elementary;
mod projection_endpoints;
mod projection_segments;
mod projection_state;
mod projection_wrappers;
pub use periodic::{adjust_by_period, adjust_to_period};
pub use projection_state::CurveProjection;
/// Distance, projected point and parameter; `let (distance, point, parameter) = result;`.
pub type CurveProjectionResult = (f64, rustycad_tkmath::gp::Point3, f64);
/// Stateless curve analysis; `let analysis = ShapeAnalysisCurve;`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeAnalysisCurve;

mod bounds;
mod closure;
mod planarity_curve;
mod planarity_points;
mod sampling;
mod sampling_3d;
mod seam;
mod validate_range;
