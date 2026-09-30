// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx
// (EvalMaxParametricDistance, EvalMaxDistanceAlongParameter)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The maximum distance between two curves at given parameters.
use crate::extrema::LocateExtPC;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;

/// `GeomLib::EvalMaxParametricDistance(Curve, AReferenceCurve, Tolerance, Parameters,
/// MaxDistance)`: the greatest distance between the points of the two curves at the
/// same parameters (EvalD0); the tolerance is unused, as natively;
/// `geom_lib::eval_max_parametric_distance(&c, &reference, 1e-7, &params)?`.
pub fn eval_max_parametric_distance(
    curve: &dyn Adaptor3dCurve,
    reference_curve: &dyn Adaptor3dCurve,
    _tolerance: f64,
    parameters: &Array1<'_, f64>,
) -> Result<f64, Failure> {
    let mut max_squared: f64 = 0.0;
    for parameter in parameters.iter() {
        let point1: Point3 = curve.eval_d0(*parameter)?;
        let point2: Point3 = reference_curve.eval_d0(*parameter)?;
        max_squared = std_max(max_squared, point1.square_distance(&point2));
    }
    Ok(max_squared.sqrt())
}

/// `GeomLib::EvalMaxDistanceAlongParameter(Curve, AReferenceCurve, Tolerance,
/// Parameters, MaxDistance)`: at each parameter where the two curves are farther
/// than `tolerance`, the distance from the curve point to its local projection
/// (Extrema_LocateExtPC started from the previous projection, the parameter itself
/// when the projection fails with a zero distance); the greatest distance, or
/// `tolerance` when every distance is within it;
/// `geom_lib::eval_max_distance_along_parameter(&c, &reference, 1e-7, &params)?`.
pub fn eval_max_distance_along_parameter(
    curve: &dyn Adaptor3dCurve,
    reference_curve: &dyn Adaptor3dCurve,
    tolerance: f64,
    parameters: &Array1<'_, f64>,
) -> Result<f64, Failure> {
    let mut max_squared: f64 = 0.0;
    let tolerance_squared: f64 = tolerance * tolerance;
    let first: f64 = *parameters
        .first()
        .map_err(|e| crate::native_failure::array_failure(e, "Parameters"))?;
    let mut other_parameter: f64 = first;
    let para_tolerance: f64 = reference_curve.resolution(tolerance)?;
    let initial_point: Point3 = curve.eval_d0(other_parameter)?;
    let mut projector = LocateExtPC::with_curve(
        initial_point,
        reference_curve,
        other_parameter,
        para_tolerance,
    )?;
    for parameter in parameters.iter().copied() {
        let point: Point3 = curve.eval_d0(parameter)?;
        let mut local_distance_squared: f64 =
            point.square_distance(&reference_curve.eval_d0(parameter)?);
        if local_distance_squared > tolerance_squared {
            projector.perform(point, other_parameter)?;
            if projector.is_done() {
                other_parameter = projector.point()?.parameter();
                local_distance_squared =
                    point.square_distance(&reference_curve.eval_d0(other_parameter)?);
            } else {
                local_distance_squared = 0.0;
                other_parameter = parameter;
            }
        } else {
            other_parameter = parameter;
        }
        max_squared = std_max(max_squared, local_distance_squared);
    }
    if max_squared > tolerance_squared {
        return Ok(max_squared.sqrt());
    }
    Ok(tolerance)
}
