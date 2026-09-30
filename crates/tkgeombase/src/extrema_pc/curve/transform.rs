// Rust translation of initFromTransformedCurve, postProcessTransform and the transformed
// branch of Perform in ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Curve.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GeomAdaptor_TransformedCurve support of ExtremaPC_Curve: a 3D curve is solved in
//! its original space so that its parameterization is kept; a curve on a surface is
//! evaluated through the transformed adaptor.
use super::super::{
    Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status, is_valid_tolerance,
};
use super::super::{OffsetCurve, OtherCurve};
use super::{EvaluatorVariant, dispatch, non_identity, shared};
use crate::extrema_pc_core::PcCurveRef;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorTransformedCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point3, Transform3};

/// `initFromTransformedCurve`: the evaluator and the forward transformation (none for
/// the identity or a curve on a surface).
pub(super) fn from_transformed<'a>(
    curve: &'a GeomAdaptorTransformedCurve,
    domain: Option<Domain1D>,
) -> Result<(EvaluatorVariant<'a>, Option<Transform3>), Failure> {
    let domain: Domain1D = match domain {
        Some(domain) => domain,
        None => Domain1D::new(curve.first_parameter()?, curve.last_parameter()?),
    };
    if curve.is_3d_curve() {
        let evaluator = dispatch::from_adaptor(shared(curve.curve()), domain)?;
        return Ok((evaluator, non_identity(curve.trsf())));
    }
    let reference = PcCurveRef::Borrowed(curve as &dyn Adaptor3dCurve);
    if curve.get_type()? == CurveType::OffsetCurve {
        let evaluator = OffsetCurve::from_curve_ref(reference, Some(domain))?;
        return Ok((EvaluatorVariant::Offset(evaluator), None));
    }
    let evaluator = OtherCurve::from_curve_ref(reference, Some(domain))?;
    Ok((EvaluatorVariant::Other(evaluator), None))
}

/// The transformed branch of `Perform` / `PerformWithEndpoints`: InvalidInput for a bad
/// tolerance, NumericalError when the scaled tolerance is unusable; otherwise the query
/// is solved in the original space and post-processed.
pub(super) fn run<'r>(
    evaluator: &mut EvaluatorVariant<'_>,
    not_done: &ExtremaResult,
    transform_result: &'r mut ExtremaResult,
    trsf: Transform3,
    (point, tol, mode, ends): (Point3, f64, SearchMode, bool),
) -> Result<&'r ExtremaResult, Failure> {
    if !is_valid_tolerance(tol) {
        transform_result.clear();
        transform_result.status = Status::InvalidInput;
        return Ok(transform_result);
    }
    // Inverse-transform query point and delegate to untransformed evaluator
    let local_tol: f64 = tol / trsf.scale_factor().abs();
    if !is_valid_tolerance(local_tol) {
        transform_result.clear();
        transform_result.status = Status::NumericalError;
        return Ok(transform_result);
    }
    let inverse: Point3 = point.transformed(trsf.inverted()?);
    let source = dispatch::run(evaluator, not_done, (inverse, local_tol, mode, ends))?;
    post_process(source, point, trsf, transform_result);
    Ok(transform_result)
}

/// `postProcessTransform`: forward-transform the points, scale the infinite distance and
/// recompute the distances to the original query.
fn post_process(
    source: &ExtremaResult,
    point: Point3,
    trsf: Transform3,
    target: &mut ExtremaResult,
) {
    let scale: f64 = trsf.scale_factor().abs();
    target.clear();
    target.status = source.status;
    target.infinite_square_distance = source.infinite_square_distance * scale * scale;
    for extremum in &source.extrema {
        let transformed: Point3 = extremum.point.transformed(trsf);
        target.extrema.push(ExtremumResult::new(
            extremum.parameter,
            transformed,
            point.square_distance(&transformed),
            extremum.is_minimum,
            extremum.is_maximum,
        ));
    }
}
