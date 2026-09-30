// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of AdjustUFirst, IsSeamParameter, AdjustU, Correct2DBounds,
// AdjustLine and InsertSeamVertices of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The U parameter of the quadric in `[0, 2 PI]` and the vertices on its seam.
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use std::f64::consts::PI;

const TWO_PI: f64 = 2.0 * PI;

/// `U` in `[0, 2 PI]` from `u2` (reduced): 0 or 2 PI, whichever is nearer.
fn nearer_seam_end(u2: f64) -> f64 {
    let mut uu: f64 = u2;
    if !(u2 > 0.0 && u2 < TWO_PI) {
        if u2 > TWO_PI {
            while uu > TWO_PI {
                uu -= TWO_PI;
            }
        } else {
            while uu < 0.0 {
                uu += TWO_PI;
            }
        }
    }
    if uu < (TWO_PI - uu) { 0.0 } else { TWO_PI }
}

/// `AdjustUFirst(U1, U2)`: the U of the start point of a line on the quadric (0 and
/// 2 PI chosen by the next point `u2`, else brought into `[0, 2 PI]`).
pub(super) fn adjust_u_first(u1: f64, u2: f64) -> f64 {
    let mut u: f64 = u1;
    if u1 > 0.0 && u1 < TWO_PI {
        return u;
    }
    if u1 == 0.0 || u1.abs() <= 1.0e-9 || u1 == TWO_PI || (TWO_PI - u1.abs()).abs() <= 1.0e-9 {
        return nearer_seam_end(u2);
    }
    if u1 < 0.0 {
        while u < 0.0 {
            u += 2.0 * PI;
        }
    }
    if u1 > TWO_PI {
        while u > TWO_PI {
            u -= TWO_PI;
        }
    }
    u
}

/// `IsSeamParameter(U, TOL2D)`.
pub(super) fn is_seam_parameter(u: f64, tol_2d: f64) -> bool {
    u.abs() <= tol_2d || (TWO_PI - u).abs() <= tol_2d
}

/// `AdjustU(U)`: `U` brought into `[0, 2 PI]`.
pub(super) fn adjust_u(mut u: f64) -> f64 {
    if u < 0.0 {
        while u < 0.0 {
            u += TWO_PI;
        }
    } else if u > TWO_PI {
        while u > TWO_PI {
            u -= TWO_PI;
        }
    }
    u
}

/// `Correct2DBounds(UF, UL, VF, VL, TOL2D, U, V)`: a parameter within `tol_2d` (and
/// above 1e-16) of a bound takes it.
fn correct_2d_bounds(
    (uf, ul, vf, vl): (f64, f64, f64, f64),
    tol_2d: f64,
    u: &mut f64,
    v: &mut f64,
) {
    let eps: f64 = 1.0e-16;
    let (duf, dul, dvf, dvl) = (
        (*u - uf).abs(),
        (*u - ul).abs(),
        (*v - vf).abs(),
        (*v - vl).abs(),
    );
    if duf <= tol_2d && duf > eps {
        *u = uf;
    }
    if dul <= tol_2d && dul > eps {
        *u = ul;
    }
    if dvf <= tol_2d && dvf > eps {
        *v = vf;
    }
    if dvl <= tol_2d && dvl > eps {
        *v = vl;
    }
}

/// `AdjustLine(Line, IsReversed, QSurf, TOL2D)`: the quadric parameters of every point
/// in `[0, 2 PI]` and snapped to the bounds.
pub(super) fn adjust_line(
    line: &IntSurfLineOn2S,
    is_reversed: bool,
    q_surf: &Adaptor3dSurfaceHandle,
    tol_2d: f64,
) -> Result<(), Failure> {
    let bounds = (
        q_surf.first_u_parameter()?,
        q_surf.last_u_parameter()?,
        q_surf.first_v_parameter()?,
        q_surf.last_v_parameter()?,
    );
    let bounds = (bounds.0, bounds.1, bounds.2, bounds.3);
    for ip in 1..=line.nb_points() {
        let (mut u, mut v) = line.value(ip)?.parameters_on_surface(!is_reversed);
        u = adjust_u(u);
        correct_2d_bounds(bounds, tol_2d, &mut u, &mut v);
        line.set_uv(ip, !is_reversed, u, v)?;
    }
    Ok(())
}

