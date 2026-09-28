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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_OffsetSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_OffsetSurface`: an offset surface on a grid from the basis grid and
//! `Geom_OffsetSurfaceUtils`.
use super::surface::GeomGridEvalSurface;
use super::surface_grid::{any_empty, empty_grid, invalid_orders, try_map_indexed};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::offset_surface_utils::{
    OsculatingQuery, evaluate_d0_from, evaluate_d1_from, evaluate_d2_from, evaluate_d3_outcome,
    evaluate_dn_from,
};
use crate::geom::{GeomOffsetSurface, GeomSurface};
use crate::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point3, Vector3};

/// Batch evaluator of a `Geom_OffsetSurface` (`GeomGridEval_OffsetSurface`).
///
/// The basis and offset are read at construction. When the offset surface has an
/// equivalent surface (`Surface()`, a parallel plane or a quadric with another radius)
/// every member evaluates that surface through `GeomGridEval_Surface`. Otherwise
/// `EvaluateGrid`, `D1`, `D2` and `DN` evaluate the basis grid one order higher (D1
/// for DN) and pass each record to `Geom_OffsetSurfaceUtils` with a
/// `GeomAdaptor_Surface` on the basis and the offset surface as osculating query;
/// `EvaluateGridD3` calls `EvaluateD3` at every cell. Quirk kept: the success flag of
/// the utilities is ignored, so a cell where no normal is found holds the basis values
/// of the last point tried (the zero vector for DN); an empty basis grid (an eval
/// surface basis) gives an empty grid;
/// `GeomGridEvalOffsetSurface::new(offset).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalOffsetSurface {
    geom: Handle<GeomOffsetSurface>,
    basis: Handle<dyn GeomSurface>,
    offset: f64,
}

// The per-cell context of the non-equivalent path: the basis adaptor and the query.
struct OffsetContext<'a> {
    adaptor: GeomAdaptorSurface,
    query: Option<&'a dyn OsculatingQuery>,
    offset: f64,
}

