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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_BezierSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_BezierSurface`: a Bezier surface on a grid through one span cache.
use super::curve::GeomGridEvalCurve;
use super::surface_grid::{
    any_empty, constant_surface_grid, empty_grid, grid_array, invalid_orders, try_map_rows,
};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::{GeomBezierSurface, GeomCurve, GeomSurface};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkmath::bsplslib::{self, Cache, SurfaceSpan};
use rustycad_tkmath::gp::{Point3, Vector3};

/// Minimum number of U values of an `Nx1` grid that goes through the V isoline
/// (`THE_ISOLINE_THRESHOLD`).
const ISOLINE_THRESHOLD: usize = 8;

/// Batch evaluator of a `Geom_BezierSurface` (`GeomGridEval_BezierSurface`).
///
/// `EvaluateGrid` of an `Nx1` grid with at least 8 U values evaluates the `VIso` curve
/// through `GeomGridEval_Curve`; any failure there falls back to the surface cache, as
/// the native `try`/`catch`. The other `EvaluateGrid` calls and `EvaluateGridD1`/`D2`
/// build a `BSplSLib_Cache` of the single patch at `(0.5, 0.5)`; `EvaluateGridD3` and
/// `EvaluateGridDN` call `BSplSLib::D3`/`DN` on the flat knots. Quirks kept: the
/// evaluation representation of the surface is ignored, and orders above a degree give
/// zero vectors even for a rational surface;
/// `GeomGridEvalBezierSurface::new(bezier).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalBezierSurface {
    geom: Handle<GeomBezierSurface>,
}

// The arrays that the native evaluator reads by reference, copied once per call.
struct BezierSurfaceArrays {
    u_degree: i32,
    v_degree: i32,
    u_flat: Array1<'static, f64>,
    v_flat: Array1<'static, f64>,
    poles: Array2<'static, Point3>,
    weights: Option<Array2<'static, f64>>,
}

impl BezierSurfaceArrays {
    fn read(bezier: &GeomBezierSurface) -> Result<Self, Failure> {
        Ok(Self {
            u_degree: bezier.u_degree(),
            v_degree: bezier.v_degree(),
            u_flat: bezier.u_knot_sequence()?,
            v_flat: bezier.v_knot_sequence()?,
            poles: bezier.poles(),
            weights: bezier.weights(),
        })
    }
    // `buildBezierCache`: the cache of the single patch, built at (0.5, 0.5).
    fn cache(&self) -> Result<Cache, Failure> {
        let weights = self.weights.as_ref();
        let (u_flat, v_flat) = (&self.u_flat, &self.v_flat);
        let mut cache: Cache = Cache::new(
            self.u_degree,
            false,
            u_flat,
            self.v_degree,
            false,
            v_flat,
            weights,
        )?;
        cache.build_cache(0.5, 0.5, u_flat, v_flat, &self.poles, weights)?;
        Ok(cache)
    }
    // The single span (index 0 on flat knots, non-periodic, rational when weighted).
    fn span(&self, u: f64, v: f64) -> SurfaceSpan<'_, 'static> {
        let rational: bool = self.weights.is_some();
        SurfaceSpan {
            u,
            v,
            u_index: 0,
            v_index: 0,
            u_knots: &self.u_flat,
            v_knots: &self.v_flat,
            u_mults: None,
            v_mults: None,
            u_degree: self.u_degree,
            v_degree: self.v_degree,
            u_rat: rational,
            v_rat: rational,
            u_per: false,
            v_per: false,
        }
    }
}

// The cell loop of every member: U outer, V inner, the first failure aborts.
fn cells<T: Default>(
    u: &Array1<'_, f64>,
    v: &Array1<'_, f64>,
    eval: impl Fn(f64, f64) -> Result<T, Failure>,
) -> Result<Array2<'static, T>, Failure> {
    try_map_rows(u, v, Ok, |&u, v| eval(u, v))
}