/// A point with the quadric U `nu` (and V `v`) and the parametric parameters of the
/// point `ip`.
fn seam_point(
    line: &IntSurfLineOn2S,
    ap: &IntSurfPntOn2S,
    ip: i32,
    (nu, v): (f64, f64),
    is_reversed: bool,
) -> Result<IntSurfPntOn2S, Failure> {
    let mut np = IntSurfPntOn2S::new();
    np.set_point(ap.value());
    let (u3, v3) = line.value(ip)?.parameters_on_surface(is_reversed);
    np.set_parameters_on(!is_reversed, nu, v);
    np.set_parameters_on(is_reversed, u3, v3);
    Ok(np)
}

/// `InsertSeamVertices(Line, IsReversed, Vertices, PTypes, TOL2D)`: at the first vertex
/// point on the seam: an end point jumps to the other seam end when the next point is
/// on the other side; an inner point gets a copy on the other seam end (after it when
/// both neighbours are on its side, else before it), also added to the vertices.
/// Returns whether a point was inserted.
pub(super) fn insert_seam_vertices(
    line: &IntSurfLineOn2S,
    is_reversed: bool,
    vertices: &IntSurfLineOn2S,
    ptypes: &[i32],
    tol_2d: f64,
) -> Result<bool, Failure> {
    let nbp: i32 = line.nb_points();
    for ip in 1..=nbp {
        if ptypes[ip as usize] == 0 {
            continue;
        }
        let ap: IntSurfPntOn2S = line.value(ip)?;
        let (u, v) = ap.parameters_on_surface(!is_reversed);
        let u: f64 = adjust_u(u);
        if !is_seam_parameter(u, tol_2d) {
            continue;
        }
        if ip == 1 || ip == nbp {
            end_on_seam(line, &ap, (ip, nbp), (u, v), is_reversed)?;
            continue;
        }
        if inner_on_seam(line, vertices, &ap, ip, (u, v, tol_2d), is_reversed)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// An end point on the seam: moved to the other seam end when the neighbour says so.
fn end_on_seam(
    line: &IntSurfLineOn2S,
    _ap: &IntSurfPntOn2S,
    (ip, nbp): (i32, i32),
    (u, v): (f64, f64),
    is_reversed: bool,
) -> Result<(), Failure> {
    let ipp: i32 = if ip == 1 { ip + 1 } else { ip - 1 };
    let (u1, _) = line.value(ipp)?.parameters_on_surface(!is_reversed);
    let nu: f64 = adjust_u_first(u, u1);
    if (nu - u).abs() >= 1.5 * PI {
        let (u2, v2) = line.value(ip)?.parameters_on_surface(is_reversed);
        line.set_uv(ip, !is_reversed, nu, v)?;
        line.set_uv(ip, is_reversed, u2, v2)?;
    }
    let _ = nbp;
    Ok(())
}

/// An inner point on the seam: true when its copy was inserted.
fn inner_on_seam(
    line: &IntSurfLineOn2S,
    vertices: &IntSurfLineOn2S,
    ap: &IntSurfPntOn2S,
    ip: i32,
    (u, v, tol_2d): (f64, f64, f64),
    is_reversed: bool,
) -> Result<bool, Failure> {
    let (ipp, ipn) = (ip - 1, ip + 1);
    let (u1, _) = line.value(ipp)?.parameters_on_surface(!is_reversed);
    let (u2, _) = line.value(ipn)?.parameters_on_surface(!is_reversed);
    let (u1, u2) = (adjust_u(u1), adjust_u(u2));
    let pnear_zero: bool = u1.abs() < (TWO_PI - u1).abs();
    let cnear_zero: bool = u.abs() < (TWO_PI - u).abs();
    if is_seam_parameter(u2, tol_2d) || is_seam_parameter(u1, tol_2d) {
        return Ok(false);
    }
    let nu: f64 = if cnear_zero { TWO_PI } else { 0.0 };
    let np: IntSurfPntOn2S = seam_point(line, ap, ip, (nu, v), is_reversed)?;
    line.insert_before(if pnear_zero == cnear_zero { ipn } else { ip }, &np)?;
    vertices.add(&np);
    Ok(true)
}