impl GeomGridEvalOffsetSurface {
    /// Evaluator of `offset_surface`, reading its basis and distance now
    /// (`GeomGridEval_OffsetSurface(theOffset)`); a null handle gives empty grids;
    /// `GeomGridEvalOffsetSurface::new(offset_surface)`.
    pub fn new(offset_surface: Handle<GeomOffsetSurface>) -> Self {
        let (basis, offset) = match offset_surface.get() {
            Some(surface) => (surface.basis_surface(), surface.offset()),
            None => (Handle::default(), 0.0),
        };
        Self {
            geom: offset_surface,
            basis,
            offset,
        }
    }
    /// The evaluated surface (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomOffsetSurface> {
        &self.geom
    }
    // Whether there is nothing to evaluate (`myBasis.IsNull()` or empty parameters).
    fn is_void(&self, u: &Array1<'_, f64>, v: &Array1<'_, f64>) -> bool {
        self.basis.is_null() || any_empty(u, v)
    }
    // The fast path: the equivalent surface when it exists and is not the offset
    // surface itself.
    fn equivalent(&self) -> Result<Option<GeomGridEvalSurface<'static>>, Failure> {
        let Some(offset_surface) = self.geom.get() else {
            return Ok(None);
        };
        let equivalent: Handle<dyn GeomSurface> = offset_surface.surface()?;
        if equivalent.is_null() || equivalent == self.geom {
            return Ok(None);
        }
        Ok(Some(GeomGridEvalSurface::from_surface(&equivalent)))
    }
    fn basis_eval(&self) -> GeomGridEvalSurface<'static> {
        GeomGridEvalSurface::from_surface(&self.basis)
    }
    // `GeomAdaptor_Surface aBasisAdaptor(myBasis)` and the osculating query.
    fn context(&self) -> Result<OffsetContext<'_>, Failure> {
        let query: Option<&dyn OsculatingQuery> =
            self.geom.get().map(|s| s as &dyn OsculatingQuery);
        Ok(OffsetContext {
            adaptor: GeomAdaptorSurface::with_surface(&self.basis)?,
            query,
            offset: self.offset,
        })
    }
    /// Offset points from the basis D1 grid (`EvaluateGrid`); `evaluator.evaluate_grid(&u, &v)?`.
    pub fn evaluate_grid(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, Point3>, Failure> {
        if self.is_void(u, v) {
            return Ok(empty_grid());
        }
        if let Some(equivalent) = self.equivalent()? {
            return equivalent.evaluate_grid(u, v);
        }
        let basis: Array2<'static, GeomGridEvalSurfD1> =
            self.basis_eval().evaluate_grid_d1(u, v)?;
        if basis.is_empty() {
            return Ok(empty_grid());
        }
        let c: OffsetContext<'_> = self.context()?;
        try_map_indexed(u, v, |(i, u), (j, v)| {
            let first: GeomGridEvalSurfD1 = *cell(&basis, i, j)?;
            Ok(evaluate_d0_from((u, v), &c.adaptor, c.offset, c.query, first)?.value)
        })
    }
    /// Offset points and first derivatives from the basis D2 grid (`EvaluateGridD1`);
    /// `evaluator.evaluate_grid_d1(&u, &v)?`.
    pub fn evaluate_grid_d1(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD1>, Failure> {
        if self.is_void(u, v) {
            return Ok(empty_grid());
        }
        if let Some(equivalent) = self.equivalent()? {
            return equivalent.evaluate_grid_d1(u, v);
        }
        let basis: Array2<'static, GeomGridEvalSurfD2> =
            self.basis_eval().evaluate_grid_d2(u, v)?;
        if basis.is_empty() {
            return Ok(empty_grid());
        }
        let c: OffsetContext<'_> = self.context()?;
        try_map_indexed(u, v, |(i, u), (j, v)| {
            let b: GeomGridEvalSurfD2 = *cell(&basis, i, j)?;
            let first = GeomGridEvalSurfD1 {
                point: b.point,
                d1u: b.d1u,
                d1v: b.d1v,
            };
            let start = (first, [b.d2u, b.d2v, b.d2uv]);
            Ok(evaluate_d1_from((u, v), &c.adaptor, c.offset, c.query, start)?.value)
        })
    }
    /// Offset points and derivatives up to order 2 from the basis D3 grid
    /// (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&u, &v)?`.
    pub fn evaluate_grid_d2(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD2>, Failure> {
        if self.is_void(u, v) {
            return Ok(empty_grid());
        }
        if let Some(equivalent) = self.equivalent()? {
            return equivalent.evaluate_grid_d2(u, v);
        }
        let basis: Array2<'static, GeomGridEvalSurfD3> =
            self.basis_eval().evaluate_grid_d3(u, v)?;
        if basis.is_empty() {
            return Ok(empty_grid());
        }
        let c: OffsetContext<'_> = self.context()?;
        try_map_indexed(u, v, |(i, u), (j, v)| {
            let first: GeomGridEvalSurfD3 = *cell(&basis, i, j)?;
            Ok(evaluate_d2_from((u, v), &c.adaptor, c.offset, c.query, first)?.value)
        })
    }
    /// Offset points and derivatives up to order 3 through `EvaluateD3` at every cell
    /// (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&u, &v)?`.
    pub fn evaluate_grid_d3(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD3>, Failure> {
        if self.is_void(u, v) {
            return Ok(empty_grid());
        }
        if let Some(equivalent) = self.equivalent()? {
            return equivalent.evaluate_grid_d3(u, v);
        }
        let c: OffsetContext<'_> = self.context()?;
        try_map_indexed(u, v, |(_, u), (_, v)| {
            Ok(evaluate_d3_outcome((u, v), &c.adaptor, c.offset, c.query)?.value)
        })
    }
    /// Offset derivative `(nu, nv)` from the basis D1 grid; empty for negative orders or
    /// `nu + nv < 1` (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&u, &v, 1, 1)?`.
    pub fn evaluate_grid_dn(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
        nu: i32,
        nv: i32,
    ) -> Result<Array2<'static, Vector3>, Failure> {
        if self.is_void(u, v) || invalid_orders(nu, nv) {
            return Ok(empty_grid());
        }
        if let Some(equivalent) = self.equivalent()? {
            return equivalent.evaluate_grid_dn(u, v, nu, nv);
        }
        let basis: Array2<'static, GeomGridEvalSurfD1> =
            self.basis_eval().evaluate_grid_d1(u, v)?;
        if basis.is_empty() {
            return Ok(empty_grid());
        }
        let c: OffsetContext<'_> = self.context()?;
        try_map_indexed(u, v, |(i, u), (j, v)| {
            let b: GeomGridEvalSurfD1 = *cell(&basis, i, j)?;
            let first = (b.d1u, b.d1v);
            let found = evaluate_dn_from((u, v), (nu, nv), &c.adaptor, c.offset, c.query, first)?;
            // A failed EvaluateDN leaves the default `gp_Vec aDN` untouched.
            Ok(found.unwrap_or_default())
        })
    }
}

// The basis record of cell `(i, j)` (zero-based), range checked as `Value(i, j)`.
fn cell<'g, T>(grid: &'g Array2<'_, T>, i: usize, j: usize) -> Result<&'g T, Failure> {
    grid.at(i, j).map_err(|error| {
        Failure::out_of_range(&format!(
            "NCollection_Array2::Value({}, {}): {error}; expected a cell of the basis grid",
            i + 1,
            j + 1
        ))
    })
}
