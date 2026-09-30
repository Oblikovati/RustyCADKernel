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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecAna.cxx (Distance) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_BisecAna::Distance`: the distance from the point of passage to an
//! analytic bisector, its start parameter and the side of it to keep.
// The native tests are written `x < t`; their negations keep the native answer for a NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::gcc_int::{Bisec, IType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};

/// The inputs of one `Distance` call: the point of passage, the two vectors that
/// choose the sector, the reference vector (null when unused) and the side.
#[derive(Clone, Copy)]
pub(super) struct DistanceQuery {
    pub apoint: Point2,
    pub first: Vector2,
    pub second: Vector2,
    pub vec_ref: Vector2,
    pub adirection: f64,
}

/// The outputs of `Distance`: the distance, the parameter on the bisector of the
/// projection of the point, the sense in which to keep the bisector (the in-out
/// `asense`) and the status.
#[derive(Clone, Copy, Debug)]
pub(super) struct DistanceResult {
    pub distance: f64,
    pub parameter: f64,
    pub sense: bool,
    pub ok: bool,
}

// The projection of the point on the bisector: its parameter, the point and the
// tangent. A point locus is skipped by the native switch: its null tangent then
// raises in the gp_Dir2d constructor before the parameter is read.
fn project(bisector: &dyn Bisec, apoint: Point2) -> Result<(f64, Point2, Vector2), Failure> {
    match bisector.arc_type() {
        IType::Lin => project_on(&bisector.line()?, apoint),
        IType::Cir => project_on(&bisector.circle()?, apoint),
        IType::Hpr => project_on(&bisector.hyperbola()?, apoint),
        IType::Par => project_on(&bisector.parabola()?, apoint),
        IType::Ell => project_on(&bisector.ellipse()?, apoint),
        IType::Pnt => Ok((0., Point2::default(), Vector2::default())),
    }
}

// `ElCLib::Parameter` then `ElCLib::D1` on one conic.
fn project_on<C>(curve: &C, apoint: Point2) -> Result<(f64, Point2, Vector2), Failure>
where
    C: elclib::ElementaryCurve<Point = Point2, Vector = Vector2>,
{
    let parameter: f64 = elclib::parameter(curve, apoint)?;
    let (point, tangent) = elclib::d1(parameter, curve);
    Ok((parameter, point, tangent))
}

/// `Distance(apoint, abisector, afirstvector, asecondvector, VecRef, adirection,
/// aparameter, asense, astatus, IsBisecOfTwoLines)`. `sense` is the in-out
/// `asense`: true asks for the status of the on-curve pass only.
pub(super) fn distance(
    query: DistanceQuery,
    bisector: &dyn Bisec,
    sense: bool,
    two_lines: bool,
) -> Result<DistanceResult, Failure> {
    let (parameter, point, tangent) = project(bisector, query.apoint)?;
    let distance: f64 = query.apoint.distance(point);
    let first = Direction2::from_vector(query.first)?;
    let second = Direction2::from_vector(query.second)?;
    let tang = Direction2::from_vector(tangent)?;
    let mut result = DistanceResult {
        distance,
        parameter,
        sense,
        ok: true,
    };
    if sense {
        result.ok = status_on_curve(first, second, tang);
        return Ok(result);
    }
    result.sense = true;
    if !two_lines {
        let in_sector: bool = sense_in_sector(first, second.reversed(), tang, query.adirection);
        // The reference test runs whatever the sector test answered, as natively.
        let agrees: bool = reference_agrees(query.vec_ref, tang)?;
        result.sense = in_sector && agrees;
    }
    Ok(result)
}

// The comparisons keep the native form (`!(x < t)`), so a NaN answers as natively.
// The status of the on-curve pass: the tangent of the bisector must bisect the two
// vectors.
fn status_on_curve(first: Direction2, second: Direction2, tang: Direction2) -> bool {
    let sin_plat: f64 = 1.0e-3;
    if first.crossed(second).abs() < sin_plat {
        return flat_status(first, second, tang);
    }
    !(first.crossed(tang) * tang.crossed(second) < -1.0E-8)
}

// The status when the two vectors are flat: tangent or opposed.
fn flat_status(first: Direction2, second: Direction2, tang: Direction2) -> bool {
    if first.dot(second) >= 0.0 {
        // Tangent mixed: correct if the scalar product is close to 1.
        return !(tang.dot(first).abs() < 0.5);
    }
    // Opposed tangents: correct if the scalar product is close to 0.
    !(tang.dot(first).abs() > 0.5)
}

// The sense of the second pass: false when the tangent leaves the sector of
// `first` and the reversed second vector on the side `adirection`.
fn sense_in_sector(
    first: Direction2,
    secdirrev: Direction2,
    tang: Direction2,
    adirection: f64,
) -> bool {
    // Modified by Sergey KHROMOV - Tue Oct 22 16:35:51 2002: -1.0E-8 became 1.0e-4.
    let a_tol: f64 = 1.0e-4;
    let opening: f64 = first.crossed(secdirrev) * adirection;
    if opening < -0.1 {
        // input
        return !(first.crossed(tang) * adirection < a_tol
            && secdirrev.crossed(tang) * adirection < a_tol);
    }
    if opening > 0.1 {
        // output
        return !(first.crossed(tang) * adirection < a_tol
            || secdirrev.crossed(tang) * adirection < a_tol);
    }
    flat_sense(first, secdirrev, tang, adirection)
}

// The sense when the sector is flat: tangent vectors or a turn back.
fn flat_sense(first: Direction2, secdirrev: Direction2, tang: Direction2, adirection: f64) -> bool {
    if first.dot(secdirrev) > 0. {
        return !(first.crossed(tang) * adirection < 0.);
    }
    // Modified by Sergey KHROMOV - Thu Oct 31 14:16:53 2002: the test no longer
    // multiplies by adirection.
    !(first.dot(tang) < 0.)
}

// jgv, OCC26185: a non-null reference vector rejects a tangent opposed to it.
fn reference_agrees(vec_ref: Vector2, tang: Direction2) -> Result<bool, Failure> {
    if vec_ref.square_magnitude() == 0. {
        return Ok(true);
    }
    let dir_ref = Direction2::from_vector(vec_ref)?;
    Ok(!(tang.dot(dir_ref) < 0.))
}
