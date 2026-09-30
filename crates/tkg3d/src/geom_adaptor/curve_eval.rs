// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Curve.cxx (EvalD0..EvalDN, IsBoundary, RebuildCache) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation of `GeomAdaptor_Curve`.
//!
//! Elementary curves go through ElCLib on the recorded gp geometry. Bezier and
//! B-spline curves go through the span cache, except at the adaptor bounds of a
//! B-spline, which use the local evaluators on the boundary span, and except when
//! the curve had an evaluation representation at load time, which the curve then
//! evaluates itself. Offset curves combine the nested basis adaptor with
//! Geom_OffsetCurveUtils.
use super::curve::GeomAdaptorCurve;
use super::curve_eval_order::{EvalD0, EvalD1, EvalD2, EvalD3, EvalOrder};
use super::curve_state::{AdaptorState, BSplineData, BezierData, CurveData};
use crate::adaptor3d::Adaptor3dCurve;
use crate::geom::errors::undefined_derivative;
use crate::geom::offset_curve_utils::evaluate_dn;
use crate::geom::{GeomBSplineCurve, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::span_cache::{self, BOUNDARY_LOCATE_TOLERANCE, CacheSource};
use rustycad_tkmath::bsplclib::Cache;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// `EvalD0`; `d0(&mut state, 0.5)?`.
pub(super) fn d0(state: &mut AdaptorState, u: f64) -> Result<Point3, Failure> {
    evaluate::<EvalD0>(state, u)
}
/// `EvalD1`; `d1(&mut state, 0.5)?`.
pub(super) fn d1(state: &mut AdaptorState, u: f64) -> Result<GeomCurveD1, Failure> {
    evaluate::<EvalD1>(state, u)
}
/// `EvalD2`; `d2(&mut state, 0.5)?`.
pub(super) fn d2(state: &mut AdaptorState, u: f64) -> Result<GeomCurveD2, Failure> {
    evaluate::<EvalD2>(state, u)
}
/// `EvalD3`; `d3(&mut state, 0.5)?`.
pub(super) fn d3(state: &mut AdaptorState, u: f64) -> Result<GeomCurveD3, Failure> {
    evaluate::<EvalD3>(state, u)
}

// The shared switch of EvalD0..EvalD3 on the curve kind.
fn evaluate<O: EvalOrder>(state: &mut AdaptorState, u: f64) -> Result<O::Output, Failure> {
    if let Some(result) = O::elementary(&state.data, u) {
        return Ok(result);
    }
    if state.data.has_eval_rep() {
        return O::curve(state.curve()?, u);
    }
    let (first, last) = (state.first, state.last);
    match &mut state.data {
        CurveData::Bezier(bezier) => O::cached(bezier_cache(bezier, u)?, u),
        CurveData::BSpline(bspline) => bspline_eval::<O>(bspline, u, first, last),
        CurveData::Offset(offset) => {
            let basis: &dyn Adaptor3dCurve = &*offset.basis_adaptor;
            O::offset(u, basis, offset.direction, offset.offset)?.ok_or_else(|| O::failure())
        }
        _ => O::curve(state.curve()?, u),
    }
}

fn bspline_eval<O: EvalOrder>(
    data: &mut BSplineData,
    u: f64,
    first: f64,
    last: f64,
) -> Result<O::Output, Failure> {
    if let Some((start, finish)) = boundary_span(&data.curve, u, first, last)? {
        return O::local(&data.curve, u, start, finish);
    }
    let valid: bool = data
        .cache
        .as_ref()
        .is_some_and(|cache| cache.is_cache_valid(u));
    if !valid {
        rebuild_bspline_cache(data, u)?;
    }
    O::cached(data.cache.as_ref().expect("cache built above"), u)
}

/// `IsBoundary` on the knots of `curve`; `boundary_span(&c, 0.0, 0.0, 1.0)?`.
pub(super) fn boundary_span(
    curve: &GeomBSplineCurve,
    u: f64,
    first: f64,
    last: f64,
) -> Result<Option<(i32, i32)>, Failure> {
    let locate = |bound: f64| curve.locate_u(bound, BOUNDARY_LOCATE_TOLERANCE, false);
    span_cache::boundary_span(u, (first, last), || curve.nb_knots(), locate)
}

// RebuildCache for a B-spline: create the cache once, then fill it on the span of `u`.
fn rebuild_bspline_cache(data: &mut BSplineData, u: f64) -> Result<(), Failure> {
    let curve: &GeomBSplineCurve = &data.curve;
    let (flat, poles, weights) = (curve.knot_sequence(), curve.poles(), curve.weights());
    let source = CacheSource {
        flat: &flat,
        poles: &poles,
        weights: weights.as_ref(),
    };
    let (degree, periodic) = (|| curve.degree(), || curve.is_periodic());
    span_cache::rebuild_cache(&mut data.cache, &source, degree, periodic, u)
}

// RebuildCache for a Bezier curve, which runs only while no cache exists: its single
// span is valid for every parameter.
fn bezier_cache(data: &mut BezierData, u: f64) -> Result<&Cache, Failure> {
    if data.cache.is_none() {
        let degree: i32 = data.curve.degree();
        let (poles, weights) = (data.curve.poles(), data.curve.weights());
        let periodic: bool = data.curve.is_periodic();
        let cache = span_cache::bezier_span_cache(degree, periodic, &poles, weights.as_ref(), u)?;
        data.cache = Some(cache);
    }
    Ok(data.cache.as_ref().expect("cache built above"))
}

/// `EvalDN`: ElCLib for elementary curves, the curve itself for Bezier curves, the
/// boundary span or the curve for B-splines, Geom_OffsetCurveUtils for offsets;
/// `dn(&state, 0.5, 2)?`.
pub(super) fn dn(state: &AdaptorState, u: f64, n: i32) -> Result<Vector3, Failure> {
    match &state.data {
        CurveData::Line(line) => Ok(elclib::dn(u, line, n)),
        CurveData::Circle(circle) => Ok(elclib::dn(u, circle, n)),
        CurveData::Ellipse(ellipse) => Ok(elclib::dn(u, ellipse, n)),
        CurveData::Hyperbola(hyperbola) => Ok(elclib::dn(u, hyperbola, n)),
        CurveData::Parabola(parabola) => Ok(elclib::dn(u, parabola, n)),
        CurveData::BSpline(bspline) if !bspline.eval_rep.is_some() => {
            match boundary_span(&bspline.curve, u, state.first, state.last)? {
                Some((start, finish)) => bspline.curve.local_dn(u, start, finish, n),
                None => state.curve()?.eval_dn(u, n),
            }
        }
        CurveData::Offset(offset) if !offset.eval_rep.is_some() => offset_dn(
            &offset.basis_adaptor,
            offset.direction,
            offset.offset,
            (u, n),
        ),
        _ => state.curve()?.eval_dn(u, n),
    }
}

fn offset_dn(
    basis: &GeomAdaptorCurve,
    direction: Direction3,
    offset: f64,
    (u, n): (f64, i32),
) -> Result<Vector3, Failure> {
    let basis: &dyn Adaptor3dCurve = basis;
    evaluate_dn(u, basis, direction, offset, n)?
        .ok_or_else(|| undefined_derivative("GeomAdaptor_Curve::EvalDN: evaluation failed"))
}
