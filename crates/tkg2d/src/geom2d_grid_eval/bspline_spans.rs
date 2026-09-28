// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of the anonymous namespace of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_BSplineCurve.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Span location and the cached grid loop of the B-spline grid evaluator
//! (`CurveData`, `locateSpan`, `countSpanSize`, `computeSpanCoeffs`,
//! `evaluateGridCached`).
use super::grid::result_array;
use crate::geom2d::arrays::at;
use crate::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{self, Cache, CurveSpan};
use rustycad_tkmath::gp::Point2;

/// Runs shorter than this many parameters in one span are evaluated directly
/// (`THE_CACHE_THRESHOLD`).
const CACHE_THRESHOLD: i32 = 4;

/// The curve arrays of `extractCurveData`, copied once per call.
pub(super) struct SplineArrays {
    pub(super) flat_knots: Array1<'static, f64>,
    pub(super) poles: Array1<'static, Point2>,
    weights: Option<Array1<'static, f64>>,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    pub(super) degree: i32,
    periodic: bool,
}

impl SplineArrays {
    /// Reads the flat knots, poles, weights, knots, multiplicities, degree and
    /// periodicity, in the native order.
    pub(super) fn read(curve: &Geom2dBSplineCurve) -> Self {
        Self {
            flat_knots: curve.knot_sequence(),
            poles: curve.poles(),
            weights: curve.weights(),
            knots: curve.knots(),
            mults: curve.multiplicities(),
            degree: curve.degree(),
            periodic: curve.is_periodic(),
        }
    }
    /// The weights of a rational curve (`Weights()`, null otherwise).
    pub(super) fn weights(&self) -> Option<&Array1<'static, f64>> {
        self.weights.as_ref()
    }
    /// `locateSpan`: `BSplCLib::LocateParameter` on the flat knots with `NoMults()`,
    /// from index 0; returns the adjusted parameter and the flat-knot span index.
    pub(super) fn locate_span(&self, u: f64) -> Result<(f64, i32), Failure> {
        let (mut adjusted, mut span): (f64, i32) = (u, 0);
        bsplclib::locate_parameter_for_eval(
            self.degree,
            &self.flat_knots,
            None,
            u,
            self.periodic,
            &mut span,
            &mut adjusted,
        )?;
        Ok((adjusted, span))
    }
    /// The flat-knot span of the direct `BSplCLib::D0`..`D3` calls.
    pub(super) fn flat_span(&self, u: f64, span: i32) -> CurveSpan<'_, 'static> {
        self.span(u, span, &self.flat_knots, None)
    }
    /// The span of the `BSplCLib::DN` call: knots and multiplicities with the
    /// flat-knot index (the native mix).
    pub(super) fn knot_span(&self, u: f64, span: i32) -> CurveSpan<'_, 'static> {
        self.span(u, span, &self.knots, Some(&self.mults))
    }
    fn span<'r>(
        &'r self,
        u: f64,
        index: i32,
        knots: &'r Array1<'static, f64>,
        mults: Option<&'r Array1<'static, i32>>,
    ) -> CurveSpan<'r, 'static> {
        CurveSpan {
            u,
            index,
            degree: self.degree,
            periodic: self.periodic,
            knots,
            mults,
        }
    }
    // `countSpanSize`: the parameters from `start` on that stay in `target`.
    fn count_span_size(&self, params: &[f64], start: usize, target: i32) -> Result<i32, Failure> {
        let mut count: i32 = 1;
        for &u in &params[start + 1..] {
            if self.locate_span(u)?.1 != target {
                break;
            }
            count += 1;
        }
        Ok(count)
    }
}

// The span state of the grid loop (`aPrevSpan`, `aSpanStart`, `aSpanLen`, `aUseCache`).
struct SpanRun {
    prev_span: i32,
    start: f64,
    length: f64,
    use_cache: bool,
}

impl SpanRun {
    // Updates the run when `span` differs from the previous one, rebuilding the cache
    // at the adjusted parameter when the run is long enough.
    fn enter(
        &mut self,
        data: &SplineArrays,
        cache: &mut Cache,
        position: (&[f64], usize),
        span: i32,
        adjusted: f64,
    ) -> Result<(), Failure> {
        if span == self.prev_span {
            return Ok(());
        }
        self.prev_span = span;
        let size: i32 = data.count_span_size(position.0, position.1, span)?;
        self.start = at(&data.flat_knots, span)?;
        self.length = at(&data.flat_knots, span + 1)? - self.start;
        self.use_cache = size >= CACHE_THRESHOLD;
        if self.use_cache {
            cache.build_cache(adjusted, &data.flat_knots, &data.poles, data.weights())?;
        }
        Ok(())
    }
    // The curve cache works on the [0, 1] local parameter of the span.
    fn local(&self, adjusted: f64) -> f64 {
        if self.length > 0.0 {
            (adjusted - self.start) / self.length
        } else {
            0.0
        }
    }
}

/// `evaluateGridCached`: every parameter through the span cache or the direct call.
pub(super) fn evaluate_cached<T: Default>(
    data: &SplineArrays,
    params: &Array1<'_, f64>,
    cache_eval: impl Fn(&Cache, f64) -> Result<T, Failure>,
    direct_eval: impl Fn(&SplineArrays, f64, i32) -> Result<T, Failure>,
) -> Result<Array1<'static, T>, Failure> {
    let values: &[f64] = params.as_slice();
    let mut results: Array1<'static, T> = result_array(values.len());
    let mut cache: Cache = Cache::new(
        data.degree,
        data.periodic,
        &data.flat_knots,
        &data.poles,
        data.weights(),
    )?;
    let mut run = SpanRun {
        prev_span: -1,
        start: 0.0,
        length: 1.0,
        use_cache: false,
    };
    for (i, slot) in results.iter_mut().enumerate() {
        let (adjusted, span) = data.locate_span(values[i])?;
        run.enter(data, &mut cache, (values, i), span, adjusted)?;
        *slot = if run.use_cache {
            cache_eval(&cache, run.local(adjusted))?
        } else {
            direct_eval(data, adjusted, span)?
        };
    }
    Ok(results)
}
