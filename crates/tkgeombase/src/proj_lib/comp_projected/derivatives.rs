// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of d1, d2 and d2CurvOnSurf (ProjLib_CompProjectedCurve.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The derivatives of the projected curve `t -> (u(t), v(t))` defined implicitly by
//! `E(t, u, v) = ((S - C).Su, (S - C).Sv) = 0`: `X' = -(dE/dX)^-1 dE/dt`, and the
//! second derivative from the derivative of that system.
use crate::native_failure::failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomSurfaceD2, GeomSurfaceD3};
use rustycad_tkmath::gp::{self, Point3, Vector2, Vector3, Xy};

/// The curve and surface of the projection.
#[derive(Clone, Copy)]
pub(super) struct Pair<'a> {
    pub(super) curve: &'a dyn Adaptor3dCurve,
    pub(super) surface: &'a dyn Adaptor3dSurface,
}

/// The rows of `(dE/dX)^-1` (the native gp_Mat2d built from its columns).
struct InverseRows {
    row1: Vector2,
    row2: Vector2,
}

impl InverseRows {
    // `-(M.Row(1) . w, M.Row(2) . w)`.
    fn solve(&self, w: Vector2) -> Vector2 {
        -Vector2::new(self.row1 * w, self.row2 * w)
    }
}

// dE/du and dE/dv, then the inverse; Standard_ConstructionError when singular.
fn inverse(
    s_d1u: Vector3,
    s_d1v: Vector3,
    ort: Vector3,
    d2: [Vector3; 3],
) -> Result<InverseRows, Failure> {
    let [d2u, d2v, d2uv] = d2;
    let de_du: Xy = Xy::new(s_d1u * s_d1u + ort * d2u, s_d1u * s_d1v + ort * d2uv);
    let de_dv: Xy = Xy::new(s_d1v * s_d1u + ort * d2uv, s_d1v * s_d1v + ort * d2v);
    let det: f64 = de_du.x() * de_dv.y() - de_du.y() * de_dv.x();
    if det.abs() < gp::resolution() {
        return Err(failure(
            "Standard_ConstructionError",
            &format!(
                "ProjLib_CompProjectedCurve: singular projection Jacobian (determinant {det})"
            ),
        ));
    }
    Ok(InverseRows {
        row1: Vector2::new(de_dv.y() / det, -de_dv.x() / det),
        row2: Vector2::new(-de_du.y() / det, de_du.x() / det),
    })
}

/// `d1(t, u, v, V, Curve, Surface)`: the first derivative of the projection.
pub(super) fn d1(t: f64, u: f64, v: f64, pair: Pair<'_>) -> Result<Vector2, Failure> {
    let s: GeomSurfaceD2 = pair.surface.d2(u, v)?;
    let (c, dc1_t): (Point3, Vector3) = pair.curve.d1(t)?;
    let ort: Vector3 = Vector3::between(c, s.point);
    let de_dt: Vector2 = Vector2::new(-dc1_t * s.d1u, -dc1_t * s.d1v);
    let m: InverseRows = inverse(s.d1u, s.d1v, ort, [s.d2u, s.d2v, s.d2uv])?;
    Ok(m.solve(de_dt))
}

/// The first and second derivatives of the projection (`d2`) and the surface
/// data they come from.
struct Second {
    v1: Vector2,
    v2: Vector2,
    s: GeomSurfaceD3,
}

fn second(t: f64, u: f64, v: f64, pair: Pair<'_>) -> Result<Second, Failure> {
    let s: GeomSurfaceD3 = pair.surface.d3(u, v)?;
    let (c, dc1_t, dc2_t) = pair.curve.d2(t)?;
    let ort: Vector3 = Vector3::between(c, s.point);
    let de_dt: Vector2 = Vector2::new(-dc1_t * s.d1u, -dc1_t * s.d1v);
    let m: InverseRows = inverse(s.d1u, s.d1v, ort, [s.d2u, s.d2v, s.d2uv])?;
    let v1: Vector2 = m.solve(de_dt);
    // d2E/dt2 = S1.
    let d2e_dt: Vector2 = Vector2::new(-dc2_t * s.d1u, -dc2_t * s.d1v);
    // 2 (d2E/dtdX)(dX/dt) = S2.
    let d2e1_dtdx: Vector2 = Vector2::new(-dc1_t * s.d2u, -dc1_t * s.d2uv);
    let d2e2_dtdx: Vector2 = Vector2::new(-dc1_t * s.d2uv, -dc1_t * s.d2v);
    let s2: Vector2 = 2. * Vector2::new(d2e1_dtdx * v1, d2e2_dtdx * v1);
    let s3: Vector2 = hessian_term(&s, ort, v1);
    let sum: Vector2 = d2e_dt + s2 + s3;
    Ok(Second {
        v1,
        v2: m.solve(sum),
        s,
    })
}

// (d2E/dX2)(dX/dt)^2 = S3, from the rows of the second derivatives of E1 and E2.
fn hessian_term(s: &GeomSurfaceD3, ort: Vector3, v1: Vector2) -> Vector2 {
    let tmp1: f64 = 2. * s.d1u * s.d2uv + s.d1v * s.d2u + ort * s.d3uuv;
    let row11: Vector2 = Vector2::new(3. * s.d1u * s.d2u + ort * s.d3u, tmp1);
    let row12: Vector2 = Vector2::new(tmp1, s.d2v * s.d1u + 2. * s.d1v * s.d2uv + ort * s.d3uvv);
    let tmp2: f64 = 2. * s.d2uv * s.d1v + s.d1u * s.d2v + ort * s.d3uvv;
    let row21: Vector2 = Vector2::new(s.d2u * s.d1v + 2. * s.d1u * s.d2uv + ort * s.d3uuv, tmp2);
    let row22: Vector2 = Vector2::new(tmp2, 3. * s.d1v * s.d2v + ort * s.d3v);
    Vector2::new(
        v1 * Vector2::new(row11 * v1, row12 * v1),
        v1 * Vector2::new(row21 * v1, row22 * v1),
    )
}

/// `d2(t, u, v, V1, V2, Curve, Surface)`: the first and second derivatives.
pub(super) fn d2(t: f64, u: f64, v: f64, pair: Pair<'_>) -> Result<(Vector2, Vector2), Failure> {
    let result: Second = second(t, u, v, pair)?;
    Ok((result.v1, result.v2))
}

/// `d2CurvOnSurf(t, u, v, V1, V2, Curve, Surface)`: the first and second
/// derivatives of the projected curve in 3D, `S(u(t), v(t))`.
pub(super) fn d2_curv_on_surf(
    t: f64,
    u: f64,
    v: f64,
    pair: Pair<'_>,
) -> Result<(Vector3, Vector3), Failure> {
    let Second {
        v1: v12d,
        v2: v22d,
        s,
    } = second(t, u, v, pair)?;
    let v1: Vector3 = s.d1u * v12d.x() + s.d1v * v12d.y();
    let v2: Vector3 = s.d2u * v12d.x() * v12d.x()
        + s.d1u * v22d.x()
        + 2. * s.d2uv * v12d.x() * v12d.y()
        + s.d2v * v12d.y() * v12d.y()
        + s.d1v * v22d.y();
    Ok((v1, v2))
}
