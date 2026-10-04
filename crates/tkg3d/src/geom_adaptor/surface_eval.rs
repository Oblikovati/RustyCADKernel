// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Surface.cxx (EvalD0..EvalDN, RebuildCache, offsetDN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation of `GeomAdaptor_Surface`.
//!
//! Elementary surfaces go through ElSLib on the recorded gp geometry. Bezier and
//! B-spline surfaces go through the span cache, except that a B-spline evaluated at
//! a snapped bound on a knot uses the local evaluators of the boundary span, and
//! except when the surface had an evaluation representation at load time, which the
//! surface then evaluates itself. Swept surfaces combine the basis curve adaptor
//! with the Geom extrusion and revolution utilities, offset surfaces the
//! equivalent-surface adaptor or the basis adaptor with Geom_OffsetSurfaceUtils.
use super::surface::GeomAdaptorSurface;
use super::surface_eval_order::{CachedOrder, SurfaceEvalOrder};
use super::surface_span::{Snapped, if_uv_bound, snap};
use super::surface_state::{BSplineData, BezierData, OffsetData, SurfaceData, SurfaceState};
use crate::geom::errors::{array_bounds, undefined_derivative};
use crate::geom::offset_surface_utils::{self as utils, OsculatingQuery};
use crate::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomSurface, extrusion_utils, revolution_utils,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::flat_bezier_knots;
use rustycad_tkmath::bsplslib::{Cache, SharedBSplineSurfaceCache};
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::Vector3;

/// The shared switch of EvalD0..EvalD3 on the surface kind: elementary kinds and
/// the swept, offset and default branches read the snapped parameters, the
/// evaluation representation the raw ones; `evaluate::<EvalD1>(&mut s, u, v)?`.
pub(super) fn evaluate<O: SurfaceEvalOrder>(
    state: &mut SurfaceState,
    u: f64,
    v: f64,
) -> Result<O::Output, Failure> {
    let snapped: Snapped = if O::SNAPS {
        snap(state, u, v)
    } else {
        Snapped::raw(u, v)
    };
    let (su, sv) = (snapped.u, snapped.v);
    if let Some(result) = O::elementary(&state.data, su, sv) {
        return Ok(result);
    }
    // The Bezier branch checks its representation itself: EvalD3 has no Bezier case.
    let bezier: bool = matches!(state.data, SurfaceData::Bezier(_));
    if state.data.has_eval_rep() && !bezier {
        return O::surface(state.surface()?, u, v);
    }
    match &mut state.data {
        SurfaceData::Bezier(data) => O::bezier(data, (u, v), snapped),
        SurfaceData::BSpline(data) => O::bspline(data, (u, v), snapped),
        SurfaceData::Extrusion(data) => O::extrusion(su, sv, data.curve()?, data.direction),
        SurfaceData::Revolution(data) => O::revolution(su, sv, data.curve()?, data.axis),
        SurfaceData::Offset(data) => O::offset(su, sv, data)?.ok_or_else(|| O::failure()),
        _ => O::surface(state.surface()?, su, sv),
    }
}

/// The Bezier branch of D0..D2: the surface itself when it had an evaluation
/// representation at load time, else the cache, built once on the first evaluation
/// (its single span is valid everywhere); `cached_bezier::<EvalD1>(&mut data, (u, v))?`.
pub(super) fn cached_bezier<O: CachedOrder>(
    data: &mut BezierData,
    (u, v): (f64, f64),
) -> Result<O::Output, Failure> {
    if data.eval_rep.is_some() {
        return O::surface(&*data.surface, u, v);
    }
    if data.cache.is_null() {
        data.cache = SharedBSplineSurfaceCache::new(bezier_cache(&data.surface, (u, v))?);
    }
    data.cache
        .with(|cache| O::cached(cache, u, v))
        .expect("Bezier cache built above")
}

/// The B-spline branch of D0: the cache, rebuilt when `(u, v)` leaves its span;
/// `cached_bspline::<EvalD0>(&mut data, (u, v))?`.
pub(super) fn cached_bspline<O: CachedOrder>(
    data: &mut BSplineData,
    (u, v): (f64, f64),
) -> Result<O::Output, Failure> {
    let valid: bool = data
        .cache
        .with(|cache| cache.is_cache_valid(u, v))
        .unwrap_or(false);
    if !valid {
        rebuild_bspline_cache(data, (u, v))?;
    }
    data.cache
        .with(|cache| O::cached(cache, u, v))
        .expect("B-spline cache built above")
}

/// The B-spline branch of D1 and D2: the boundary span at a snapped bound on a knot
/// (`IfUVBound`), the cache otherwise; `bound_or_cached_bspline::<EvalD1>(&mut d, raw, s)?`.
pub(super) fn bound_or_cached_bspline<O: CachedOrder>(
    data: &mut BSplineData,
    raw: (f64, f64),
    snapped: Snapped,
) -> Result<O::Output, Failure> {
    if snapped.on_bound() {
        let (local, knots) = if_uv_bound(&data.surface, snapped)?;
        if local {
            return O::local(&data.surface, snapped.u, snapped.v, knots);
        }
    }
    cached_bspline::<O>(data, raw)
}

