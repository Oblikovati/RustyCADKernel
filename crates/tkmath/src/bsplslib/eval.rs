// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (D0, HomogeneousD0, D1, HomogeneousD1, D2) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Point and first/second derivative evaluation of B-spline surfaces.
//!
//! The first-direction scheme runs on rows of `(d2+1)` poles, then each resulting
//! row runs the second-direction scheme; outputs are picked from the patch buffer
//! (or from the rational derivatives) according to which direction came first.
use super::prepare::{Patch, SurfaceSpan, prepare_eval, validate_degrees};
use super::rational::rational_derivative;
use crate::bsplclib::{bohm, eval};
use crate::gp::{Point3, Vector3};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;

/// Point and first and second derivatives (`BSplSLib::D2` outputs, in native order).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfaceD2 {
    /// Point.
    pub p: Point3,
    /// dP/dU.
    pub vu: Vector3,
    /// dP/dV.
    pub vv: Vector3,
    /// d2P/dU2.
    pub vuu: Vector3,
    /// d2P/dV2.
    pub vvv: Vector3,
    /// d2P/dUdV.
    pub vuv: Vector3,
}

pub(crate) fn point(values: &[f64]) -> Point3 {
    Point3::new(values[0], values[1], values[2])
}

pub(crate) fn vector(values: &[f64]) -> Vector3 {
    Vector3::new(values[0], values[1], values[2])
}

pub(crate) const ZERO: [f64; 3] = [0.0; 3];

// Runs Bohm or Eval in the first direction on whole rows.
pub(crate) fn first_direction(patch: &mut Patch, n: Option<i32>) -> Result<(), Failure> {
    let row: i32 = patch.dim() as i32 * (patch.d2 + 1);
    match n {
        Some(n) => bohm(patch.u1, patch.d1, n, &patch.knots1, row, &mut patch.poles),
        None => eval(patch.u1, patch.d1, &patch.knots1, row, &mut patch.poles),
    }
}

// Runs Bohm (Some(n)) or Eval (None) in the second direction on the row at `offset`.
pub(crate) fn second_direction(
    patch: &mut Patch,
    n: Option<i32>,
    offset: usize,
) -> Result<(), Failure> {
    let dim: i32 = patch.dim() as i32;
    let slice: &mut [f64] = &mut patch.poles[offset..];
    match n {
        Some(n) => bohm(patch.u2, patch.d2, n, &patch.knots2, dim, slice),
        None => eval(patch.u2, patch.d2, &patch.knots2, dim, slice),
    }
}

