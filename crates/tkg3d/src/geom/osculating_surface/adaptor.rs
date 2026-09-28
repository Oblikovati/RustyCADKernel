// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_OsculatingSurface.cxx (isQPunctual) and the GeomAdaptor_Surface D1 path it uses at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The first derivatives that `isQPunctual` reads through a `GeomAdaptor_Surface`
//! on a B-spline or Bezier surface: the Bezier and B-spline caches of the adaptor
//! and, at the bounds of a B-spline, its `LocalD1` with the knot span chosen by
//! `IfUVBound` and `Span`. Only this path of GeomAdaptor is ported here; the
//! GeomAdaptor package itself is a later TKG3d work item.
use crate::geom::errors;
use crate::geom::{GeomBSplineSurface, GeomBezierSurface, GeomSurface, GeomSurfaceD1, LocalKnots};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib::Cache;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `PosTol` of GeomAdaptor_Surface.cxx.
fn pos_tol() -> f64 {
    precision::p_confusion() * 0.5
}

/// A GeomAdaptor_Surface loaded with a B-spline or Bezier surface and bounds (the
/// tolerances are 0, as in the osculating code); its cache is built on demand.
pub(super) struct SplineAdaptor {
    surface: Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
    cache: Option<Cache>,
}

impl SplineAdaptor {
    /// `GeomAdaptor_Surface(S, U1, U2, V1, V2)` or `Load(S)` (the surface bounds).
    pub(super) fn new(
        surface: Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<Self, Failure> {
        let (u1, u2, v1, v2) = bounds;
        if u1 > u2 || v1 > v2 {
            return Err(errors::construction(&format!(
                "GeomAdaptor_Surface::Load: bounds [{u1}, {u2}] x [{v1}, {v2}]; expected First <= Last"
            )));
        }
        Ok(Self {
            surface,
            bounds,
            cache: None,
        })
    }

    /// `GeomAdaptor_Surface::EvalD1` for the Bezier and B-spline types.
    pub(super) fn eval_d1(&mut self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let bezier: Handle<GeomBezierSurface> = self.surface.cast();
        if let Some(bezier) = bezier.get() {
            if bezier.has_eval_representation() {
                return self.surface.eval_d1(u, v);
            }
            return self.bezier_d1(bezier, u, v);
        }
        let bspline: Handle<GeomBSplineSurface> = self.surface.cast();
        let Some(bspline) = bspline.get() else {
            return self.surface.eval_d1(u, v);
        };
        if bspline.has_eval_representation() {
            return self.surface.eval_d1(u, v);
        }
        self.bspline_d1(bspline, u, v)
    }

    fn bezier_d1(
        &mut self,
        bezier: &GeomBezierSurface,
        u: f64,
        v: f64,
    ) -> Result<GeomSurfaceD1, Failure> {
        if self.cache.is_none() {
            let (du, dv) = (bezier.u_degree(), bezier.v_degree());
            let flat_u = flat_bezier(du)?;
            let flat_v = flat_bezier(dv)?;
            let (poles, weights) = (bezier.poles(), bezier.weights());
            let mut cache = Cache::new(du, false, &flat_u, dv, false, &flat_v, weights.as_ref())?;
            cache.build_cache(u, v, &flat_u, &flat_v, &poles, weights.as_ref())?;
            self.cache = Some(cache);
        }
        cached_d1(self.cache.as_ref(), u, v)
    }

    fn bspline_d1(
        &mut self,
        bspline: &GeomBSplineSurface,
        u: f64,
        v: f64,
    ) -> Result<GeomSurfaceD1, Failure> {
        let (u_first, u_last, v_first, v_last) = self.bounds;
        let (u_side, uu) = side(u, u_first, u_last);
        let (v_side, vv) = side(v, v_first, v_last);
        if u_side != 0 || v_side != 0 {
            let (local, knots) = if_uv_bound(bspline, (uu, vv), (u_side, v_side))?;
            if local {
                return bspline.local_d1(uu, vv, knots);
            }
        }
        let valid: bool = self
            .cache
            .as_ref()
            .is_some_and(|cache| cache.is_cache_valid(u, v));
        if !valid {
            self.rebuild_bspline_cache(bspline, u, v)?;
        }
        cached_d1(self.cache.as_ref(), u, v)
    }

    // RebuildCache for a B-spline: the cache is created once and rebuilt in place.
    fn rebuild_bspline_cache(
        &mut self,
        bspline: &GeomBSplineSurface,
        u: f64,
        v: f64,
    ) -> Result<(), Failure> {
        let (flat_u, flat_v) = (bspline.u_knot_sequence(), bspline.v_knot_sequence());
        let (poles, weights) = (bspline.poles(), bspline.weights());
        if self.cache.is_none() {
            let cache = Cache::new(
                bspline.u_degree(),
                bspline.is_u_periodic(),
                &flat_u,
                bspline.v_degree(),
                bspline.is_v_periodic(),
                &flat_v,
                weights.as_ref(),
            )?;
            self.cache = Some(cache);
        }
        if let Some(cache) = self.cache.as_mut() {
            cache.build_cache(u, v, &flat_u, &flat_v, &poles, weights.as_ref())?;
        }
        Ok(())
    }

    /// `FirstUParameter` .. `LastVParameter`.
    pub(super) fn bounds(&self) -> (f64, f64, f64, f64) {
        self.bounds
    }
}

fn flat_bezier(degree: i32) -> Result<Array1<'static, f64>, Failure> {
    let knots: &[f64] = bsplclib::flat_bezier_knots(degree)?;
    let count: usize = 2 * (degree as usize + 1);
    let mut array: Array1<'static, f64> =
        Array1::with_bounds(1, count as i32).map_err(errors::array_bounds)?;
    array.as_mut_slice().copy_from_slice(&knots[..count]);
    Ok(array)
}

