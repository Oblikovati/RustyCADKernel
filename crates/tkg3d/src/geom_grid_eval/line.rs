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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Line.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Line`: `P(t) = Location + t * Direction` on a parameter grid.
use super::conic::{
    ConicTerms, conic_grid, conic_grid_d1, conic_grid_d2, conic_grid_d3, conic_grid_dn,
};
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use crate::geom::GeomLine;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::empty;
use rustycad_tkmath::gp::{Point3, Vector3};

/// Batch evaluator of a `Geom_Line` (`GeomGridEval_Line`).
///
/// A null line or an empty parameter array gives an empty result; results are indexed
/// from 1 whatever the bounds of the parameters; the line cannot raise;
/// `GeomGridEvalLine::new(line).evaluate_grid(&params)`.
pub struct GeomGridEvalLine {
    geom: Handle<GeomLine>,
}

// The pre-extracted location and direction coordinates (`aLocX`...`aDirZ`).
struct LineCoords {
    loc: [f64; 3],
    dir: [f64; 3],
}

impl ConicTerms for LineCoords {
    type Local = f64;
    fn local(&self, t: f64) -> f64 {
        t
    }
    // `aLocX + t * aDirX` (and Y, Z).
    fn point(&self, t: f64) -> Point3 {
        let (l, d) = (self.loc, self.dir);
        Point3::new(l[0] + t * d[0], l[1] + t * d[1], l[2] + t * d[2])
    }
    fn d1(&self, _t: f64) -> Vector3 {
        Vector3::new(self.dir[0], self.dir[1], self.dir[2])
    }
    fn d2(&self, _t: f64) -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }
    fn d3(&self, _t: f64) -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }
    // D1 is the direction, every higher derivative is zero.
    fn dn(&self, t: f64, n: i32) -> Vector3 {
        if n == 1 { self.d1(t) } else { self.d2(t) }
    }
}

impl GeomGridEvalLine {
    /// Evaluator of `line` (`GeomGridEval_Line(theLine)`); a null handle is accepted;
    /// `GeomGridEvalLine::new(line)`.
    pub fn new(line: Handle<GeomLine>) -> Self {
        Self { geom: line }
    }
    /// The evaluated line (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomLine> {
        &self.geom
    }
    // The coordinates of a loaded line when there is something to evaluate.
    fn terms(&self, params: &Array1<'_, f64>) -> Option<LineCoords> {
        let line: &GeomLine = self.geom.get()?;
        if params.is_empty() {
            return None;
        }
        let lin = line.lin();
        let (location, direction) = (lin.location(), lin.direction());
        Some(LineCoords {
            loc: [location.x(), location.y(), location.z()],
            dir: [direction.x(), direction.y(), direction.z()],
        })
    }
    /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)`.
    pub fn evaluate_grid(&self, params: &Array1<'_, f64>) -> Array1<'static, Point3> {
        self.terms(params)
            .map_or_else(empty, |t| conic_grid(&t, params))
    }
    /// Points and the constant first derivative (`EvaluateGridD1`);
    /// `evaluator.evaluate_grid_d1(&params)`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Array1<'static, GeomGridEvalCurveD1> {
        self.terms(params)
            .map_or_else(empty, |t| conic_grid_d1(&t, params))
    }
    /// Points, D1 and a zero D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Array1<'static, GeomGridEvalCurveD2> {
        self.terms(params)
            .map_or_else(empty, |t| conic_grid_d2(&t, params))
    }
    /// Points, D1 and zero D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Array1<'static, GeomGridEvalCurveD3> {
        self.terms(params)
            .map_or_else(empty, |t| conic_grid_d3(&t, params))
    }
    /// Derivative of order `n`: the direction for `n == 1`, zero above; empty for
    /// `n < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&params, 2)`.
    pub fn evaluate_grid_dn(&self, params: &Array1<'_, f64>, n: i32) -> Array1<'static, Vector3> {
        if n < 1 {
            return empty();
        }
        self.terms(params)
            .map_or_else(empty, |t| conic_grid_dn(&t, params, n))
    }
}
