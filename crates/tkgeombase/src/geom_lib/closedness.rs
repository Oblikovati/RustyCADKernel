// Created on: 1993-07-07
// Created by: Jean Claude VAUTHIER
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (IsClosed,
// IsBSplUClosed, IsBSplVClosed, IsBzUClosed, IsBzVClosed, CompareWeightPoles)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Whether the opposite boundaries of a surface coincide.
use super::live;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCurve,
    GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Xyz};

// Geometric sampling of the closedness test of free-form surfaces.
const THE_CLOSEDNESS_SAMPLE_COUNT: i32 = 23;

/// `GeomLib::IsClosed(S, Tol, isUClosed, isVClosed)`: whether the U-opposite and the
/// V-opposite boundaries of `s` coincide within `tol`, as `(u_closed, v_closed)`.
///
/// A plane is never closed; a cylinder, cone, sphere or extrusion compares the two U
/// boundaries at one V (a cone at the V farthest from the apex, a sphere at the
/// equator when the range crosses it); a torus compares its ranges with the periods;
/// B-spline and Bezier surfaces compare their boundary poles; revolutions, offset and
/// other surfaces sample 23 points along each boundary (infinite bounds replaced by
/// +-1); `let (u_closed, v_closed) = geom_lib::is_closed(&surface, 1e-7)?;`.
pub fn is_closed(s: &Handle<dyn GeomSurface>, tol: f64) -> Result<(bool, bool), Failure> {
    let gas = GeomAdaptorSurface::with_surface(s)?;
    let (u1, u2) = (gas.first_u_parameter()?, gas.last_u_parameter()?);
    let (v1, v2) = (gas.first_v_parameter()?, gas.last_v_parameter()?);
    let tol2: f64 = tol * tol;
    let u_closed_at = |v: f64| -> Result<bool, Failure> {
        Ok(gas.eval_d0(u1, v)?.square_distance(&gas.eval_d0(u2, v)?) <= tol2)
    };
    match gas.get_type()? {
        SurfaceType::Plane => Ok((false, false)),
        SurfaceType::SurfaceOfExtrusion
            if precision::is_infinite(u1) || precision::is_infinite(u2) =>
        {
            // Not closed.
            Ok((false, false))
        }
        SurfaceType::SurfaceOfExtrusion | SurfaceType::Cylinder => {
            let v: f64 = if precision::is_infinite(v1) { 0.0 } else { v1 };
            Ok((u_closed_at(v)?, false))
        }
        SurfaceType::Cone => Ok((u_closed_at(cone_v(&gas, [u1, v1, v2])?)?, false)),
        SurfaceType::Sphere => Ok((u_closed_at(sphere_v(v1, v2))?, false)),
        SurfaceType::Torus => {
            let (ures, vres) = (gas.u_resolution(tol)?, gas.v_resolution(tol)?);
            Ok((
                (u2 - u1) >= gas.u_period()? - ures,
                (v2 - v1) >= gas.v_period()? - vres,
            ))
        }
        SurfaceType::BSplineSurface => {
            let bspl: Handle<GeomBSplineSurface> = gas.bspline()?;
            Ok((
                is_bspl_u_closed(&bspl, u1, u2, tol)?,
                is_bspl_v_closed(&bspl, v1, v2, tol)?,
            ))
        }
        SurfaceType::BezierSurface => {
            let bz: Handle<GeomBezierSurface> = gas.bezier()?;
            Ok((
                is_bz_u_closed(&bz, u1, u2, tol)?,
                is_bz_v_closed(&bz, v1, v2, tol)?,
            ))
        }
        kind @ (SurfaceType::SurfaceOfRevolution
        | SurfaceType::OffsetSurface
        | SurfaceType::OtherSurface) => sampled_closedness(&gas, kind, [u1, u2, v1, v2], tol),
    }
}

// The cone: the V of the boundary farthest from the apex, 0 for an infinite range.
fn cone_v(gas: &GeomAdaptorSurface, [u1, v1, v2]: [f64; 3]) -> Result<f64, Failure> {
    if precision::is_infinite(v1) || precision::is_infinite(v2) {
        return Ok(0.0);
    }
    let an_apex: Point3 = gas.cone()?.apex();
    let p1: Point3 = gas.eval_d0(u1, v1)?;
    let p2: Point3 = gas.eval_d0(u1, v2)?;
    Ok(
        if p2.square_distance(&an_apex) > p1.square_distance(&an_apex) {
            v2
        } else {
            v1
        },
    )
}