fn cached_d1(cache: Option<&Cache>, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
    let Some(cache) = cache else {
        return Err(errors::program(
            "GeomAdaptor_Surface::EvalD1: no cache; expected a built cache",
        ));
    };
    let (point, d1u, d1v): (Point3, Vector3, Vector3) = cache.d1(u, v)?;
    Ok(GeomSurfaceD1 { point, d1u, d1v })
}

// The side of a parameter at the adaptor bounds (tolerance 0): 1 at the first
// bound, -1 at the last, with the parameter snapped to the bound.
fn side(value: f64, first: f64, last: f64) -> (i32, f64) {
    if (value - first).abs() <= 0.0 {
        return (1, first);
    }
    if (value - last).abs() <= 0.0 {
        return (-1, last);
    }
    (0, value)
}

// GeomAdaptor_Surface::IfUVBound: the local spans and whether either parameter is
// on a knot.
fn if_uv_bound(
    bspline: &GeomBSplineSurface,
    uv: (f64, f64),
    sides: (i32, i32),
) -> Result<(bool, LocalKnots), Failure> {
    let (u_first_index, u_last_index) =
        (bspline.first_u_knot_index()?, bspline.last_u_knot_index()?);
    let (v_first_index, v_last_index) =
        (bspline.first_v_knot_index()?, bspline.last_v_knot_index()?);
    let (ideb, ifin) = bspline.locate_u(uv.0, pos_tol(), false)?;
    let mut local: bool = ideb == ifin;
    let u_span = span(sides.0, (ideb, ifin), (u_first_index, u_last_index));
    let (ivdeb, ivfin) = bspline.locate_v(uv.1, pos_tol(), false)?;
    if ivdeb == ivfin {
        local = true;
    }
    let v_span = span(sides.1, (ivdeb, ivfin), (v_first_index, v_last_index));
    Ok((local, (u_span, v_span)))
}

// GeomAdaptor_Surface::Span: the knot interval used by LocalD1.
fn span(side: i32, located: (i32, i32), range: (i32, i32)) -> (i32, i32) {
    let ((ideb, ifin), (first, last)) = (located, range);
    if ideb != ifin {
        if ideb < first {
            return (first, first + 1);
        }
        if ifin > last || ideb >= last - 1 {
            return (last - 1, last);
        }
        if ifin <= first + 1 {
            return (first, first + 1);
        }
        if ideb > ifin {
            return (ifin - 1, ifin);
        }
        return (ideb, ifin);
    }
    if ideb <= first {
        return (first, first + 1);
    }
    if ifin >= last {
        return (last - 1, last);
    }
    if side == -1 {
        (ideb - 1, ifin)
    } else {
        (ideb, ifin + 1)
    }
}

/// `isQPunctual`: whether the largest tangent along the iso (11 samples) lies in
/// `[tol_min, tol_max]`; `is_q_punctual(&mut adaptor, v1, true, 0.0, 1e-7)?`.
pub(super) fn is_q_punctual(
    adaptor: &mut SplineAdaptor,
    parameter: f64,
    iso_v: bool,
    tol_min: f64,
    tol_max: f64,
) -> Result<bool, Failure> {
    const THE_NB_SUBDIVISIONS: i32 = 10;
    let (u_first, u_last, v_first, v_last) = adaptor.bounds();
    let (p_min, p_max) = if iso_v {
        (u_first, u_last)
    } else {
        (v_first, v_last)
    };
    let range: f64 = p_max - p_min;
    let mut d1_norm_max: f64 = 0.0;
    for index in 0..=THE_NB_SUBDIVISIONS {
        let sample: f64 = if index == THE_NB_SUBDIVISIONS {
            p_max
        } else {
            p_min + range * f64::from(index) / f64::from(THE_NB_SUBDIVISIONS)
        };
        let norm: f64 = if iso_v {
            adaptor.eval_d1(sample, parameter)?.d1u.magnitude()
        } else {
            adaptor.eval_d1(parameter, sample)?.d1v.magnitude()
        };
        d1_norm_max = std_max(d1_norm_max, norm);
    }
    Ok(!(d1_norm_max > tol_max || d1_norm_max < tol_min))
}