/// Homogeneous point and weight at (U, V) (`BSplSLib::HomogeneousD0`); returns `(W, P)`
/// with W = 1 for a (locally) non-rational patch;
/// `bsplslib::homogeneous_d0(&span, &poles, None)?`.
pub fn homogeneous_d0(
    span: &SurfaceSpan<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<(f64, Point3), Failure> {
    validate_degrees(span.u_degree, span.v_degree)?;
    let mut patch: Patch = prepare_eval(span, poles, weights)?;
    first_direction(&mut patch, None)?;
    second_direction(&mut patch, None, 0)?;
    let w: f64 = if patch.rational {
        patch.poles[3]
    } else {
        1.0e0
    };
    Ok((w, point(&patch.poles)))
}

/// Point at (U, V) (`BSplSLib::D0`): the homogeneous point divided by its weight;
/// `bsplslib::d0(&span, &poles, None)?`.
pub fn d0(
    span: &SurfaceSpan<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<Point3, Failure> {
    let (w, p) = homogeneous_d0(span, poles, weights)?;
    Ok(Point3::new(p.x() / w, p.y() / w, p.z() / w))
}

// Bohm to order 1 in the first direction, order 1 on row 0 and a value on row 1.
fn first_order_patch(patch: &mut Patch) -> Result<usize, Failure> {
    let row: usize = patch.dim() * (patch.d2 as usize + 1);
    first_direction(patch, Some(1))?;
    second_direction(patch, Some(1), 0)?;
    second_direction(patch, None, row)?;
    Ok(row)
}

/// Point and first derivatives (`BSplSLib::D1`); returns `(P, Vu, Vv)`;
/// `bsplslib::d1(&span, &poles, None)?`.
pub fn d1(
    span: &SurfaceSpan<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<(Point3, Vector3, Vector3), Failure> {
    validate_degrees(span.u_degree, span.v_degree)?;
    let mut patch: Patch = prepare_eval(span, poles, weights)?;
    let row: usize = first_order_patch(&mut patch)?;
    let (result, first, second): (Vec<f64>, usize, usize) = if patch.rational {
        let mut ders: Vec<f64> = vec![0.0; 48];
        rational_derivative(patch.d1, patch.d2, 1, 1, &patch.poles, &mut ders, true)?;
        (ders, 6, 3)
    } else {
        (patch.poles.clone(), row, 3)
    };
    // `first` is the derivative along the first direction.
    let (vu, vv): (usize, usize) = if patch.u_first {
        (first, second)
    } else {
        (second, first)
    };
    Ok((point(&result), vector(&result[vu..]), vector(&result[vv..])))
}

/// Homogeneous point and first derivatives with the weight derivatives
/// (`BSplSLib::HomogeneousD1`); returns `(N, Nu, Nv, D, Du, Dv)`, D = 1 and Du = Dv = 0
/// for a (locally) non-rational patch; `bsplslib::homogeneous_d1(&span, &poles, None)?`.
pub fn homogeneous_d1(
    span: &SurfaceSpan<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<(Point3, Vector3, Vector3, f64, f64, f64), Failure> {
    validate_degrees(span.u_degree, span.v_degree)?;
    let mut patch: Patch = prepare_eval(span, poles, weights)?;
    let row: usize = first_order_patch(&mut patch)?;
    let dim: usize = patch.dim();
    let (vu, vv): (usize, usize) = if patch.u_first {
        (row, dim)
    } else {
        (dim, row)
    };
    let result: &[f64] = &patch.poles;
    let (d, du, dv): (f64, f64, f64) = if patch.rational {
        (result[3], result[vu + 3], result[vv + 3])
    } else {
        (1.0e0, 0.0e0, 0.0e0)
    };
    Ok((
        point(result),
        vector(&result[vu..]),
        vector(&result[vv..]),
        d,
        du,
        dv,
    ))
}

/// Point and first and second derivatives (`BSplSLib::D2`). Non-rational patches of
/// degree <= 1 in a direction give zero second derivatives in it;
/// `bsplslib::d2(&span, &poles, None)?`.
pub fn d2(
    span: &SurfaceSpan<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<SurfaceD2, Failure> {
    validate_degrees(span.u_degree, span.v_degree)?;
    let mut patch: Patch = prepare_eval(span, poles, weights)?;
    let row: usize = patch.dim() * (patch.d2 as usize + 1);
    first_direction(&mut patch, Some(2))?;
    second_direction(&mut patch, Some(2), 0)?;
    second_direction(&mut patch, Some(1), row)?;
    if patch.d1 > 1 {
        second_direction(&mut patch, None, 2 * row)?;
    }
    if patch.rational {
        let mut ders: Vec<f64> = vec![0.0; 48];
        rational_derivative(patch.d1, patch.d2, 2, 2, &patch.poles, &mut ders, true)?;
        // (i, j) of the first/second direction sits at 9*i + 3*j.
        let [vu, vv, vuu, vvv] = oriented(patch.u_first, [9, 3, 18, 6]);
        return Ok(pick_d2(&ders, [0, vu, vv, vuu, vvv, 12], [false, false]));
    }
    let d2p: usize = patch.d2 as usize;
    let [vu, vv, vuu, vvv] = oriented(patch.u_first, [row, 3, 2 * row, 6]);
    let zero: [bool; 2] = [span.u_degree <= 1, span.v_degree <= 1];
    Ok(pick_d2(
        &patch.poles,
        [0, vu, vv, vuu, vvv, 3 * d2p + 6],
        zero,
    ))
}

// Maps [first, second, first-first, second-second] offsets to [u, v, uu, vv].
pub(crate) fn oriented(u_first: bool, [a, b, aa, bb]: [usize; 4]) -> [usize; 4] {
    if u_first {
        [a, b, aa, bb]
    } else {
        [b, a, bb, aa]
    }
}

// `zero` replaces the uu / vv derivatives by BSplSLib_zero.
fn pick_d2(values: &[f64], [p, vu, vv, vuu, vvv, vuv]: [usize; 6], zero: [bool; 2]) -> SurfaceD2 {
    SurfaceD2 {
        p: point(&values[p..]),
        vu: vector(&values[vu..]),
        vv: vector(&values[vv..]),
        vuu: if zero[0] {
            vector(&ZERO)
        } else {
            vector(&values[vuu..])
        },
        vvv: if zero[1] {
            vector(&ZERO)
        } else {
            vector(&values[vvv..])
        },
        vuv: vector(&values[vuv..]),
    }
}
