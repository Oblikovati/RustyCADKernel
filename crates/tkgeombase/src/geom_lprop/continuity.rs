// Created on: 1994-02-23
// Created by: Laurent BOURESCHE
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
// Rust translation of ModelingData/TKGeomBase/GeomLProp/GeomLProp.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLProp::Continuity`: the continuity of two curves at a junction point.
use super::CLProps;
use crate::lprop::curve_access::live;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Direction3, Vector3};

// GeomAbsToInteger: C0 0, G1 1, C1 2, G2 3, C2 4, C3 5, CN 6.
fn geom_abs_to_integer(shape: Shape) -> i32 {
    match shape {
        Shape::C0 => 0,
        Shape::G1 => 1,
        Shape::C1 => 2,
        Shape::G2 => 3,
        Shape::C2 => 4,
        Shape::C3 => 5,
        Shape::CN => 6,
    }
}

// The derivative level compared at the junction: 3 from C3, 2 for C2, 1 for C1,
// 0 otherwise (G1 and G2 compare nothing).
fn derivative_level(cont: i32) -> i32 {
    match cont {
        5.. => 3,
        4 => 2,
        2 => 1,
        _ => 0,
    }
}

// The local continuity at `u`: the global one, or for a B-spline (possibly under a
// trim) `Degree - Multiplicity` on an interior knot located within the parametric
// resolution of `tl`, else 5.
fn local_continuity(curve: &Handle<dyn GeomCurve>, u: f64, tl: f64) -> Result<i32, Failure> {
    let cont: i32 = geom_abs_to_integer(live(curve)?.continuity());
    let trimmed: Handle<GeomTrimmedCurve> = curve.cast();
    let basis: Handle<dyn GeomCurve> = match trimmed.get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => curve.clone(),
    };
    let bspline: Handle<GeomBSplineCurve> = basis.cast();
    let Some(bspline) = bspline.get() else {
        return Ok(cont);
    };
    let tolerance: f64 = bspline.resolution(tl)?;
    let (index1, index2) = bspline.locate_u(u, tolerance, false)?;
    if index1 > 1 && index2 < bspline.nb_knots() && index1 == index2 {
        return Ok(bspline.degree() - bspline.multiplicity(index1)?);
    }
    Ok(5)
}

/// `GeomLProp::Continuity(C1, C2, u1, u2, r1, r2, tl, ta)`: the continuity at the
/// junction of `c1` at `u1` and `c2` at `u2` (each reversed when `r1`/`r2`), with
/// the linear and angular tolerances `tl` and `ta`.
///
/// Standard_Failure ("Courbes non jointives") when the points differ by more than
/// `tl`. C1 when the first derivatives match, G1 when only the tangents do, C2 when
/// the second derivatives also match; the derivative levels follow each curve's
/// (local) continuity;
/// `continuity_with_tolerances(&c1, &c2, 1.0, 0.0, false, false, 1e-7, 1e-12)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument signature.
pub fn continuity_with_tolerances(
    c1: &Handle<dyn GeomCurve>,
    c2: &Handle<dyn GeomCurve>,
    u1: f64,
    u2: f64,
    r1: bool,
    r2: bool,
    tl: f64,
    ta: f64,
) -> Result<Shape, Failure> {
    let n1: i32 = derivative_level(local_continuity(c1, u1, tl)?);
    let n2: i32 = derivative_level(local_continuity(c2, u2, tl)?);
    let mut clp1 = CLProps::with_parameter(c1.clone(), u1, n1, tl)?;
    let mut clp2 = CLProps::with_parameter(c2.clone(), u2, n2, tl)?;
    if !clp1.value().is_equal(&clp2.value(), tl) {
        return Err(Failure::new("Courbes non jointives"));
    }
    let min: i32 = n1.min(n2);
    let mut cont: Shape = Shape::C0;
    let mut fini: bool = false;
    if min >= 1 {
        (cont, fini) = first_order(&mut clp1, &mut clp2, (r1, r2), (tl, ta))?;
    }
    if min >= 2 && !fini && clp1.d2()?.is_equal(clp2.d2()?, tl, ta)? {
        cont = Shape::C2;
    }
    Ok(cont)
}

// The first-order comparison: C1 when the (reversed) first derivatives are equal
// (the search goes on), else G1 when both tangents are defined and equal (the
// search stops, as it does without tangents).
fn first_order(
    clp1: &mut CLProps,
    clp2: &mut CLProps,
    (r1, r2): (bool, bool),
    (tl, ta): (f64, f64),
) -> Result<(Shape, bool), Failure> {
    let d1: Vector3 = reversed_if(clp1.d1()?, r1);
    let d2: Vector3 = reversed_if(clp2.d1()?, r2);
    if d1.is_equal(d2, tl, ta)? {
        return Ok((Shape::C1, false));
    }
    if !(clp1.is_tangent_defined()? && clp2.is_tangent_defined()?) {
        return Ok((Shape::C0, true));
    }
    let dir1: Direction3 = reversed_dir_if(clp1.tangent()?, r1);
    let dir2: Direction3 = reversed_dir_if(clp2.tangent()?, r2);
    if dir1.is_equal(&dir2, ta) {
        return Ok((Shape::G1, true));
    }
    Ok((Shape::C0, true))
}

fn reversed_if(vector: Vector3, reverse: bool) -> Vector3 {
    if reverse { vector.reversed() } else { vector }
}

fn reversed_dir_if(direction: Direction3, reverse: bool) -> Direction3 {
    if reverse {
        direction.reversed()
    } else {
        direction
    }
}

/// `GeomLProp::Continuity(C1, C2, u1, u2, r1, r2)`: as
/// [`continuity_with_tolerances`] with Precision::Confusion() and
/// Precision::Angular(); `continuity(&c1, &c2, 1.0, 0.0, false, false)?`.
pub fn continuity(
    c1: &Handle<dyn GeomCurve>,
    c2: &Handle<dyn GeomCurve>,
    u1: f64,
    u2: f64,
    r1: bool,
    r2: bool,
) -> Result<Shape, Failure> {
    continuity_with_tolerances(
        c1,
        c2,
        u1,
        u2,
        r1,
        r2,
        precision::confusion(),
        precision::angular(),
    )
}
