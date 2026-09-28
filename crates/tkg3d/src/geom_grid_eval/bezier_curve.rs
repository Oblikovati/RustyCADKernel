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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_BezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_BezierCurve`: a Bezier curve on a grid through one span cache.
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use crate::geom::GeomBezierCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_grid_eval::bezier_arrays::BezierCurveArrays;
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::{constant_grid, empty, try_map_grid};
use rustycad_tkmath::bsplclib::Cache;
use rustycad_tkmath::gp::{Point3, Vector3};

/// Batch evaluator of a `Geom_BezierCurve` (`GeomGridEval_BezierCurve`).
///
/// `EvaluateGrid` to `EvaluateGridD3` build a `BSplCLib_Cache` of the single span at
/// `0.5` and evaluate every parameter through it; `EvaluateGridDN` calls
/// `BSplCLib::DN` on the flat knots. Quirks kept: the evaluation representation of the
/// curve is ignored, and orders above the degree give zero vectors even for a rational
/// curve, whose derivatives never vanish there;
/// `GeomGridEvalBezierCurve::new(bezier).evaluate_grid(&params)?`.
pub struct GeomGridEvalBezierCurve {
    geom: Handle<GeomBezierCurve>,
}

// The arrays of the curve, read once per call.
fn read_arrays(bezier: &GeomBezierCurve) -> Result<BezierCurveArrays<Point3>, Failure> {
    let flat_knots: Array1<'static, f64> = bezier.knot_sequence()?;
    Ok(BezierCurveArrays {
        degree: bezier.degree(),
        poles: bezier.poles(),
        weights: bezier.weights(),
        flat_knots,
    })
}

impl GeomGridEvalBezierCurve {
    /// Evaluator of `bezier` (`GeomGridEval_BezierCurve(theBezier)`); a null handle is
    /// accepted; `GeomGridEvalBezierCurve::new(bezier)`.
    pub fn new(bezier: Handle<GeomBezierCurve>) -> Self {
        Self { geom: bezier }
    }
    /// The evaluated curve (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomBezierCurve> {
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
        Ok(Some(read_arrays(bezier)?.cache()?))
    }
    /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point3>, Failure> {
        let Some(cache) = self.cache(params)? else {
            return Ok(empty());
        };
        try_map_grid(params, |u| cache.d0::<Point3>(u))
    }
    /// Points and D1 (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD1>, Failure> {
        let Some(cache) = self.cache(params)? else {
            return Ok(empty());
        };
        try_map_grid(params, |u| {
            let (point, d1) = cache.d1::<Point3>(u)?;
            Ok(GeomGridEvalCurveD1 { point, d1 })
        })
    }
    /// Points, D1 and D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD2>, Failure> {
        let Some(cache) = self.cache(params)? else {
            return Ok(empty());
        };
        try_map_grid(params, |u| {
            let (point, d1, d2) = cache.d2::<Point3>(u)?;
            Ok(GeomGridEvalCurveD2 { point, d1, d2 })
        })
    }
    /// Points, D1, D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD3>, Failure> {
        let Some(cache) = self.cache(params)? else {
            return Ok(empty());
        };
        try_map_grid(params, |u| {
            let (point, d1, d2, d3) = cache.d3::<Point3>(u)?;
            Ok(GeomGridEvalCurveD3 { point, d1, d2, d3 })
        })
    }
    /// Derivative of order `n` through `BSplCLib::DN`, zero above the degree; empty for
    /// `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector3>, Failure> {
        let Some(bezier) = self.geom.get() else {
            return Ok(empty());
        };
        if params.is_empty() || n < 1 {
            return Ok(empty());
        }
        if n > bezier.degree() {
            return Ok(constant_grid(params.len(), Vector3::new(0.0, 0.0, 0.0)));
        }
        let arrays: BezierCurveArrays<Point3> = read_arrays(bezier)?;
        try_map_grid(params, |u| arrays.dn(u, n))
    }
}
