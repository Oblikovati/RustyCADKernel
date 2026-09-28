// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKG2d/Geom2dAdaptor/Geom2dAdaptor_Curve.cxx (EvalD0..EvalDN, IsBoundary, RebuildCache) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation of `Geom2dAdaptor_Curve`.
//!
//! Elementary curves go through ElCLib on the recorded gp geometry. Bezier and
//! B-spline curves go through the span cache, except at the adaptor bounds of a
//! B-spline, which use the local evaluators on the boundary span, and except when
//! the curve had an evaluation representation at load time, which the curve then
//! evaluates itself. Offset curves combine the nested basis adaptor with
//! Geom2d_OffsetCurveUtils.
use super::curve::Geom2dAdaptorCurve;
use super::curve_eval_order::{EvalD0, EvalD1, EvalD2, EvalD3, EvalOrder};
use super::curve_state::{AdaptorState, BSplineData, BezierData, CurveData};
use crate::geom2d::errors::undefined_derivative;
use crate::geom2d::offset_curve_utils::evaluate_dn;
use crate::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{Cache, flat_bezier_knots};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Point2, Vector2};

/// `EvalD0`; `d0(&mut state, 0.5)?`.
pub(super) fn d0(state: &mut AdaptorState, u: f64) -> Result<Point2, Failure> {
    evaluate::<EvalD0>(state, u)
}
/// `EvalD1`; `d1(&mut state, 0.5)?`.
pub(super) fn d1(state: &mut AdaptorState, u: f64) -> Result<Geom2dCurveD1, Failure> {
    evaluate::<EvalD1>(state, u)
}
/// `EvalD2`; `d2(&mut state, 0.5)?`.
pub(super) fn d2(state: &mut AdaptorState, u: f64) -> Result<Geom2dCurveD2, Failure> {
    evaluate::<EvalD2>(state, u)
}
/// `EvalD3`; `d3(&mut state, 0.5)?`.
pub(super) fn d3(state: &mut AdaptorState, u: f64) -> Result<Geom2dCurveD3, Failure> {
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
            O::offset(u, &offset.basis_adaptor, offset.offset)?.ok_or_else(|| O::failure())
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

// Half the parametric confusion: the LocateU tolerance of the boundary spans.
const POS_TOL: f64 = precision::p_confusion() / 2.0;

/// `IsBoundary`: the knot span that `u` ends when it equals an adaptor bound, widened
/// to one span inside `1..=NbKnots`; `None` elsewhere; `boundary_span(&c, 0.0, 0.0, 1.0)?`.
pub(super) fn boundary_span(
    curve: &Geom2dBSplineCurve,
    u: f64,
    first: f64,
    last: f64,
) -> Result<Option<(i32, i32)>, Failure> {
    if u == first {
        let (mut start, mut finish) = curve.locate_u(first, POS_TOL, false)?;
        start = start.max(1);
        if start >= finish {
            finish = start + 1;
        }
        return Ok(Some((start, finish)));
    }
    if u == last {
        let (mut start, mut finish) = curve.locate_u(last, POS_TOL, false)?;
        finish = finish.min(curve.nb_knots());
        if start >= finish {
            start = finish - 1;
        }
        return Ok(Some((start, finish)));
    }
    Ok(None)
}

// RebuildCache for a B-spline: create the cache once, then fill it on the span of `u`.
fn rebuild_bspline_cache(data: &mut BSplineData, u: f64) -> Result<(), Failure> {
    let curve: &Geom2dBSplineCurve = &data.curve;
    let (flat, poles, weights) = (curve.knot_sequence(), curve.poles(), curve.weights());
    if data.cache.is_none() {
        let periodic: bool = curve.is_periodic();
        data.cache = Some(Cache::new(
            curve.degree(),
            periodic,
            &flat,
            &poles,
            weights.as_ref(),
        )?);
    }
    let cache: &mut Cache = data.cache.as_mut().expect("cache created above");
    cache.build_cache(u, &flat, &poles, weights.as_ref())
}

// RebuildCache for a Bezier curve, which runs only while no cache exists: its single
// span is valid for every parameter.
fn bezier_cache(data: &mut BezierData, u: f64) -> Result<&Cache, Failure> {
    if data.cache.is_none() {
        let degree: i32 = data.curve.degree();
        let knots: &'static [f64] = flat_bezier_knots(degree)?;
        let flat = Array1::from_shared(&knots[..2 * (degree as usize + 1)], 1);
        let (poles, weights) = (data.curve.poles(), data.curve.weights());
        let mut cache = Cache::new(
            degree,
            data.curve.is_periodic(),
            &flat,
            &poles,
            weights.as_ref(),
        )?;
        cache.build_cache(u, &flat, &poles, weights.as_ref())?;
        data.cache = Some(cache);
    }
    Ok(data.cache.as_ref().expect("cache built above"))
}

/// `EvalDN`: ElCLib for elementary curves, the curve itself for Bezier curves, the
/// boundary span or the curve for B-splines, Geom2d_OffsetCurveUtils for offsets;
/// `dn(&state, 0.5, 2)?`.
pub(super) fn dn(state: &AdaptorState, u: f64, n: i32) -> Result<Vector2, Failure> {
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
        CurveData::Offset(offset) if !offset.eval_rep.is_some() => {
            offset_dn(&offset.basis_adaptor, offset.offset, u, n)
        }
        _ => state.curve()?.eval_dn(u, n),
    }
}

fn offset_dn(basis: &Geom2dAdaptorCurve, offset: f64, u: f64, n: i32) -> Result<Vector2, Failure> {
    evaluate_dn(u, basis, offset, n)?
        .ok_or_else(|| undefined_derivative("Geom2dAdaptor_Curve::EvalDN: evaluation failed"))
}
