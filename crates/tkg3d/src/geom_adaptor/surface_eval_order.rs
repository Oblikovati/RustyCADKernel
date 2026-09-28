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
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Surface.cxx (the per-order branches of EvalD0..EvalD3 and offsetD0..offsetD3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The four native evaluators repeat one switch with order-specific calls; each
//! order here supplies those calls, so the switch is written once.
use super::surface::GeomAdaptorSurface;
use super::surface_eval::{bound_or_cached_bspline, cached_bezier, cached_bspline};
use super::surface_span::{Snapped, if_uv_bound};
use super::surface_state::{BSplineData, BezierData, OffsetData, SurfaceData};
use crate::adaptor3d::Adaptor3dCurve;
use crate::geom::elementary_results::{surface_d2, surface_d3};
use crate::geom::errors::{undefined_derivative, undefined_value};
use crate::geom::offset_surface_utils::{self as utils, OsculatingQuery};
use crate::geom::{
    GeomBSplineSurface, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, LocalKnots,
    extrusion_utils, revolution_utils,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplslib::{Cache, SurfaceD2 as BSplSLibD2};
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis1, Point3, Vector3, Xyz};

/// The order-specific calls of one `EvalDk` of `GeomAdaptor_Surface`.
pub(super) trait SurfaceEvalOrder {
    /// `gp_Pnt` or `Geom_Surface::ResDk`.
    type Output;
    /// Whether the parameters are snapped to the bounds (all orders but D0).
    const SNAPS: bool;
    /// ElSLib on elementary data, `None` for the other kinds.
    fn elementary(data: &SurfaceData, u: f64, v: f64) -> Option<Self::Output>;
    /// `mySurface->EvalDk(U, V)`.
    fn surface(surface: &dyn GeomSurface, u: f64, v: f64) -> Result<Self::Output, Failure>;
    /// `LocalDk` on the given knot spans.
    fn local(
        surface: &GeomBSplineSurface,
        u: f64,
        v: f64,
        knots: LocalKnots,
    ) -> Result<Self::Output, Failure>;
    /// The Bezier branch (before it, the evaluation representation was checked);
    /// `raw` are the unsnapped parameters.
    fn bezier(
        data: &mut BezierData,
        raw: (f64, f64),
        snapped: Snapped,
    ) -> Result<Self::Output, Failure>;
    /// The B-spline branch.
    fn bspline(
        data: &mut BSplineData,
        raw: (f64, f64),
        snapped: Snapped,
    ) -> Result<Self::Output, Failure>;
    /// `Geom_ExtrusionUtils::Dk`.
    fn extrusion(
        u: f64,
        v: f64,
        basis: &dyn Adaptor3dCurve,
        dir: Xyz,
    ) -> Result<Self::Output, Failure>;
    /// `Geom_RevolutionUtils::Dk`.
    fn revolution(
        u: f64,
        v: f64,
        basis: &dyn Adaptor3dCurve,
        axis: Axis1,
    ) -> Result<Self::Output, Failure>;
    /// `offsetDk`: the equivalent adaptor, else Geom_OffsetSurfaceUtils::EvaluateDk.
    fn offset(u: f64, v: f64, data: &OffsetData) -> Result<Option<Self::Output>, Failure>;
    /// The exception of a failed evaluation.
    fn failure() -> Failure;
}

/// An order that reads the span cache (`Cache->Dk`): D0, D1 and D2.
pub(super) trait CachedOrder: SurfaceEvalOrder {
    /// `Cache->Dk(U, V)`.
    fn cached(cache: &Cache, u: f64, v: f64) -> Result<Self::Output, Failure>;
}

const ZERO: Vector3 = Vector3::new(0.0, 0.0, 0.0);

// The osculating query of the offset evaluators: the offset surface itself.
fn osculating(data: &OffsetData) -> Option<&dyn OsculatingQuery> {
    data.offset_surface
        .get()
        .map(|surface| surface as &dyn OsculatingQuery)
}

/// `EvalD0`.
pub(super) struct EvalD0;
impl SurfaceEvalOrder for EvalD0 {
    type Output = Point3;
    const SNAPS: bool = false;
    fn elementary(data: &SurfaceData, u: f64, v: f64) -> Option<Point3> {
        Some(match data {
            SurfaceData::Plane(plane) => elslib::d0(u, v, plane),
            SurfaceData::Cylinder(cylinder) => elslib::d0(u, v, cylinder),
            SurfaceData::Cone(cone) => elslib::d0(u, v, cone),
            SurfaceData::Sphere(sphere) => elslib::d0(u, v, sphere),
            SurfaceData::Torus(torus) => elslib::d0(u, v, torus),
            _ => return None,
        })
    }
    fn surface(surface: &dyn GeomSurface, u: f64, v: f64) -> Result<Point3, Failure> {
        surface.eval_d0(u, v)
    }
    fn local(
        surface: &GeomBSplineSurface,
        u: f64,
        v: f64,
        knots: LocalKnots,
    ) -> Result<Point3, Failure> {
        surface.local_d0(u, v, knots)
    }
    fn bezier(
        data: &mut BezierData,
        raw: (f64, f64),
        _snapped: Snapped,
    ) -> Result<Point3, Failure> {
        cached_bezier::<Self>(data, raw)
    }
    fn bspline(
        data: &mut BSplineData,
        raw: (f64, f64),
        _snapped: Snapped,
    ) -> Result<Point3, Failure> {
        cached_bspline::<Self>(data, raw)
    }
    fn extrusion(u: f64, v: f64, basis: &dyn Adaptor3dCurve, dir: Xyz) -> Result<Point3, Failure> {
        extrusion_utils::d0(u, v, basis, dir)
    }
    fn revolution(
        u: f64,
        v: f64,
        basis: &dyn Adaptor3dCurve,
        axis: Axis1,
    ) -> Result<Point3, Failure> {
        revolution_utils::d0(u, v, basis, axis)
    }
    fn offset(u: f64, v: f64, data: &OffsetData) -> Result<Option<Point3>, Failure> {
        if let Some(equivalent) = data.equivalent_adaptor.get() {
            return Ok(Some(equivalent_eval::<EvalD0>(equivalent, u, v)?));
        }
        let basis: &GeomAdaptorSurface = &data.basis_adaptor;
        utils::evaluate_d0((u, v), basis, data.offset, osculating(data))
    }
    fn failure() -> Failure {
        undefined_value("GeomAdaptor_Surface::EvalD0: evaluation failed")
    }
}

/// `EvalD1`.
pub(super) struct EvalD1;
impl SurfaceEvalOrder for EvalD1 {
    type Output = GeomSurfaceD1;
    const SNAPS: bool = true;
    fn elementary(data: &SurfaceData, u: f64, v: f64) -> Option<GeomSurfaceD1> {
        let (point, d1u, d1v) = match data {
            SurfaceData::Plane(plane) => elslib::d1(u, v, plane),
            SurfaceData::Cylinder(cylinder) => elslib::d1(u, v, cylinder),
            SurfaceData::Cone(cone) => elslib::d1(u, v, cone),
            SurfaceData::Sphere(sphere) => elslib::d1(u, v, sphere),
            SurfaceData::Torus(torus) => elslib::d1(u, v, torus),
            _ => return None,
        };
        Some(GeomSurfaceD1 { point, d1u, d1v })
    }
    fn surface(surface: &dyn GeomSurface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        surface.eval_d1(u, v)
    }
    fn local(
        surface: &GeomBSplineSurface,
        u: f64,
        v: f64,
        knots: LocalKnots,
    ) -> Result<GeomSurfaceD1, Failure> {
        surface.local_d1(u, v, knots)
    }
    fn bezier(
        data: &mut BezierData,
        raw: (f64, f64),
        _snapped: Snapped,
    ) -> Result<GeomSurfaceD1, Failure> {
        cached_bezier::<Self>(data, raw)
    }
    fn bspline(
        data: &mut BSplineData,
        raw: (f64, f64),
        snapped: Snapped,
    ) -> Result<GeomSurfaceD1, Failure> {
        bound_or_cached_bspline::<Self>(data, raw, snapped)
    }
    fn extrusion(
        u: f64,
        v: f64,
        basis: &dyn Adaptor3dCurve,
        dir: Xyz,
    ) -> Result<GeomSurfaceD1, Failure> {
        extrusion_utils::d1(u, v, basis, dir)
    }
    fn revolution(
        u: f64,
        v: f64,
        basis: &dyn Adaptor3dCurve,
        axis: Axis1,
    ) -> Result<GeomSurfaceD1, Failure> {
        revolution_utils::d1(u, v, basis, axis)
    }
    fn offset(u: f64, v: f64, data: &OffsetData) -> Result<Option<GeomSurfaceD1>, Failure> {
        if let Some(equivalent) = data.equivalent_adaptor.get() {
            return Ok(Some(equivalent_eval::<EvalD1>(equivalent, u, v)?));
        }
        let basis: &GeomAdaptorSurface = &data.basis_adaptor;
        utils::evaluate_d1((u, v), basis, data.offset, osculating(data))
    }
    fn failure() -> Failure {
        undefined_derivative("GeomAdaptor_Surface::EvalD1: evaluation failed")
    }
}

/// `EvalD2`: a plane has no ElSLib D2, its second derivatives are set to zero.
pub(super) struct EvalD2;
impl SurfaceEvalOrder for EvalD2 {
    type Output = GeomSurfaceD2;
    const SNAPS: bool = true;
    fn elementary(data: &SurfaceData, u: f64, v: f64) -> Option<GeomSurfaceD2> {
        Some(match data {
            SurfaceData::Plane(plane) => {
                let (point, d1u, d1v) = elslib::d1(u, v, plane);
                GeomSurfaceD2 {
                    point,
                    d1u,
                    d1v,
                    d2u: ZERO,
                    d2v: ZERO,
                    d2uv: ZERO,
                }
            }
            SurfaceData::Cylinder(cylinder) => surface_d2(elslib::d2(u, v, cylinder)),
            SurfaceData::Cone(cone) => surface_d2(elslib::d2(u, v, cone)),
            SurfaceData::Sphere(sphere) => surface_d2(elslib::d2(u, v, sphere)),
            SurfaceData::Torus(torus) => surface_d2(elslib::d2(u, v, torus)),
            _ => return None,
        })
    }
    fn surface(surface: &dyn GeomSurface, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        surface.eval_d2(u, v)
    }
    fn local(
        surface: &GeomBSplineSurface,
        u: f64,
        v: f64,
        knots: LocalKnots,
    ) -> Result<GeomSurfaceD2, Failure> {
        surface.local_d2(u, v, knots)
    }
    fn bezier(
        data: &mut BezierData,
        raw: (f64, f64),
        _snapped: Snapped,
    ) -> Result<GeomSurfaceD2, Failure> {
        cached_bezier::<Self>(data, raw)
    }
    fn bspline(
        data: &mut BSplineData,
        raw: (f64, f64),
        snapped: Snapped,
    ) -> Result<GeomSurfaceD2, Failure> {
        bound_or_cached_bspline::<Self>(data, raw, snapped)
    }
    fn extrusion(
        u: f64,
        v: f64,
        basis: &dyn Adaptor3dCurve,
        dir: Xyz,
    ) -> Result<GeomSurfaceD2, Failure> {
        extrusion_utils::d2(u, v, basis, dir)
    }
    fn revolution(
        u: f64,
        v: f64,
        basis: &dyn Adaptor3dCurve,
        axis: Axis1,
    ) -> Result<GeomSurfaceD2, Failure> {
        revolution_utils::d2(u, v, basis, axis)
    }
    fn offset(u: f64, v: f64, data: &OffsetData) -> Result<Option<GeomSurfaceD2>, Failure> {
        if let Some(equivalent) = data.equivalent_adaptor.get() {
            return Ok(Some(equivalent_eval::<EvalD2>(equivalent, u, v)?));
        }
        let basis: &GeomAdaptorSurface = &data.basis_adaptor;
        utils::evaluate_d2((u, v), basis, data.offset, osculating(data))
    }
    fn failure() -> Failure {
        undefined_derivative("GeomAdaptor_Surface::EvalD2: evaluation failed")
    }
}

/// `EvalD3`: a plane has no ElSLib D3 (zero second and third derivatives), and there
/// is neither a Bezier case nor a cached evaluation.
pub(super) struct EvalD3;
impl SurfaceEvalOrder for EvalD3 {
    type Output = GeomSurfaceD3;
    const SNAPS: bool = true;
    fn elementary(data: &SurfaceData, u: f64, v: f64) -> Option<GeomSurfaceD3> {
        Some(match data {
            SurfaceData::Plane(plane) => plane_d3(elslib::d1(u, v, plane)),
            SurfaceData::Cylinder(cylinder) => surface_d3(elslib::d3(u, v, cylinder)),
            SurfaceData::Cone(cone) => surface_d3(elslib::d3(u, v, cone)),
            SurfaceData::Sphere(sphere) => surface_d3(elslib::d3(u, v, sphere)),
            SurfaceData::Torus(torus) => surface_d3(elslib::d3(u, v, torus)),
            _ => return None,
        })
    }
    fn surface(surface: &dyn GeomSurface, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        surface.eval_d3(u, v)
    }
    fn local(
        surface: &GeomBSplineSurface,
        u: f64,
        v: f64,
        knots: LocalKnots,
    ) -> Result<GeomSurfaceD3, Failure> {
        surface.local_d3(u, v, knots)
    }
    // No Bezier case natively: the default branch, the surface on the snapped
    // parameters.
    fn bezier(
        data: &mut BezierData,
        _raw: (f64, f64),
        snapped: Snapped,
    ) -> Result<GeomSurfaceD3, Failure> {
        data.surface.eval_d3(snapped.u, snapped.v)
    }
    // The surface itself inside the bounds, the boundary span at a snapped knot.
    fn bspline(
        data: &mut BSplineData,
        _raw: (f64, f64),
        snapped: Snapped,
    ) -> Result<GeomSurfaceD3, Failure> {
        if snapped.on_bound() {
            let (local, knots) = if_uv_bound(&data.surface, snapped)?;
            if local {
                return data.surface.local_d3(snapped.u, snapped.v, knots);
            }
        }
        data.surface.eval_d3(snapped.u, snapped.v)
    }
    fn extrusion(
        u: f64,
        v: f64,
        basis: &dyn Adaptor3dCurve,
        dir: Xyz,
    ) -> Result<GeomSurfaceD3, Failure> {
        extrusion_utils::d3(u, v, basis, dir)
    }
    fn revolution(
        u: f64,
        v: f64,
        basis: &dyn Adaptor3dCurve,
        axis: Axis1,
    ) -> Result<GeomSurfaceD3, Failure> {
        revolution_utils::d3(u, v, basis, axis)
    }
    fn offset(u: f64, v: f64, data: &OffsetData) -> Result<Option<GeomSurfaceD3>, Failure> {
        if let Some(equivalent) = data.equivalent_adaptor.get() {
            return Ok(Some(equivalent_eval::<EvalD3>(equivalent, u, v)?));
        }
        let basis: &GeomAdaptorSurface = &data.basis_adaptor;
        utils::evaluate_d3((u, v), basis, data.offset, osculating(data))
    }
    fn failure() -> Failure {
        undefined_derivative("GeomAdaptor_Surface::EvalD3: evaluation failed")
    }
}

fn plane_d3((point, d1u, d1v): (Point3, Vector3, Vector3)) -> GeomSurfaceD3 {
    GeomSurfaceD3 {
        point,
        d1u,
        d1v,
        d2u: ZERO,
        d2v: ZERO,
        d2uv: ZERO,
        d3u: ZERO,
        d3v: ZERO,
        d3uuv: ZERO,
        d3uvv: ZERO,
    }
}

// The equivalent-surface shortcut of offsetDk: that adaptor's own EvalDk.
fn equivalent_eval<O: SurfaceEvalOrder>(
    adaptor: &GeomAdaptorSurface,
    u: f64,
    v: f64,
) -> Result<O::Output, Failure> {
    adaptor.evaluate::<O>(u, v)
}

impl CachedOrder for EvalD0 {
    fn cached(cache: &Cache, u: f64, v: f64) -> Result<Point3, Failure> {
        cache.d0(u, v)
    }
}
impl CachedOrder for EvalD1 {
    fn cached(cache: &Cache, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let (point, d1u, d1v) = cache.d1(u, v)?;
        Ok(GeomSurfaceD1 { point, d1u, d1v })
    }
}
impl CachedOrder for EvalD2 {
    fn cached(cache: &Cache, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let values: BSplSLibD2 = cache.d2(u, v)?;
        Ok(GeomSurfaceD2 {
            point: values.p,
            d1u: values.vu,
            d1v: values.vv,
            d2u: values.vuu,
            d2v: values.vvv,
            d2uv: values.vuv,
        })
    }
}
