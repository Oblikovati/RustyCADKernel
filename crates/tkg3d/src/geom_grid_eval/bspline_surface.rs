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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_BSplineSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_BSplineSurface`: a B-spline surface on a grid, with a patch cache for
//! runs of cells in the same span pair.
use super::bezier_surface::surf_d3;
use super::bspline_surface_loops::{GridCell, SurfaceData, evaluate_cached, evaluate_direct};
use super::surface_grid::{any_empty, constant_surface_grid, empty_grid, invalid_orders};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::GeomBSplineSurface;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplslib::{self, SurfaceD2};
use rustycad_tkmath::gp::{Point3, Vector3};

/// Batch evaluator of a `Geom_BSplineSurface` (`GeomGridEval_BSplineSurface`).
///
/// Each U and V value is located in the flat knots (a non-periodic direction assumes
/// ascending values, see `spans`); `EvaluateGrid` to `EvaluateGridD2` rebuild a
/// `BSplSLib_Cache` whenever a row enters a new V span and the U run times the V run
/// covers at least two cells, and call `BSplSLib::D0`..`D2` directly otherwise;
/// `EvaluateGridD3` and `EvaluateGridDN` always call `BSplSLib::D3`/`DN`. Orders above
/// a degree give zero vectors even for a rational surface (quirk kept); the evaluation
/// representation of the surface is ignored;
/// `GeomGridEvalBSplineSurface::new(bspline).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalBSplineSurface {
    geom: Handle<GeomBSplineSurface>,
}

fn surf_d2(r: SurfaceD2) -> GeomGridEvalSurfD2 {
    GeomGridEvalSurfD2 {
        point: r.p,
        d1u: r.vu,
        d1v: r.vv,
        d2u: r.vuu,
        d2v: r.vvv,
        d2uv: r.vuv,
    }
}

impl GeomGridEvalBSplineSurface {
    /// Evaluator of `bspline` (`GeomGridEval_BSplineSurface(theSurface)`); a null handle
    /// is accepted; `GeomGridEvalBSplineSurface::new(bspline)`.
    pub fn new(bspline: Handle<GeomBSplineSurface>) -> Self {
        Self { geom: bspline }
    }
    /// The evaluated surface (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomBSplineSurface> {
        &self.geom
    }
    // `extractSurfaceData` after the empty-parameter test.
    fn data(&self, u: &Array1<'_, f64>, v: &Array1<'_, f64>) -> Option<SurfaceData> {
        if any_empty(u, v) {
            return None;
        }
        Some(SurfaceData::read(self.geom.get()?))
    }
    /// Points at every `(u, v)` (`EvaluateGrid`); `evaluator.evaluate_grid(&u, &v)?`.
    pub fn evaluate_grid(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, Point3>, Failure> {
        let Some(data) = self.data(u, v) else {
            return Ok(empty_grid());
        };
        evaluate_cached(
            &data,
            (u, v),
            |cache, lu, lv| cache.d0_local(lu, lv),
            |d, c: GridCell| bsplslib::d0(&d.span(c.uv, c.spans), &d.poles, d.weights.as_ref()),
        )
    }
    /// Points and first derivatives (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&u, &v)?`.
    pub fn evaluate_grid_d1(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD1>, Failure> {
        let Some(data) = self.data(u, v) else {
            return Ok(empty_grid());
        };
        let pack = |(point, d1u, d1v)| GeomGridEvalSurfD1 { point, d1u, d1v };
        evaluate_cached(
            &data,
            (u, v),
            |cache, lu, lv| cache.d1_local(lu, lv).map(pack),
            |d, c: GridCell| {
                bsplslib::d1(&d.span(c.uv, c.spans), &d.poles, d.weights.as_ref()).map(pack)
            },
        )
    }
    /// Points and derivatives up to order 2 (`EvaluateGridD2`);
    /// `evaluator.evaluate_grid_d2(&u, &v)?`.
    pub fn evaluate_grid_d2(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD2>, Failure> {
        let Some(data) = self.data(u, v) else {
            return Ok(empty_grid());
        };
        evaluate_cached(
            &data,
            (u, v),
            |cache, lu, lv| cache.d2_local(lu, lv).map(surf_d2),
            |d, c: GridCell| {
                bsplslib::d2(&d.span(c.uv, c.spans), &d.poles, d.weights.as_ref()).map(surf_d2)
            },
        )
    }
    /// Points and derivatives up to order 3 through `BSplSLib::D3` (`EvaluateGridD3`);
    /// `evaluator.evaluate_grid_d3(&u, &v)?`.
    pub fn evaluate_grid_d3(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD3>, Failure> {
        let Some(data) = self.data(u, v) else {
            return Ok(empty_grid());
        };
        evaluate_direct(&data, (u, v), |d, c: GridCell| {
            let r = bsplslib::d3(&d.span(c.uv, c.spans), &d.poles, d.weights.as_ref())?;
            Ok(surf_d3(&r))
        })
    }
    /// Derivative `(nu, nv)` through `BSplSLib::DN`, zero when an order exceeds its
    /// degree; empty for negative orders or `nu + nv < 1` (`EvaluateGridDN`);
    /// `evaluator.evaluate_grid_dn(&u, &v, 1, 1)?`.
    pub fn evaluate_grid_dn(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
        nu: i32,
        nv: i32,
    ) -> Result<Array2<'static, Vector3>, Failure> {
        if invalid_orders(nu, nv) {
            return Ok(empty_grid());
        }
        let Some(data) = self.data(u, v) else {
            return Ok(empty_grid());
        };
        if nu > data.u_degree || nv > data.v_degree {
            return Ok(constant_surface_grid(
                u.len(),
                v.len(),
                Vector3::new(0.0, 0.0, 0.0),
            ));
        }
        evaluate_direct(&data, (u, v), |d, c: GridCell| {
            bsplslib::dn(&d.span(c.uv, c.spans), nu, nv, &d.poles, d.weights.as_ref())
        })
    }
}