// RebuildCache for a B-spline: create the cache once, then fill it on the span of (u, v).
fn rebuild_bspline_cache(data: &mut BSplineData, (u, v): (f64, f64)) -> Result<(), Failure> {
    let surface: &GeomBSplineSurface = &data.surface;
    let (flat_u, flat_v) = (surface.u_knot_sequence(), surface.v_knot_sequence());
    let (poles, weights) = (surface.poles(), surface.weights());
    if data.cache.is_null() {
        let cache = Cache::new(
            surface.u_degree(),
            surface.is_u_periodic(),
            &flat_u,
            surface.v_degree(),
            surface.is_v_periodic(),
            &flat_v,
            weights.as_ref(),
        )?;
        data.cache = SharedBSplineSurfaceCache::new(cache);
    }
    data.cache
        .with_mut(|cache| cache.build_cache(u, v, &flat_u, &flat_v, &poles, weights.as_ref()))
        .expect("B-spline cache created above")
}

// RebuildCache for a Bezier surface: the flat knots of each degree.
fn bezier_cache(surface: &GeomBezierSurface, (u, v): (f64, f64)) -> Result<Cache, Failure> {
    let (degree_u, degree_v) = (surface.u_degree(), surface.v_degree());
    let (flat_u, flat_v) = (bezier_knots(degree_u)?, bezier_knots(degree_v)?);
    let (poles, weights) = (surface.poles(), surface.weights());
    let mut cache = Cache::new(
        degree_u,
        surface.is_u_periodic(),
        &flat_u,
        degree_v,
        surface.is_v_periodic(),
        &flat_v,
        weights.as_ref(),
    )?;
    cache.build_cache(u, v, &flat_u, &flat_v, &poles, weights.as_ref())?;
    Ok(cache)
}

// NCollection_Array1<double>(BSplCLib::FlatBezierKnots(Deg), 1, 2 * (Deg + 1)).
fn bezier_knots(degree: i32) -> Result<Array1<'static, f64>, Failure> {
    let knots: &'static [f64] = flat_bezier_knots(degree)?;
    let count: usize = 2 * (degree as usize + 1);
    let mut array: Array1<'static, f64> =
        Array1::with_bounds(1, count as i32).map_err(array_bounds)?;
    array.as_mut_slice().copy_from_slice(&knots[..count]);
    Ok(array)
}

/// `EvalDN`: the snapped parameters throughout, the B-spline boundary span at a
/// snapped knot, the swept and offset utilities, ElSLib for elementary kinds and the
/// surface itself otherwise (Bezier included); `dn(&state, u, v, (1, 2))?`.
pub(super) fn dn(
    state: &SurfaceState,
    u: f64,
    v: f64,
    (nu, nv): (i32, i32),
) -> Result<Vector3, Failure> {
    let snapped: Snapped = snap(state, u, v);
    let (su, sv) = (snapped.u, snapped.v);
    let swept_or_spline = matches!(
        state.data,
        SurfaceData::BSpline(_)
            | SurfaceData::Extrusion(_)
            | SurfaceData::Revolution(_)
            | SurfaceData::Offset(_)
    );
    if swept_or_spline && state.data.has_eval_rep() {
        return state.surface()?.eval_dn(u, v, nu, nv);
    }
    Ok(match &state.data {
        SurfaceData::BSpline(data) => return bspline_dn(&data.surface, snapped, (nu, nv)),
        SurfaceData::Extrusion(data) => {
            extrusion_utils::dn(su, data.curve()?, data.direction, nu, nv)?
        }
        SurfaceData::Revolution(data) => {
            revolution_utils::dn(su, sv, data.curve()?, data.axis, (nu, nv))?
        }
        SurfaceData::Offset(data) => offset_dn(su, sv, data, (nu, nv))?,
        SurfaceData::Plane(plane) => elslib::dn(su, sv, plane, nu, nv),
        SurfaceData::Cylinder(cylinder) => elslib::dn(su, sv, cylinder, nu, nv),
        SurfaceData::Cone(cone) => elslib::dn(su, sv, cone, nu, nv),
        SurfaceData::Sphere(sphere) => elslib::dn(su, sv, sphere, nu, nv),
        SurfaceData::Torus(torus) => elslib::dn(su, sv, torus, nu, nv),
        _ => state.surface()?.eval_dn(su, sv, nu, nv)?,
    })
}

fn bspline_dn(
    surface: &GeomBSplineSurface,
    snapped: Snapped,
    orders: (i32, i32),
) -> Result<Vector3, Failure> {
    if snapped.on_bound() {
        let (local, knots) = if_uv_bound(surface, snapped)?;
        if local {
            return surface.local_dn(snapped.u, snapped.v, knots, orders);
        }
    }
    surface.eval_dn(snapped.u, snapped.v, orders.0, orders.1)
}

// offsetDN: the equivalent adaptor, else Geom_OffsetSurfaceUtils::EvaluateDN.
fn offset_dn(u: f64, v: f64, data: &OffsetData, orders: (i32, i32)) -> Result<Vector3, Failure> {
    if let Some(equivalent) = data.equivalent_adaptor.get() {
        return equivalent.dn_at(u, v, orders);
    }
    let basis: &GeomAdaptorSurface = &data.basis_adaptor;
    let query: Option<&dyn OsculatingQuery> = data
        .offset_surface
        .get()
        .map(|surface| surface as &dyn OsculatingQuery);
    utils::evaluate_dn((u, v), orders, basis, data.offset, query)?
        .ok_or_else(|| undefined_derivative("GeomAdaptor_Surface::EvalDN: evaluation failed"))
}