// The sphere: the equator when the V range crosses it, else the bound nearest to it.
fn sphere_v(v1: f64, v2: f64) -> f64 {
    if v1 * v2 <= 0.0 {
        return 0.0;
    }
    if v1 < 0.0 { v2 } else { v1 }
}

// The sampling test of revolutions, offset and other surfaces.
fn sampled_closedness(
    gas: &GeomAdaptorSurface,
    kind: SurfaceType,
    [mut u1, mut u2, mut v1, mut v2]: [f64; 4],
    tol: f64,
) -> Result<(bool, bool), Failure> {
    let unit = |t: f64| {
        if precision::is_infinite(t) {
            1.0f64.copysign(t)
        } else {
            t
        }
    };
    (v1, v2) = (unit(v1), unit(v2));
    if kind == SurfaceType::OffsetSurface || kind == SurfaceType::OtherSurface {
        (u1, u2) = (unit(u1), unit(u2));
    }
    let tol2: f64 = tol * tol;
    let u_res: f64 = std_max(gas.u_resolution(tol)?, precision::p_confusion());
    let is_u_closed = sampled_match([v1, v2], u_res, |t| {
        Ok(gas.eval_d0(u1, t)?.square_distance(&gas.eval_d0(u2, t)?) <= tol2)
    })?;
    let v_res: f64 = std_max(gas.v_resolution(tol)?, precision::p_confusion());
    let is_v_closed = sampled_match([u1, u2], v_res, |t| {
        Ok(gas.eval_d0(t, v1)?.square_distance(&gas.eval_d0(t, v2)?) <= tol2)
    })?;
    Ok((is_u_closed, is_v_closed))
}

