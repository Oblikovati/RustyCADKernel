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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_SurfaceOfRevolution.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_SurfaceOfRevolution`: the basis curve grid on V, rotated about the
//! axis for every U value.
use super::curve::GeomGridEvalCurve;
use super::surface_grid::{any_empty, empty_grid, invalid_orders, try_map_indexed};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::revolution_utils::{
    calculate_d0, calculate_d1, calculate_d2, calculate_d3, calculate_dn,
};
use crate::geom::{GeomCurve, GeomSurfaceOfRevolution, GeomSweptSurface};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkmath::gp::{Axis1, Point3, Vector3};

/// Batch evaluator of a `Geom_SurfaceOfRevolution` (`GeomGridEval_SurfaceOfRevolution`).
///
/// The basis curve and axis are read at construction. Each order evaluates the basis
/// grid on the V values once through `GeomGridEval_Curve` (an empty curve grid, as for
/// an eval curve, gives an empty surface grid) and applies `Geom_RevolutionUtils` at
/// every cell. Quirk kept: `EvaluateGridDN` reads the curve grid without testing it,
/// so an empty one raises Standard_OutOfRange;
/// `GeomGridEvalSurfaceOfRevolution::new(revolution).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalSurfaceOfRevolution {
    geom: Handle<GeomSurfaceOfRevolution>,
    basis_curve: Handle<dyn GeomCurve>,
    axis: Axis1,
}

// Every cell from the curve record of its V index: `cell(record_j, u_i)`; an empty
// curve grid gives an empty surface grid.
fn by_column<R: Copy, T: Default>(
    records: &Array1<'_, R>,
    u: &Array1<'_, f64>,
    v: &Array1<'_, f64>,
    cell: impl Fn(R, f64) -> Result<T, Failure>,
) -> Result<Array2<'static, T>, Failure> {
    if records.is_empty() {
        return Ok(empty_grid());
    }
    let values: &[R] = records.as_slice();
    try_map_indexed(u, v, |(_, u), (j, _)| cell(values[j], u))
}

impl GeomGridEvalSurfaceOfRevolution {
    /// Evaluator of `revolution`, reading its basis curve and axis now
    /// (`GeomGridEval_SurfaceOfRevolution(theRevolution)`); a null handle gives empty
    /// grids; `GeomGridEvalSurfaceOfRevolution::new(revolution)`.
    pub fn new(revolution: Handle<GeomSurfaceOfRevolution>) -> Self {
        let (basis_curve, axis) = match revolution.get() {
            Some(surface) => (surface.basis_curve(), surface.axis()),
            None => (Handle::default(), Axis1::default()),
        };
        Self {
            geom: revolution,
            basis_curve,
            axis,
        }
    }
    /// The evaluated surface (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomSurfaceOfRevolution> {
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
    /// Points at every `(u, v)` (`EvaluateGrid`); `evaluator.evaluate_grid(&u, &v)?`.
    pub fn evaluate_grid(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, Point3>, Failure> {
        let Some(curve) = self.curve_eval(u, v) else {
            return Ok(empty_grid());
        };
        let grid = curve.evaluate_grid(v)?;
        by_column(&grid, u, v, |p, u| calculate_d0(p, u, self.axis))
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
        let grid = curve.evaluate_grid_d1(v)?;
        by_column(&grid, u, v, |c, u| {
            calculate_d1(c.point, c.d1, u, self.axis)
        })
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
        let grid = curve.evaluate_grid_d2(v)?;
        by_column(&grid, u, v, |c, u| {
            calculate_d2(c.point, [c.d1, c.d2], u, self.axis)
        })
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
        let grid = curve.evaluate_grid_d3(v)?;
        by_column(&grid, u, v, |c, u| {
            calculate_d3(c.point, [c.d1, c.d2, c.d3], u, self.axis)
        })
    }
    /// Derivative `(nu, nv)`: `CalculateDN` of the curve DN, or of `P - AxisLocation`
    /// for `nu > 0, nv == 0`; empty for negative orders or `nu + nv < 1`
    /// (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&u, &v, 1, 1)?`.
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
        let axis: Axis1 = self.axis;
        if nu != 0 && nv == 0 {
            let points: Array1<'static, Point3> = curve.evaluate_grid(v)?;
            let location = axis.location().xyz();
            return try_map_indexed(u, v, |(_, u), (j, _)| {
                // `gp_Vec(aCurvePts.Value(j).XYZ() - myAxisLocation.XYZ())`.
                let base = Vector3::from(at(&points, j as i32 + 1)?.xyz() - location);
                calculate_dn(base, u, axis, nu, nv)
            });
        }
        let curve_dn: Array1<'static, Vector3> = curve.evaluate_grid_dn(v, nv)?;
        try_map_indexed(u, v, |(_, u), (j, _)| {
            calculate_dn(at(&curve_dn, j as i32 + 1)?, u, axis, nu, nv)
        })
    }
}
