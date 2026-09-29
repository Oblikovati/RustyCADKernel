// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of GeomProjLib::Project and GeomProjLib::ProjectOnPlane
// (ModelingData/TKGeomBase/GeomProjLib/GeomProjLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 3D projections of GeomProjLib: the normal projection on a surface and the
//! projection on a plane along a direction.
use super::{null_input, trimmed_range};
use crate::approx::{CurveOnSurface, CurveOnSurfaceOptions};
use crate::proj_lib::{CompProjectedCurve, ProjectOnPlane};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomElementarySurface, GeomEllipse, GeomHyperbola, GeomLine,
    GeomParabola, GeomPlane, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::{Axis3, Direction3};

/// `GeomProjLib::Project(C, S)`: the 3D curve of the normal projection of `curve` on
/// `surface`. On a plane it is `ProjLib_ProjectOnPlane` along the normal (a trimmed
/// curve stays trimmed over its own parameters); on another surface the first part
/// of `ProjLib_CompProjectedCurve` (tolerance 1e-4) approximated by
/// `Approx_CurveOnSurface` as a B-spline. A failed projection returns a null handle;
/// a projection with no part raises Standard_OutOfRange;
/// `let projected = project(&curve, &surface)?;`.
pub fn project(
    curve: &Handle<dyn GeomCurve>,
    surface: &Handle<dyn GeomSurface>,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let adapted_curve = GeomAdaptorCurve::with_curve(curve)?;
    let adapted_surface = GeomAdaptorSurface::with_surface(surface)?;
    let hc: Handle<dyn Adaptor3dCurve> = Handle::new(adapted_curve).cast();
    if adapted_surface.get_type()? == SurfaceType::Plane {
        return project_on_plane_surface(curve, &hc, adapted_surface.plane()?.position());
    }
    let hs: Handle<dyn Adaptor3dSurface> = Handle::new(adapted_surface).cast();
    project_by_approximation(&hc, &hs)
}

// The normal projection on a plane, with PApproximation and the parametrization kept
// (the default `KeepParametrization = true` of ProjLib_ProjectOnPlane::Load).
fn project_on_plane_surface(
    curve: &Handle<dyn GeomCurve>,
    hc: &Handle<dyn Adaptor3dCurve>,
    plane: Axis3,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let projection: ProjectOnPlane = ProjectOnPlane::with_plane(&plane);
    projection.load(hc, precision::p_approximation(), true)?;
    let Some(projected) = typed_curve3d(&projection)? else {
        return Ok(Handle::default());
    };
    let Some((u1, u2)) = trimmed_range(curve) else {
        return Ok(projected);
    };
    Ok(Handle::new(GeomTrimmedCurve::new(&projected, u1, u2, true, true)?).cast())
}

// The first CompProjectedCurve part, trimmed to its bounds, approximated in 3D
// (16 segments, degree 14, C2). The native tolerances: Tol = 1e-4 and
// TolU = TolV = pow(Tol, 2/3) (std::pow, the libm pow as powf).
fn project_by_approximation(
    hc: &Handle<dyn Adaptor3dCurve>,
    hs: &Handle<dyn Adaptor3dSurface>,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let tol: f64 = 0.0001;
    let tol_u: f64 = tol.powf(2. / 3.);
    let tol_v: f64 = tol.powf(2. / 3.);
    let projection = CompProjectedCurve::with_max_dist(hs, hc, tol_u, tol_v, -1.)?;
    let (first, last) = projection.bounds(1)?;
    let hc2d: Handle<dyn Adaptor2dCurve2d> = projection.trim(first, last, tol_u)?;
    let mut approx = CurveOnSurface::without_perform(&hc2d, hs, first, last, tol);
    approx.perform(CurveOnSurfaceOptions {
        max_segments: 16,
        max_degree: 14,
        continuity: Shape::C2,
        only_3d: true,
        only_2d: false,
    })?;
    // Here the result is always a B-spline.
    if approx.is_done() && approx.has_result() {
        return Ok(approx.curve_3d().cast());
    }
    Ok(Handle::default())
}

/// `GeomProjLib::ProjectOnPlane(Curve, Plane, Dir, KeepParametrization)`: the
/// projection of `curve` on `plane` along `direction` (tolerance Approximation).
/// With `keep_parametrization` the projected curve PC satisfies
/// `proj(C(u)) = PC(u)`; otherwise the parametrization may change. A trimmed curve
/// gives a trimmed result over the projection's parameters. A kind that is not a
/// curve kind returns a null handle;
/// `project_on_plane(&curve, &plane, direction, false)?`.
pub fn project_on_plane(
    curve: &Handle<dyn GeomCurve>,
    plane: &Handle<GeomPlane>,
    direction: Direction3,
    keep_parametrization: bool,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let adapted_curve = GeomAdaptorCurve::with_curve(curve)?;
    let hc: Handle<dyn Adaptor3dCurve> = Handle::new(adapted_curve).cast();
    let geom_plane: &GeomPlane = plane.get().ok_or_else(|| null_input("the plane"))?;
    let projection = ProjectOnPlane::with_direction(&geom_plane.position(), direction);
    projection.load(&hc, precision::approximation(), keep_parametrization)?;
    let Some(projected) = typed_curve3d(&projection)? else {
        return Ok(Handle::default());
    };
    if curve.cast::<GeomTrimmedCurve>().is_null() {
        return Ok(projected);
    }
    let (u1, u2) = (projection.first_parameter()?, projection.last_parameter()?);
    Ok(Handle::new(GeomTrimmedCurve::new(&projected, u1, u2, true, true)?).cast())
}

// The Geom curve of the projection kind; None for the other kinds (the native
// `default: return GC;`). The Bezier and B-spline results are those of the adaptor.
fn typed_curve3d(projection: &ProjectOnPlane) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
    let curve: Handle<dyn GeomCurve> = match projection.get_type()? {
        CurveType::Line => Handle::new(GeomLine::from_lin(projection.line()?)).cast(),
        CurveType::Circle => Handle::new(GeomCircle::from_circ(projection.circle()?)).cast(),
        CurveType::Ellipse => Handle::new(GeomEllipse::from_elips(projection.ellipse()?)).cast(),
        CurveType::Parabola => Handle::new(GeomParabola::from_parab(projection.parabola()?)).cast(),
        CurveType::Hyperbola => {
            Handle::new(GeomHyperbola::from_hypr(projection.hyperbola()?)).cast()
        }
        CurveType::BezierCurve => projection.bezier()?.cast(),
        CurveType::BSplineCurve => projection.bspline()?.cast(),
        _ => return Ok(None),
    };
    Ok(Some(curve))
}