// Whether `matches` holds at 23 samples of [first, last] (fewer when the step is
// below `res`, at least 2), the last one exactly at `last`.
fn sampled_match(
    [first, last]: [f64; 2],
    res: f64,
    matches: impl Fn(f64) -> Result<bool, Failure>,
) -> Result<bool, Failure> {
    let mut nbp: i32 = THE_CLOSEDNESS_SAMPLE_COUNT;
    let mut dt: f64 = (last - first) / f64::from(nbp - 1);
    if dt <= res {
        nbp = real_to_int((last - first) / (2.0 * res)) + 1;
        nbp = nbp.max(2);
        dt = (last - first) / f64::from(nbp - 1);
    }
    for i in 0..nbp {
        let t: f64 = if i == nbp - 1 {
            last
        } else {
            first + f64::from(i) * dt
        };
        if !matches(t)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `GeomLib::IsBSplUClosed(S, U1, U2, Tol)`: whether the poles of the U1 and U2
/// isolines are equal within `2 Tol`; for rational surfaces the homogeneous poles
/// normalized by the first weight of each isoline are compared;
/// `geom_lib::is_bspl_u_closed(&surface, 0.0, 1.0, 1e-7)?`.
pub fn is_bspl_u_closed(
    s: &Handle<GeomBSplineSurface>,
    u1: f64,
    u2: f64,
    tol: f64,
) -> Result<bool, Failure> {
    let surface: &GeomBSplineSurface = live(s, "GeomLib::IsBSplUClosed")?;
    compare_bspline_isos(&surface.u_iso(u1)?, &surface.u_iso(u2)?, tol)
}

/// `GeomLib::IsBSplVClosed(S, V1, V2, Tol)`: the V twin of [`is_bspl_u_closed`];
/// `geom_lib::is_bspl_v_closed(&surface, 0.0, 1.0, 1e-7)?`.
pub fn is_bspl_v_closed(
    s: &Handle<GeomBSplineSurface>,
    v1: f64,
    v2: f64,
    tol: f64,
) -> Result<bool, Failure> {
    let surface: &GeomBSplineSurface = live(s, "GeomLib::IsBSplVClosed")?;
    compare_bspline_isos(&surface.v_iso(v1)?, &surface.v_iso(v2)?, tol)
}

/// `GeomLib::IsBzUClosed(S, U1, U2, Tol)`: [`is_bspl_u_closed`] for a Bezier surface;
/// `geom_lib::is_bz_u_closed(&surface, 0.0, 1.0, 1e-7)?`.
pub fn is_bz_u_closed(
    s: &Handle<GeomBezierSurface>,
    u1: f64,
    u2: f64,
    tol: f64,
) -> Result<bool, Failure> {
    let surface: &GeomBezierSurface = live(s, "GeomLib::IsBzUClosed")?;
    compare_bezier_isos(&surface.u_iso(u1)?, &surface.u_iso(u2)?, tol)
}

/// `GeomLib::IsBzVClosed(S, V1, V2, Tol)`: the V twin of [`is_bz_u_closed`];
/// `geom_lib::is_bz_v_closed(&surface, 0.0, 1.0, 1e-7)?`.
pub fn is_bz_v_closed(
    s: &Handle<GeomBezierSurface>,
    v1: f64,
    v2: f64,
    tol: f64,
) -> Result<bool, Failure> {
    let surface: &GeomBezierSurface = live(s, "GeomLib::IsBzVClosed")?;
    compare_bezier_isos(&surface.v_iso(v1)?, &surface.v_iso(v2)?, tol)
}

// The isolines are B-splines natively (down_cast): their poles and weights.
fn compare_bspline_isos(
    first: &Handle<dyn GeomCurve>,
    last: &Handle<dyn GeomCurve>,
    tol: f64,
) -> Result<bool, Failure> {
    if first.is_null() || last.is_null() {
        return Ok(false);
    }
    let (bs_f, bs_l): (Handle<GeomBSplineCurve>, Handle<GeomBSplineCurve>) =
        (first.cast(), last.cast());
    let (bs_f, bs_l) = (live(&bs_f, "IsBSplClosed")?, live(&bs_l, "IsBSplClosed")?);
    compare_weight_poles(
        (&bs_f.poles(), bs_f.weights().as_ref()),
        (&bs_l.poles(), bs_l.weights().as_ref()),
        2.0 * tol,
    )
}

fn compare_bezier_isos(
    first: &Handle<dyn GeomCurve>,
    last: &Handle<dyn GeomCurve>,
    tol: f64,
) -> Result<bool, Failure> {
    if first.is_null() || last.is_null() {
        return Ok(false);
    }
    let (bz_f, bz_l): (Handle<GeomBezierCurve>, Handle<GeomBezierCurve>) =
        (first.cast(), last.cast());
    let (bz_f, bz_l) = (live(&bz_f, "IsBzClosed")?, live(&bz_l, "IsBzClosed")?);
    compare_weight_poles(
        (&bz_f.poles(), bz_f.weights().as_ref()),
        (&bz_l.poles(), bz_l.weights().as_ref()),
        2.0 * tol,
    )
}

// `CompareWeightPoles`: rational control points are projective, so both boundaries
// are normalized by their first weights and all four homogeneous coordinates are
// compared within `tol`.
fn compare_weight_poles(
    (poles1, w1): (&Array1<'static, Point3>, Option<&Array1<'static, f64>>),
    (poles2, w2): (&Array1<'static, Point3>, Option<&Array1<'static, f64>>),
    tol: f64,
) -> Result<bool, Failure> {
    if poles1.length() != poles2.length() {
        return Ok(false);
    }
    let weight = |w: Option<&Array1<'static, f64>>, index: usize| -> f64 {
        w.map_or(1.0, |values| values.as_slice()[index])
    };
    let (reference1, reference2): (f64, f64) = (weight(w1, 0), weight(w2, 0));
    for (index, (p1, p2)) in poles1.iter().zip(poles2.iter()).enumerate() {
        let a_w1: f64 = weight(w1, index) / reference1;
        let a_w2: f64 = weight(w2, index) / reference2;
        let pole1: Xyz = p1.xyz() * a_w1;
        let pole2: Xyz = p2.xyz() * a_w2;
        if (a_w1 - a_w2).abs() > tol || !pole1.is_equal(&pole2, tol) {
            return Ok(false);
        }
    }
    Ok(true)
}
