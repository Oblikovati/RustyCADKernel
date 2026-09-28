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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_BSplineCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_BSplineCurve`: a B-spline curve on a grid, with a span cache for
//! runs of parameters in the same span.
use super::spans::{FlatKnotsView, SpanStep, SpanWalker};
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use crate::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::{
    constant_grid, empty, result_array, try_map_grid,
};
use rustycad_tkmath::bsplclib::{self, Cache, CurveSpan};
use rustycad_tkmath::gp::{Point3, Vector3};

/// Runs of at least this many parameters in one span go through the cache
/// (`THE_CACHE_THRESHOLD`; 4 in the 2D evaluator).
const CACHE_THRESHOLD: i32 = 2;

/// Batch evaluator of a `Geom_BSplineCurve` (`GeomGridEval_BSplineCurve`).
///
/// Each parameter is located in the flat knots; a run of at least two parameters in one
/// span goes through a `BSplCLib_Cache` built for that span, a single parameter through
/// `BSplCLib::D0`..`D3` directly. For a non-periodic curve the loop assumes ascending
/// parameters (see `spans`). `EvaluateGridDN` calls `BSplCLib::DN` with the knots and
/// multiplicities but passes the span index found in the flat knots: `PrepareEval`
/// keeps an index that lies in the knot index range, so for such indices the
/// derivative is the polynomial of another span (quirk kept). When that index is the
/// last knot index of a non-periodic curve, the native `BuildKnots` reads past the knot
/// and multiplicity arrays (undefined); the port raises Standard_OutOfRange there. The
/// evaluation representation of the curve is ignored;
/// `GeomGridEvalBSplineCurve::new(bspline).evaluate_grid(&params)?`.
pub struct GeomGridEvalBSplineCurve {
    geom: Handle<GeomBSplineCurve>,
}

// The curve arrays of `extractCurveData`, copied once per call.
struct CurveData {
    flat_knots: Array1<'static, f64>,
    poles: Array1<'static, Point3>,
    weights: Option<Array1<'static, f64>>,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    degree: i32,
    periodic: bool,
}

impl CurveData {
    fn read(curve: &GeomBSplineCurve) -> Self {
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
    fn view(&self) -> FlatKnotsView<'_> {
        FlatKnotsView {
            flat_knots: &self.flat_knots,
            degree: self.degree,
            periodic: self.periodic,
        }
    }
    // The span of the direct `BSplCLib::D0`..`D3` calls (flat knots, no multiplicities).
    fn flat_span(&self, u: f64, index: i32) -> CurveSpan<'_, 'static> {
        self.span(u, index, &self.flat_knots, None)
    }
    // The span of the `BSplCLib::DN` call: knots and multiplicities with the flat-knot
    // index (the native mix).
    fn knot_span(&self, u: f64, index: i32) -> CurveSpan<'_, 'static> {
        self.span(u, index, &self.knots, Some(&self.mults))
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
    fn new_cache(&self) -> Result<Cache, Failure> {
        let weights = self.weights.as_ref();
        Cache::new(
            self.degree,
            self.periodic,
            &self.flat_knots,
            &self.poles,
            weights,
        )
    }
}

// The cache state of the grid loop (`aSpanStart`, `aSpanLen`, `aUseCache`).
struct CurveRun {
    start: f64,
    length: f64,
    use_cache: bool,
}

impl CurveRun {
    // A new span: count its run, read its bounds (`computeSpanCoeffs`) and rebuild the
    // cache at the adjusted parameter when the run is long enough.
    fn enter(
        &mut self,
        data: &CurveData,
        cache: &mut Cache,
        position: (&[f64], usize),
        step: SpanStep,
    ) -> Result<(), Failure> {
        let view: FlatKnotsView<'_> = data.view();
        let size: i32 = view.count_span_size(position.0, position.1, step.span)?;
        self.start = view.knot(step.span)?;
        self.length = view.knot(step.span + 1)? - self.start;
        self.use_cache = size >= CACHE_THRESHOLD;
        if self.use_cache {
            let weights = data.weights.as_ref();
            cache.build_cache(step.adjusted, &data.flat_knots, &data.poles, weights)?;
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

// `evaluateGridCached`: every parameter through the span cache or the direct call.
fn evaluate_cached<T: Default>(
    data: &CurveData,
    params: &Array1<'_, f64>,
    cache_eval: impl Fn(&Cache, f64) -> Result<T, Failure>,
    direct_eval: impl Fn(&CurveData, f64, i32) -> Result<T, Failure>,
) -> Result<Array1<'static, T>, Failure> {
    let values: &[f64] = params.as_slice();
    let mut results: Array1<'static, T> = result_array(values.len());
    let mut cache: Cache = data.new_cache()?;
    let mut walker = SpanWalker::new(data.view());
    let mut run = CurveRun {
        start: 0.0,
        length: 1.0,
        use_cache: false,
    };
    for (i, slot) in results.iter_mut().enumerate() {
        let step: SpanStep = walker.step(values[i])?;
        if step.changed {
            run.enter(data, &mut cache, (values, i), step)?;
        }
        *slot = if run.use_cache {
            cache_eval(&cache, run.local(step.adjusted))?
        } else {
            direct_eval(data, step.adjusted, step.span)?
        };
    }
    Ok(results)
}

impl GeomGridEvalBSplineCurve {
    /// Evaluator of `bspline` (`GeomGridEval_BSplineCurve(theCurve)`); a null handle
    /// is accepted; `GeomGridEvalBSplineCurve::new(bspline)`.
    pub fn new(bspline: Handle<GeomBSplineCurve>) -> Self {
        Self { geom: bspline }
    }
    /// The evaluated curve (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomBSplineCurve> {
        &self.geom
    }
    // `extractCurveData` after the empty-parameter test.
    fn data(&self, params: &Array1<'_, f64>) -> Option<CurveData> {
        if params.is_empty() {
            return None;
        }
        Some(CurveData::read(self.geom.get()?))
    }
    /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point3>, Failure> {
        let Some(data) = self.data(params) else {
            return Ok(empty());
        };
        evaluate_cached(
            &data,
            params,
            |cache, local| cache.d0_local::<Point3>(local),
            |d, u, span| bsplclib::d0(&d.flat_span(u, span), &d.poles, d.weights.as_ref()),
        )
    }
    /// Points and D1 (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD1>, Failure> {
        let Some(data) = self.data(params) else {
            return Ok(empty());
        };
        let pack = |(point, d1)| GeomGridEvalCurveD1 { point, d1 };
        evaluate_cached(
            &data,
            params,
            |cache, local| cache.d1_local::<Point3>(local).map(pack),
            |d, u, span| {
                bsplclib::d1(&d.flat_span(u, span), &d.poles, d.weights.as_ref()).map(pack)
            },
        )
    }
    /// Points, D1 and D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD2>, Failure> {
        let Some(data) = self.data(params) else {
            return Ok(empty());
        };
        let pack = |(point, d1, d2)| GeomGridEvalCurveD2 { point, d1, d2 };
        evaluate_cached(
            &data,
            params,
            |cache, local| cache.d2_local::<Point3>(local).map(pack),
            |d, u, span| {
                bsplclib::d2(&d.flat_span(u, span), &d.poles, d.weights.as_ref()).map(pack)
            },
        )
    }
    /// Points, D1, D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD3>, Failure> {
        let Some(data) = self.data(params) else {
            return Ok(empty());
        };
        let pack = |(point, d1, d2, d3)| GeomGridEvalCurveD3 { point, d1, d2, d3 };
        evaluate_cached(
            &data,
            params,
            |cache, local| cache.d3_local::<Point3>(local).map(pack),
            |d, u, span| {
                bsplclib::d3(&d.flat_span(u, span), &d.poles, d.weights.as_ref()).map(pack)
            },
        )
    }
    /// Derivative of order `n` through `BSplCLib::DN` (every parameter located again),
    /// zero above the degree; empty for `n < 1` (`EvaluateGridDN`);
    /// `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector3>, Failure> {
        if params.is_empty() || n < 1 {
            return Ok(empty());
        }
        let Some(data) = self.data(params) else {
            return Ok(empty());
        };
        if n > data.degree {
            return Ok(constant_grid(params.len(), Vector3::new(0.0, 0.0, 0.0)));
        }
        try_map_grid(params, |u| {
            let (adjusted, span) = data.view().locate_span(u)?;
            bsplclib::dn(
                &data.knot_span(adjusted, span),
                n,
                &data.poles,
                data.weights.as_ref(),
            )
        })
    }
}
