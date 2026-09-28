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
// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_BezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_BezierCurve`: a Bezier curve on a grid through one span cache.
use super::grid::{constant_grid, empty, try_map_grid};
use super::{Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3};
use crate::geom2d::Geom2dBezierCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplclib::{self, Cache, CurveSpan};
use rustycad_tkmath::gp::{Point2, Vector2};

/// Batch evaluator of a `Geom2d_BezierCurve` (`Geom2dGridEval_BezierCurve`).
///
/// `EvaluateGrid` to `EvaluateGridD3` build a `BSplCLib_Cache` of the single span at
/// `0.5` and evaluate every parameter through it; `EvaluateGridDN` calls
/// `BSplCLib::DN` on the flat knots. Quirks kept: the evaluation representation of the
/// curve is ignored, and orders above the degree give zero vectors even for a rational
/// curve, whose derivatives never vanish there;
/// `Geom2dGridEvalBezierCurve::new(bezier).evaluate_grid(&params)?`.
pub struct Geom2dGridEvalBezierCurve {
    geom: Handle<Geom2dBezierCurve>,
}

// The arrays that the native evaluator reads by reference (`Poles()`, `Weights()`,
// `KnotSequence()`); here they are copies taken once per call.
struct BezierArrays {
    degree: i32,
    poles: Array1<'static, Point2>,
    weights: Option<Array1<'static, f64>>,
    flat_knots: Array1<'static, f64>,
}

impl BezierArrays {
    fn read(bezier: &Geom2dBezierCurve) -> Result<Self, Failure> {
        let flat_knots: Array1<'static, f64> = bezier.knot_sequence()?;
        Ok(Self {
            degree: bezier.degree(),
            poles: bezier.poles(),
            weights: bezier.weights(),
            flat_knots,
        })
    }
    // `CreateBezierCache2d`: the cache of the single span, built at 0.5.
    fn cache(&self) -> Result<Cache, Failure> {
        let weights: Option<&Array1<'static, f64>> = self.weights.as_ref();
        let mut cache: Cache =
            Cache::new(self.degree, false, &self.flat_knots, &self.poles, weights)?;
        cache.build_cache(0.5, &self.flat_knots, &self.poles, weights)?;
        Ok(cache)
    }
    // `BSplCLib::DN(U, N, 0, Degree, false, Poles, Weights, KnotSequence, nullptr)`.
    fn dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        let span = CurveSpan {
            u,
            index: 0,
            degree: self.degree,
            periodic: false,
            knots: &self.flat_knots,
            mults: None,
        };
        bsplclib::dn(&span, n, &self.poles, self.weights.as_ref())
    }
}

impl Geom2dGridEvalBezierCurve {
    /// Evaluator of `bezier` (`Geom2dGridEval_BezierCurve(theBezier)`); a null handle is
    /// accepted; `Geom2dGridEvalBezierCurve::new(bezier)`.
    pub fn new(bezier: Handle<Geom2dBezierCurve>) -> Self {
        Self { geom: bezier }
    }
    /// The evaluated curve (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<Geom2dBezierCurve> {
        &self.geom
    }
    // The span cache when there is something to evaluate.
    fn cache(&self, params: &Array1<'_, f64>) -> Result<Option<Cache>, Failure> {
        let Some(bezier) = self.geom.get() else {
            return Ok(None);
        };
        if params.is_empty() {
            return Ok(None);
        }
        Ok(Some(BezierArrays::read(bezier)?.cache()?))
    }
    /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point2>, Failure> {
        let Some(cache) = self.cache(params)? else {
            return Ok(empty());
        };
        try_map_grid(params, |u| cache.d0::<Point2>(u))
    }
    /// Points and D1 (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD1>, Failure> {
        let Some(cache) = self.cache(params)? else {
            return Ok(empty());
        };
        try_map_grid(params, |u| {
            let (point, d1) = cache.d1::<Point2>(u)?;
            Ok(Geom2dGridEvalCurveD1 { point, d1 })
        })
    }
    /// Points, D1 and D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD2>, Failure> {
        let Some(cache) = self.cache(params)? else {
            return Ok(empty());
        };
        try_map_grid(params, |u| {
            let (point, d1, d2) = cache.d2::<Point2>(u)?;
            Ok(Geom2dGridEvalCurveD2 { point, d1, d2 })
        })
    }
    /// Points, D1, D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD3>, Failure> {
        let Some(cache) = self.cache(params)? else {
            return Ok(empty());
        };
        try_map_grid(params, |u| {
            let (point, d1, d2, d3) = cache.d3::<Point2>(u)?;
            Ok(Geom2dGridEvalCurveD3 { point, d1, d2, d3 })
        })
    }
    /// Derivative of order `n` through `BSplCLib::DN`, zero above the degree; empty for
    /// `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector2>, Failure> {
        let Some(bezier) = self.geom.get() else {
            return Ok(empty());
        };
        if params.is_empty() || n < 1 {
            return Ok(empty());
        }
        if n > bezier.degree() {
            return Ok(constant_grid(params.len(), Vector2::new(0.0, 0.0)));
        }
        let arrays: BezierArrays = BezierArrays::read(bezier)?;
        try_map_grid(params, |u| arrays.dn(u, n))
    }
}
