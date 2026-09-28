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
// Rust translation of ModelingData/TKG3d/Geom/Geom_ExtrusionUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_ExtrusionUtils`: the evaluation of a surface of linear extrusion
//! `P(u, v) = C(u) + v * Dir` from the basis curve values, shared by
//! Geom_SurfaceOfLinearExtrusion and the adaptors.
//!
//! The native templates over the curve type become functions generic over the
//! curve trait; the `Calculate*` functions work on already evaluated values.
use super::curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use super::surface::{GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

fn zero() -> Vector3 {
    Vector3::new(0.0, 0.0, 0.0)
}

/// `CalculateD0`: the curve point moved by `v * dir`; `calculate_d0(p, 2.0, dir)`.
pub fn calculate_d0(curve_point: Point3, v: f64, dir: Xyz) -> Point3 {
    Point3::from(curve_point.xyz() + v * dir)
}

/// `CalculateD1`: D1U is the curve derivative and D1V the direction;
/// `calculate_d1(&basis_d1, 2.0, dir)`.
pub fn calculate_d1(curve: &GeomCurveD1, v: f64, dir: Xyz) -> GeomSurfaceD1 {
    GeomSurfaceD1 {
        point: calculate_d0(curve.point, v, dir),
        d1u: curve.d1,
        d1v: Vector3::from(dir),
    }
}

/// `CalculateD2`: the only non-null second derivative is D2U, the curve one;
/// `calculate_d2(&basis_d2, 2.0, dir)`.
pub fn calculate_d2(curve: &GeomCurveD2, v: f64, dir: Xyz) -> GeomSurfaceD2 {
    GeomSurfaceD2 {
        point: calculate_d0(curve.point, v, dir),
        d1u: curve.d1,
        d1v: Vector3::from(dir),
        d2u: curve.d2,
        d2v: zero(),
        d2uv: zero(),
    }
}

/// `CalculateD3`: D3U is the curve third derivative, the other third ones are null;
/// `calculate_d3(&basis_d3, 2.0, dir)`.
pub fn calculate_d3(curve: &GeomCurveD3, v: f64, dir: Xyz) -> GeomSurfaceD3 {
    GeomSurfaceD3 {
        point: calculate_d0(curve.point, v, dir),
        d1u: curve.d1,
        d1v: Vector3::from(dir),
        d2u: curve.d2,
        d2v: zero(),
        d2uv: zero(),
        d3u: curve.d3,
        d3v: zero(),
        d3uuv: zero(),
        d3uvv: zero(),
    }
}

/// `CalculateDN`: the curve derivative for `DerV == 0`, the direction for
/// `(0, 1)`, null otherwise; `calculate_dn(curve_dn, dir, 2, 0)`.
pub fn calculate_dn(curve_dn: Vector3, dir: Xyz, der_u: i32, der_v: i32) -> Vector3 {
    if der_v == 0 {
        return curve_dn;
    }
    if der_u == 0 && der_v == 1 {
        return Vector3::from(dir);
    }
    zero()
}

/// `D0<CurveType>`: evaluate the basis at `u`, then CalculateD0; `d0(u, v, &*basis, dir)?`.
pub fn d0<C: GeomCurve + ?Sized>(u: f64, v: f64, basis: &C, dir: Xyz) -> Result<Point3, Failure> {
    Ok(calculate_d0(basis.eval_d0(u)?, v, dir))
}

/// `D1<CurveType>`; `d1(u, v, &*basis, dir)?`.
pub fn d1<C: GeomCurve + ?Sized>(
    u: f64,
    v: f64,
    basis: &C,
    dir: Xyz,
) -> Result<GeomSurfaceD1, Failure> {
    Ok(calculate_d1(&basis.eval_d1(u)?, v, dir))
}

/// `D2<CurveType>`; `d2(u, v, &*basis, dir)?`.
pub fn d2<C: GeomCurve + ?Sized>(
    u: f64,
    v: f64,
    basis: &C,
    dir: Xyz,
) -> Result<GeomSurfaceD2, Failure> {
    Ok(calculate_d2(&basis.eval_d2(u)?, v, dir))
}

/// `D3<CurveType>`; `d3(u, v, &*basis, dir)?`.
pub fn d3<C: GeomCurve + ?Sized>(
    u: f64,
    v: f64,
    basis: &C,
    dir: Xyz,
) -> Result<GeomSurfaceD3, Failure> {
    Ok(calculate_d3(&basis.eval_d3(u)?, v, dir))
}

/// `DN<CurveType>`: the basis derivative of order `der_u` is evaluated only when
/// `der_v == 0`; `dn(u, &*basis, dir, 2, 0)?`.
pub fn dn<C: GeomCurve + ?Sized>(
    u: f64,
    basis: &C,
    dir: Xyz,
    der_u: i32,
    der_v: i32,
) -> Result<Vector3, Failure> {
    let curve_dn: Vector3 = if der_v == 0 {
        basis.eval_dn(u, der_u)?
    } else {
        Vector3::default()
    };
    Ok(calculate_dn(curve_dn, dir, der_u, der_v))
}
