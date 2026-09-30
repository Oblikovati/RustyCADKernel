// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_ApproxArcsSegments.cxx
// (checkContinuity, getParameter, isInflectionPoint) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The file-static functions of Geom2dConvert_ApproxArcsSegments.cxx.
use super::{MAXPOINTS, MY_CURVATURE_TOLERANCE};
use crate::geom2d_convert::ppoint::PPoint;
use crate::native_failure::failure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::gp::{Point2, Vector2, Xy};
use std::f64::consts::PI;

/// `checkContinuity`: whether the tangent at the end of `curve1` is parallel to the
/// tangent at the start of `curve2` within `angle_tol` (either sense).
pub(super) fn check_continuity(
    curve1: &Handle<dyn Geom2dCurve>,
    curve2: &Handle<dyn Geom2dCurve>,
    angle_tol: f64,
) -> Result<bool, Failure> {
    let (curve1, curve2) = (live(curve1)?, live(curve2)?);
    let (_, v11): (Point2, Vector2) = curve1.d1(curve1.last_parameter())?;
    let (_, v21): (Point2, Vector2) = curve2.d1(curve2.first_parameter())?;
    // Check continuity with the specified tolerance.
    v11.is_parallel(v21, angle_tol)
}

// The native code dereferences the element; a null one (appended by a failed
// makeApproximation) is never followed by this test.
fn live(curve: &Handle<dyn Geom2dCurve>) -> Result<&dyn Geom2dCurve, Failure> {
    curve.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "Geom2dConvert_ApproxArcsSegments: null element; expected a curve",
        )
    })
}

/// `getParameter`: the curve point nearest `xy1` between two parameters, by halving
/// the interval towards the nearer end (at most MAXPOINTS times).
pub(super) fn get_parameter(
    xy1: Xy,
    first_param: f64,
    last_param: f64,
    curve: &dyn Adaptor2dCurve2d,
) -> Result<PPoint, Failure> {
    let (mut prev_param, mut af1, mut af2): (f64, f64, f64) =
        (first_param, first_param, last_param);
    // For finding the nearest point use the method of half division.
    let mut min_dist: f64 = f64::MAX;
    for _ in 1..=MAXPOINTS {
        let result = PPoint::from_adaptor(af1, curve)?;
        let adist1: f64 = (xy1 - result.point()).modulus();
        if adist1 < precision::confusion() {
            return Ok(result);
        }
        let result = PPoint::from_adaptor(af2, curve)?;
        let adist2: f64 = (xy1 - result.point()).modulus();
        if adist2 < precision::confusion() {
            return Ok(result);
        }
        if min_dist <= adist2 - precision::confusion()
            && min_dist <= adist1 - precision::confusion()
        {
            break;
        }
        if adist1 < adist2 - precision::confusion() {
            (prev_param, min_dist) = (af1, adist1);
            af2 = (af1 + af2) * 0.5;
        } else {
            (prev_param, min_dist) = (af2, adist2);
            af1 = (af1 + af2) * 0.5;
        }
    }
    PPoint::from_adaptor(prev_param, curve)
}

/// `isInflectionPoint(theParam, theCurve)`: whether the curvature at `param` is
/// below MyCurvatureTolerance.
pub(super) fn is_inflection_point(
    param: f64,
    curve: &dyn Adaptor2dCurve2d,
) -> Result<bool, Failure> {
    let (_, d1, d2): (Point2, Vector2, Vector2) = curve.d2(param)?;
    Ok(curvature(d1, d2) < MY_CURVATURE_TOLERANCE)
}

/// `isInflectionPoint(theParam, theFirstInfl, theCurve, theAngleTol)`: a small
/// curvature at `param`, where the tangent is also along the chord from
/// `first_infl` within `angle_tol` (either sense). The chord must not be null
/// (gp_VectorWithNullMagnitude).
pub(super) fn is_inflection_point_from(
    param: f64,
    first_infl: &PPoint,
    curve: &dyn Adaptor2dCurve2d,
    angle_tol: f64,
) -> Result<bool, Failure> {
    let (p1, d1, d2): (Point2, Vector2, Vector2) = curve.d2(param)?;
    let curv: f64 = curvature(d1, d2);
    let mut cont_angle: f64 = Vector2::from(p1.xy() - first_infl.point()).angle(d1)?.abs();
    cont_angle = std_min(cont_angle, (PI - cont_angle).abs());
    Ok(curv < MY_CURVATURE_TOLERANCE && cont_angle < angle_tol)
}

// |D1 ^ D2| / |D1|^3.
fn curvature(d1: Vector2, d2: Vector2) -> f64 {
    let sq_mod: f64 = d1.xy().square_modulus();
    (d1.xy() ^ d2.xy()).abs() / (sq_mod * sq_mod.sqrt())
}