impl GeomGridEvalBezierSurface {
    /// Evaluator of `bezier` (`GeomGridEval_BezierSurface(theBezier)`); a null handle is
    /// accepted; `GeomGridEvalBezierSurface::new(bezier)`.
    pub fn new(bezier: Handle<GeomBezierSurface>) -> Self {
        Self { geom: bezier }
    }
    /// The evaluated surface (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomBezierSurface> {
        &self.geom
    }
    // The surface when there is something to evaluate.
    fn loaded(&self, u: &Array1<'_, f64>, v: &Array1<'_, f64>) -> Option<&GeomBezierSurface> {
        let bezier: &GeomBezierSurface = self.geom.get()?;
        (!any_empty(u, v)).then_some(bezier)
    }
    /// Points at every `(u, v)` (`EvaluateGrid`); `evaluator.evaluate_grid(&u, &v)?`.
    pub fn evaluate_grid(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, Point3>, Failure> {
        let Some(bezier) = self.loaded(u, v) else {
            return Ok(empty_grid());
        };
        if v.len() == 1 && u.len() >= ISOLINE_THRESHOLD {
            // Any failure of the isoline path is caught natively and falls through.
            if let Ok(Some(grid)) = v_iso_grid(bezier, u, v) {
                return Ok(grid);
            }
        }
        let cache: Cache = BezierSurfaceArrays::read(bezier)?.cache()?;
        cells(u, v, |u, v| cache.d0(u, v))
    }
    /// Points and first derivatives through the cache (`EvaluateGridD1`);
    /// `evaluator.evaluate_grid_d1(&u, &v)?`.
    pub fn evaluate_grid_d1(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD1>, Failure> {
        let Some(bezier) = self.loaded(u, v) else {
            return Ok(empty_grid());
        };
        let cache: Cache = BezierSurfaceArrays::read(bezier)?.cache()?;
        cells(u, v, |u, v| {
            let (point, d1u, d1v) = cache.d1(u, v)?;
            Ok(GeomGridEvalSurfD1 { point, d1u, d1v })
        })
    }
    /// Points and derivatives up to order 2 through the cache (`EvaluateGridD2`);
    /// `evaluator.evaluate_grid_d2(&u, &v)?`.
    pub fn evaluate_grid_d2(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD2>, Failure> {
        let Some(bezier) = self.loaded(u, v) else {
            return Ok(empty_grid());
        };
        let cache: Cache = BezierSurfaceArrays::read(bezier)?.cache()?;
        cells(u, v, |u, v| {
            let r = cache.d2(u, v)?;
            Ok(GeomGridEvalSurfD2 {
                point: r.p,
                d1u: r.vu,
                d1v: r.vv,
                d2u: r.vuu,
                d2v: r.vvv,
                d2uv: r.vuv,
            })
        })
    }
    /// Points and derivatives up to order 3 through `BSplSLib::D3` (`EvaluateGridD3`);
    /// `evaluator.evaluate_grid_d3(&u, &v)?`.
    pub fn evaluate_grid_d3(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD3>, Failure> {
        let Some(bezier) = self.loaded(u, v) else {
            return Ok(empty_grid());
        };
        let arrays: BezierSurfaceArrays = BezierSurfaceArrays::read(bezier)?;
        let weights = arrays.weights.as_ref();
        cells(u, v, |u, v| {
            let r = bsplslib::d3(&arrays.span(u, v), &arrays.poles, weights)?;
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
        let Some(bezier) = self.loaded(u, v).filter(|_| !invalid_orders(nu, nv)) else {
            return Ok(empty_grid());
        };
        if nu > bezier.u_degree() || nv > bezier.v_degree() {
            return Ok(constant_surface_grid(
                u.len(),
                v.len(),
                Vector3::new(0.0, 0.0, 0.0),
            ));
        }
        let arrays: BezierSurfaceArrays = BezierSurfaceArrays::read(bezier)?;
        let weights = arrays.weights.as_ref();
        cells(u, v, |u, v| {
            bsplslib::dn(&arrays.span(u, v), nu, nv, &arrays.poles, weights)
        })
    }
}

/// The `BSplSLib::D3` outputs as a grid record.
pub(super) fn surf_d3(r: &bsplslib::SurfaceD3) -> GeomGridEvalSurfD3 {
    GeomGridEvalSurfD3 {
        point: r.p,
        d1u: r.vu,
        d1v: r.vv,
        d2u: r.vuu,
        d2v: r.vvv,
        d2uv: r.vuv,
        d3u: r.vuuu,
        d3v: r.vvvv,
        d3uuv: r.vuuv,
        d3uvv: r.vuvv,
    }
}

// The `Nx1` isoline path: the `VIso` curve of the single V value on the U grid,
// reshaped to `NbU x 1`; `None` for a null isoline curve.
fn v_iso_grid(
    bezier: &GeomBezierSurface,
    u: &Array1<'_, f64>,
    v: &Array1<'_, f64>,
) -> Result<Option<Array2<'static, Point3>>, Failure> {
    let curve: Handle<dyn GeomCurve> = bezier.v_iso(at(v, v.lower())?)?;
    if curve.is_null() {
        return Ok(None);
    }
    let points: Array1<'static, Point3> = GeomGridEvalCurve::from_curve(&curve).evaluate_grid(u)?;
    let mut grid: Array2<'static, Point3> = grid_array(u.len(), 1);
    for (k, slot) in (1_i32..).zip(grid.iter_mut()) {
        *slot = at(&points, k)?;
    }
    Ok(Some(grid))
}
