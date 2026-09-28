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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_HSurfaceTool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_HSurfaceTool`: the static tool that the generic surface algorithms
//! call on a handle of `Adaptor3d_Surface`. The native class has only static
//! members, so they are free functions here: the forwarding ones call the member of
//! the same name, `NbSamplesU`/`NbSamplesV` pick sample counts from the surface kind
//! and `IsSurfG1` tests the G1 continuity of a spline boundary curve.
use super::curve::Adaptor3dCurve;
use super::surface::Adaptor3dSurface;
use crate::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierSurface, GeomCurve, GeomOffsetCurve,
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use crate::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};

/// The surface handle of every tool function.
pub type Adaptor3dSurfaceHandle = Handle<dyn Adaptor3dSurface>;

/// `FirstUParameter`; `first_u_parameter(&s)?`.
pub fn first_u_parameter(surface: &Adaptor3dSurfaceHandle) -> Result<f64, Failure> {
    surface.first_u_parameter()
}
/// `FirstVParameter`; `first_v_parameter(&s)?`.
pub fn first_v_parameter(surface: &Adaptor3dSurfaceHandle) -> Result<f64, Failure> {
    surface.first_v_parameter()
}
/// `LastUParameter`; `last_u_parameter(&s)?`.
pub fn last_u_parameter(surface: &Adaptor3dSurfaceHandle) -> Result<f64, Failure> {
    surface.last_u_parameter()
}
/// `LastVParameter`; `last_v_parameter(&s)?`.
pub fn last_v_parameter(surface: &Adaptor3dSurfaceHandle) -> Result<f64, Failure> {
    surface.last_v_parameter()
}
/// `NbUIntervals`; `nb_u_intervals(&s, Shape::C2)?`.
pub fn nb_u_intervals(surface: &Adaptor3dSurfaceHandle, shape: Shape) -> Result<i32, Failure> {
    surface.nb_u_intervals(shape)
}
/// `NbVIntervals`; `nb_v_intervals(&s, Shape::C2)?`.
pub fn nb_v_intervals(surface: &Adaptor3dSurfaceHandle, shape: Shape) -> Result<i32, Failure> {
    surface.nb_v_intervals(shape)
}
/// `UIntervals`; `u_intervals(&s, &mut t, Shape::C2)?`.
pub fn u_intervals(
    surface: &Adaptor3dSurfaceHandle,
    bounds: &mut Array1<'_, f64>,
    shape: Shape,
) -> Result<(), Failure> {
    surface.u_intervals(bounds, shape)
}
/// `VIntervals`; `v_intervals(&s, &mut t, Shape::C2)?`.
pub fn v_intervals(
    surface: &Adaptor3dSurfaceHandle,
    bounds: &mut Array1<'_, f64>,
    shape: Shape,
) -> Result<(), Failure> {
    surface.v_intervals(bounds, shape)
}
/// `UTrim`; `u_trim(&s, 0.0, 1.0, 1e-7)?`.
pub fn u_trim(
    surface: &Adaptor3dSurfaceHandle,
    first: f64,
    last: f64,
    tolerance: f64,
) -> Result<Adaptor3dSurfaceHandle, Failure> {
    surface.u_trim(first, last, tolerance)
}
/// `VTrim`; `v_trim(&s, 0.0, 1.0, 1e-7)?`.
pub fn v_trim(
    surface: &Adaptor3dSurfaceHandle,
    first: f64,
    last: f64,
    tolerance: f64,
) -> Result<Adaptor3dSurfaceHandle, Failure> {
    surface.v_trim(first, last, tolerance)
}
/// `IsUClosed`; `is_u_closed(&s)?`.
pub fn is_u_closed(surface: &Adaptor3dSurfaceHandle) -> Result<bool, Failure> {
    surface.is_u_closed()
}
/// `IsVClosed`; `is_v_closed(&s)?`.
pub fn is_v_closed(surface: &Adaptor3dSurfaceHandle) -> Result<bool, Failure> {
    surface.is_v_closed()
}
/// `IsUPeriodic`; `is_u_periodic(&s)?`.
pub fn is_u_periodic(surface: &Adaptor3dSurfaceHandle) -> Result<bool, Failure> {
    surface.is_u_periodic()
}
/// `UPeriod`; `u_period(&s)?`.
pub fn u_period(surface: &Adaptor3dSurfaceHandle) -> Result<f64, Failure> {
    surface.u_period()
}
/// `IsVPeriodic`; `is_v_periodic(&s)?`.
pub fn is_v_periodic(surface: &Adaptor3dSurfaceHandle) -> Result<bool, Failure> {
    surface.is_v_periodic()
}
/// `VPeriod`; `v_period(&s)?`.
pub fn v_period(surface: &Adaptor3dSurfaceHandle) -> Result<f64, Failure> {
    surface.v_period()
}
/// `Value`; `value(&s, 0.5, 0.5)?`.
pub fn value(surface: &Adaptor3dSurfaceHandle, u: f64, v: f64) -> Result<Point3, Failure> {
    surface.value(u, v)
}
/// `D0`; `d0(&s, 0.5, 0.5)?`.
pub fn d0(surface: &Adaptor3dSurfaceHandle, u: f64, v: f64) -> Result<Point3, Failure> {
    surface.d0(u, v)
}
/// `D1`, the output parameters as one record; `d1(&s, 0.5, 0.5)?.d1u`.
pub fn d1(surface: &Adaptor3dSurfaceHandle, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
    surface.d1(u, v)
}
/// `D2`; `d2(&s, 0.5, 0.5)?.d2uv`.
pub fn d2(surface: &Adaptor3dSurfaceHandle, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
    surface.d2(u, v)
}
/// `D3`; `d3(&s, 0.5, 0.5)?.d3uuv`.
pub fn d3(surface: &Adaptor3dSurfaceHandle, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
    surface.d3(u, v)
}
/// `DN`; `dn(&s, 0.5, 0.5, 1, 1)?`.
pub fn dn(
    surface: &Adaptor3dSurfaceHandle,
    u: f64,
    v: f64,
    nu: i32,
    nv: i32,
) -> Result<Vector3, Failure> {
    surface.dn(u, v, nu, nv)
}
/// `UResolution`; `u_resolution(&s, 1e-7)?`.
pub fn u_resolution(surface: &Adaptor3dSurfaceHandle, r3d: f64) -> Result<f64, Failure> {
    surface.u_resolution(r3d)
}
/// `VResolution`; `v_resolution(&s, 1e-7)?`.
pub fn v_resolution(surface: &Adaptor3dSurfaceHandle, r3d: f64) -> Result<f64, Failure> {
    surface.v_resolution(r3d)
}
/// `GetType`; `get_type(&s)?`.
pub fn get_type(surface: &Adaptor3dSurfaceHandle) -> Result<SurfaceType, Failure> {
    surface.get_type()
}
/// `Plane`; `plane(&s)?`.
pub fn plane(surface: &Adaptor3dSurfaceHandle) -> Result<Plane3, Failure> {
    surface.plane()
}
/// `Cylinder`; `cylinder(&s)?`.
pub fn cylinder(surface: &Adaptor3dSurfaceHandle) -> Result<Cylinder3, Failure> {
    surface.cylinder()
}
/// `Cone`; `cone(&s)?`.
pub fn cone(surface: &Adaptor3dSurfaceHandle) -> Result<Cone3, Failure> {
    surface.cone()
}
/// `Torus`; `torus(&s)?`.
pub fn torus(surface: &Adaptor3dSurfaceHandle) -> Result<Torus3, Failure> {
    surface.torus()
}
/// `Sphere`; `sphere(&s)?`.
pub fn sphere(surface: &Adaptor3dSurfaceHandle) -> Result<Sphere3, Failure> {
    surface.sphere()
}
/// `Bezier`; `bezier(&s)?`.
pub fn bezier(surface: &Adaptor3dSurfaceHandle) -> Result<Handle<GeomBezierSurface>, Failure> {
    surface.bezier()
}
/// `BSpline`; `bspline(&s)?`.
pub fn bspline(surface: &Adaptor3dSurfaceHandle) -> Result<Handle<GeomBSplineSurface>, Failure> {
    surface.bspline()
}
/// `AxeOfRevolution`; `axe_of_revolution(&s)?`.
pub fn axe_of_revolution(surface: &Adaptor3dSurfaceHandle) -> Result<Axis1, Failure> {
    surface.axe_of_revolution()
}
/// `Direction`; `direction(&s)?`.
pub fn direction(surface: &Adaptor3dSurfaceHandle) -> Result<Direction3, Failure> {
    surface.direction()
}
/// `BasisCurve`; `basis_curve(&s)?`.
pub fn basis_curve(
    surface: &Adaptor3dSurfaceHandle,
) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    surface.basis_curve()
}
/// `BasisSurface`; `basis_surface(&s)?`.
pub fn basis_surface(surface: &Adaptor3dSurfaceHandle) -> Result<Adaptor3dSurfaceHandle, Failure> {
    surface.basis_surface()
}
/// `OffsetValue`; `offset_value(&s)?`.
pub fn offset_value(surface: &Adaptor3dSurfaceHandle) -> Result<f64, Failure> {
    surface.offset_value()
}

/// `NbSamplesU`: 2 for a plane, `3 + NbUPoles` for a Bezier surface,
/// `NbUKnots * UDegree` (at least 2) for a B-spline, 20 for a torus, 10 otherwise;
/// `nb_samples_u(&s)?`.
pub fn nb_samples_u(surface: &Adaptor3dSurfaceHandle) -> Result<i32, Failure> {
    Ok(match surface.get_type()? {
        SurfaceType::Plane => 2,
        SurfaceType::BezierSurface => 3 + surface.nb_u_poles()?,
        SurfaceType::BSplineSurface => (surface.nb_u_knots()? * surface.u_degree()?).max(2),
        SurfaceType::Torus => 20,
        _ => 10,
    })
}
/// `NbSamplesV`: as `NbSamplesU` in V, with 15 for the curved elementary and swept
/// surfaces; `nb_samples_v(&s)?`.
pub fn nb_samples_v(surface: &Adaptor3dSurfaceHandle) -> Result<i32, Failure> {
    Ok(match surface.get_type()? {
        SurfaceType::Plane => 2,
        SurfaceType::BezierSurface => 3 + surface.nb_v_poles()?,
        SurfaceType::BSplineSurface => (surface.nb_v_knots()? * surface.v_degree()?).max(2),
        SurfaceType::Cylinder
        | SurfaceType::Cone
        | SurfaceType::Sphere
        | SurfaceType::Torus
        | SurfaceType::SurfaceOfRevolution
        | SurfaceType::SurfaceOfExtrusion => 15,
        _ => 10,
    })
}

// The ranged sample count of a count above 10: scaled by the truncated ratio of the
// ranges, kept in [5, min(count, 50)] (`(int)` of an infinite or NaN ratio is
// undefined natively; `as i32` saturates here, and the product wraps).
fn ranged_samples(count: i32, range: (f64, f64), whole: (f64, f64)) -> i32 {
    let mut n: i32 = count.wrapping_mul(((range.1 - range.0) / (whole.1 - whole.0)) as i32);
    if n > count || n > 50 {
        n = count;
    }
    if n < 5 {
        n = 5;
    }
    n
}
/// `NbSamplesU(S, u1, u2)`; `nb_samples_u_range(&s, 0.0, 0.5)?`.
pub fn nb_samples_u_range(
    surface: &Adaptor3dSurfaceHandle,
    u1: f64,
    u2: f64,
) -> Result<i32, Failure> {
    let count: i32 = nb_samples_u(surface)?;
    if count <= 10 {
        return Ok(count);
    }
    let whole = (first_u_parameter(surface)?, last_u_parameter(surface)?);
    Ok(ranged_samples(count, (u1, u2), whole))
}
/// `NbSamplesV(S, v1, v2)`; `nb_samples_v_range(&s, 0.0, 0.5)?`.
pub fn nb_samples_v_range(
    surface: &Adaptor3dSurfaceHandle,
    v1: f64,
    v2: f64,
) -> Result<i32, Failure> {
    let count: i32 = nb_samples_v(surface)?;
    if count <= 10 {
        return Ok(count);
    }
    let whole = (first_v_parameter(surface)?, last_v_parameter(surface)?);
    Ok(ranged_samples(count, (v1, v2), whole))
}

/// `IsSurfG1`: whether the B-spline curve that governs the continuity `along_u`
/// (the middle iso of a B-spline surface, or the B-spline basis of the swept surface
/// of that direction, through an offset surface and an offset basis curve) is G1
/// over the surface range with `ang_tol`; false when there is no such curve;
/// `is_surf_g1(&s, true, Precision::Angular())?`.
pub fn is_surf_g1(
    surface: &Adaptor3dSurfaceHandle,
    along_u: bool,
    ang_tol: f64,
) -> Result<bool, Failure> {
    let bounds = (
        surface.first_u_parameter()?,
        surface.last_u_parameter()?,
        surface.first_v_parameter()?,
        surface.last_v_parameter()?,
    );
    let mut governed: Adaptor3dSurfaceHandle = surface.clone();
    if governed.get_type()? == SurfaceType::OffsetSurface {
        governed = governed.basis_surface()?;
    }
    let mut curve: Handle<GeomBSplineCurve> = swept_bspline(&governed, along_u)?;
    if governed.get_type()? == SurfaceType::BSplineSurface {
        curve = middle_iso(&governed, along_u, bounds)?;
    }
    let Some(bspline) = curve.get() else {
        return Ok(false);
    };
    if along_u {
        return bspline.is_g1(bounds.0, bounds.1, ang_tol);
    }
    bspline.is_g1(bounds.2, bounds.3, ang_tol)
}

// The B-spline basis of a swept surface (an offset basis curve replaced by an adaptor
// on its whole basis), when the sweep governs the direction: extrusion along U,
// revolution along V. Null otherwise.
fn swept_bspline(
    surface: &Adaptor3dSurfaceHandle,
    along_u: bool,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let kind: SurfaceType = surface.get_type()?;
    if kind != SurfaceType::SurfaceOfRevolution && kind != SurfaceType::SurfaceOfExtrusion {
        return Ok(Handle::default());
    }
    let mut curve: Handle<dyn Adaptor3dCurve> = surface.basis_curve()?;
    if curve.is_null() {
        return Ok(Handle::default());
    }
    if curve.get_type()? == CurveType::OffsetCurve {
        let offset: Handle<GeomOffsetCurve> = curve.offset_curve()?;
        curve = Handle::new(GeomAdaptorCurve::with_curve(&offset.basis_curve())?).cast();
    }
    let governs: bool = (along_u && kind == SurfaceType::SurfaceOfExtrusion)
        || (!along_u && kind == SurfaceType::SurfaceOfRevolution);
    if curve.get_type()? == CurveType::BSplineCurve && governs {
        return curve.bspline();
    }
    Ok(Handle::default())
}

// The iso of a B-spline surface at the middle of the other direction's range.
fn middle_iso(
    surface: &Adaptor3dSurfaceHandle,
    along_u: bool,
    (u_first, u_last, v_first, v_last): (f64, f64, f64, f64),
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let bspline: Handle<GeomBSplineSurface> = surface.bspline()?;
    let iso: Handle<dyn GeomCurve> = if along_u {
        bspline.v_iso((v_first + v_last) / 2.0)?
    } else {
        bspline.u_iso((u_first + u_last) / 2.0)?
    };
    Ok(iso.cast())
}
