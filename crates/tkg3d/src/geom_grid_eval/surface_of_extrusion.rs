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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_SurfaceOfExtrusion.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_SurfaceOfExtrusion`: `S(u, v) = C(u) + v * Direction` from the basis
//! curve grid on U.
use super::curve::GeomGridEvalCurve;
use super::surface_grid::{
    any_empty, constant_surface_grid, empty_grid, invalid_orders, try_map_indexed,
};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::extrusion_utils::{
    calculate_d0, calculate_d1, calculate_d2, calculate_d3, calculate_dn,
};
use crate::geom::{GeomCurve, GeomSurfaceOfLinearExtrusion, GeomSweptSurface};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3, Xyz};

/// Batch evaluator of a `Geom_SurfaceOfLinearExtrusion` (`GeomGridEval_SurfaceOfExtrusion`).
///
/// The basis curve and direction are read at construction. Each order evaluates the
/// basis grid on the U values once through `GeomGridEval_Curve` (an empty curve grid,
/// as for an eval curve, gives an empty surface grid) and applies
/// `Geom_ExtrusionUtils` at every cell. Quirk kept: `EvaluateGridDN` with `nv == 0`
/// reads the curve DN grid without testing it, so an empty one raises
/// Standard_OutOfRange; `GeomGridEvalSurfaceOfExtrusion::new(extrusion).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalSurfaceOfExtrusion {
    geom: Handle<GeomSurfaceOfLinearExtrusion>,
    basis_curve: Handle<dyn GeomCurve>,
    direction: Direction3,
}

// Every cell from the curve record of its U index: `cell(record_i, v_j)`; an empty
// curve grid gives an empty surface grid.
fn by_row<R: Copy, T: Default>(
    records: &Array1<'_, R>,
    u: &Array1<'_, f64>,
    v: &Array1<'_, f64>,
    cell: impl Fn(R, f64) -> T,
) -> Result<Array2<'static, T>, Failure> {
    if records.is_empty() {
        return Ok(empty_grid());
    }
    let values: &[R] = records.as_slice();
    try_map_indexed(u, v, |(i, _), (_, v)| Ok(cell(values[i], v)))
}

impl GeomGridEvalSurfaceOfExtrusion {
    /// Evaluator of `extrusion`, reading its basis curve and direction now
    /// (`GeomGridEval_SurfaceOfExtrusion(theExtrusion)`); a null handle gives empty
    /// grids; `GeomGridEvalSurfaceOfExtrusion::new(extrusion)`.
    pub fn new(extrusion: Handle<GeomSurfaceOfLinearExtrusion>) -> Self {
        let (basis_curve, direction) = match extrusion.get() {
            Some(surface) => (surface.basis_curve(), surface.direction()),
            None => (Handle::default(), Direction3::default()),
        };
        Self {
            geom: extrusion,
            basis_curve,
            direction,
        }
    }
    /// The evaluated surface (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomSurfaceOfLinearExtrusion> {
        &self.geom
    }
    // The basis curve evaluator when there is something to evaluate.
    fn curve_eval(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Option<GeomGridEvalCurve<'static>> {
        if self.basis_curve.is_null() || any_empty(u, v) {
            return None;
        }
        Some(GeomGridEvalCurve::from_curve(&self.basis_curve))
    }
    fn dir(&self) -> Xyz {
        self.direction.xyz()
    }
    /// Points at every `(u, v)` (`EvaluateGrid`); `evaluator.evaluate_grid(&u, &v)?`.
    pub fn evaluate_grid(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, Point3>, Failure> {
        let Some(curve) = self.curve_eval(u, v) else {
            return Ok(empty_grid());
        };
        let grid = curve.evaluate_grid(u)?;
        by_row(&grid, u, v, |p, v| calculate_d0(p, v, self.dir()))
    }
    /// Points and first derivatives (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&u, &v)?`.
    pub fn evaluate_grid_d1(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD1>, Failure> {
        let Some(curve) = self.curve_eval(u, v) else {
            return Ok(empty_grid());
        };
        let grid = curve.evaluate_grid_d1(u)?;
        by_row(&grid, u, v, |c, v| calculate_d1(&c, v, self.dir()))
    }
    /// Points and derivatives up to order 2 (`EvaluateGridD2`);
    /// `evaluator.evaluate_grid_d2(&u, &v)?`.
    pub fn evaluate_grid_d2(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD2>, Failure> {
        let Some(curve) = self.curve_eval(u, v) else {
            return Ok(empty_grid());
        };
        let grid = curve.evaluate_grid_d2(u)?;
        by_row(&grid, u, v, |c, v| calculate_d2(&c, v, self.dir()))
    }
    /// Points and derivatives up to order 3 (`EvaluateGridD3`);
    /// `evaluator.evaluate_grid_d3(&u, &v)?`.
    pub fn evaluate_grid_d3(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD3>, Failure> {
        let Some(curve) = self.curve_eval(u, v) else {
            return Ok(empty_grid());
        };
        let grid = curve.evaluate_grid_d3(u)?;
        by_row(&grid, u, v, |c, v| calculate_d3(&c, v, self.dir()))
    }
    /// Derivative `(nu, nv)`: the curve DN for `nv == 0`, the direction for `(0, 1)`,
    /// zero otherwise; empty for negative orders or `nu + nv < 1` (`EvaluateGridDN`);
    /// `evaluator.evaluate_grid_dn(&u, &v, 0, 1)?`.
    pub fn evaluate_grid_dn(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
        nu: i32,
        nv: i32,
    ) -> Result<Array2<'static, Vector3>, Failure> {
        let valid: Option<GeomGridEvalCurve<'static>> = self.curve_eval(u, v);
        let Some(curve) = valid.filter(|_| !invalid_orders(nu, nv)) else {
            return Ok(empty_grid());
        };
        let dir: Xyz = self.dir();
        if nv != 0 {
            let constant: Vector3 = calculate_dn(Vector3::default(), dir, nu, nv);
            return Ok(constant_surface_grid(u.len(), v.len(), constant));
        }
        let curve_dn: Array1<'static, Vector3> = curve.evaluate_grid_dn(u, nu)?;
        try_map_indexed(u, v, |(i, _), _| {
            Ok(calculate_dn(at(&curve_dn, i as i32 + 1)?, dir, nu, nv))
        })
    }
}
