// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_EdgeTool.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_EdgeTool`: the curve services the linear integrator reads.
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Point3, Vector3};

/// `BRepGProp_EdgeTool`: static services on a `BRepAdaptor_Curve` (unit struct with
/// associated functions).
///
/// ```
/// use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
/// use rustycad_tktopalgo::brep_gprop::BRepGPropEdgeTool;
/// // An adaptor without curve fails to answer its type.
/// assert!(BRepGPropEdgeTool::integration_order(&BRepAdaptorCurve::default()).is_err());
/// ```
pub struct BRepGPropEdgeTool;

impl BRepGPropEdgeTool {
    /// `FirstParameter(C)`; `BRepGPropEdgeTool::first_parameter(&curve)?`.
    pub fn first_parameter(curve: &BRepAdaptorCurve) -> Result<f64, Failure> {
        curve.first_parameter()
    }

    /// `LastParameter(C)`; `BRepGPropEdgeTool::last_parameter(&curve)?`.
    pub fn last_parameter(curve: &BRepAdaptorCurve) -> Result<f64, Failure> {
        curve.last_parameter()
    }

    /// `IntegrationOrder(C)`: 2 for a line, 5 for a parabola, `2 * NbPoles - 1` for a
    /// Bezier or B-spline curve (the adaptor's own pole count, which handles a curve on
    /// surface), 10 otherwise; `BRepGPropEdgeTool::integration_order(&curve)?`.
    pub fn integration_order(curve: &BRepAdaptorCurve) -> Result<i32, Failure> {
        Ok(match curve.get_type()? {
            CurveType::Line => 2,
            CurveType::Parabola => 5,
            CurveType::BezierCurve | CurveType::BSplineCurve => 2 * curve.nb_poles()? - 1,
            _ => 10,
        })
    }

    /// `Value(C, U)`; `BRepGPropEdgeTool::value(&curve, 0.5)?`.
    pub fn value(curve: &BRepAdaptorCurve, u: f64) -> Result<Point3, Failure> {
        curve.value(u)
    }

    /// `D1(C, U, P, V1)`: the point and first derivative (returned);
    /// `let (p, v) = BRepGPropEdgeTool::d1(&curve, 0.5)?;`.
    pub fn d1(curve: &BRepAdaptorCurve, u: f64) -> Result<(Point3, Vector3), Failure> {
        curve.d1(u)
    }

    /// `NbIntervals(C, S)`; `BRepGPropEdgeTool::nb_intervals(&curve, Shape::CN)?`.
    pub fn nb_intervals(curve: &BRepAdaptorCurve, continuity: Shape) -> Result<i32, Failure> {
        curve.nb_intervals(continuity)
    }

    /// `Intervals(C, T, S)`: the interval bounds of continuity `continuity` into `bounds`;
    /// `BRepGPropEdgeTool::intervals(&curve, &mut bounds, Shape::CN)?`.
    pub fn intervals(
        curve: &BRepAdaptorCurve,
        bounds: &mut Array1<'_, f64>,
        continuity: Shape,
    ) -> Result<(), Failure> {
        curve.intervals(bounds, continuity)
    }
}
