// Created on: 2004-05-20
// Created by: Sergey ZARITCHNY
// Copyright (c) 2004-2014 OPEN CASCADE SAS
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
// Rust translation of BinTools_Curve2dSet::ReadCurve2d and its static `operator>>`
// overloads (BinTools_Curve2dSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading one binary `Geom2d_Curve` record back.
use super::curve_write::code;
use super::errors::wrap_failure;
use super::geometry_in::{read_degree, read_direction2, read_knots, read_point2, read_poles};
use super::scalars::{get_bool, get_integer, get_real, stream_get};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2};

/// `BinTools_Curve2dSet::ReadCurve2d`: the type code byte, then the record of that
/// class; an unknown code raises "UNKNOWN CURVE2d TYPE", and every failure is wrapped
/// in "EXCEPTION in BinTools_Curve2dSet::ReadCurve2d(...)".
pub(crate) fn read_curve2d(stream: &mut IoStream) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    wrap_failure("EXCEPTION in BinTools_Curve2dSet::ReadCurve2d(...)", || {
        let ctype = stream_get(stream) as u8;
        read_record(ctype, stream)
    })
}

fn read_record(ctype: u8, stream: &mut IoStream) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    match ctype {
        code::LINE => read_line(stream).map(upcast),
        code::CIRCLE => read_circle(stream).map(upcast),
        code::ELLIPSE => read_ellipse(stream).map(upcast),
        code::PARABOLA => read_parabola(stream).map(upcast),
        code::HYPERBOLA => read_hyperbola(stream).map(upcast),
        code::BEZIER => read_bezier(stream).map(upcast),
        code::BSPLINE => read_bspline(stream).map(upcast),
        code::TRIMMED => read_trimmed(stream).map(upcast),
        code::OFFSET => read_offset(stream).map(upcast),
        _ => Err(Failure::new("UNKNOWN CURVE2d TYPE")),
    }
}

fn upcast<C: Transient + RuntimeTyped>(curve: C) -> Handle<dyn Geom2dCurve> {
    Handle::new(curve).into_cast()
}

fn read_line(stream: &mut IoStream) -> Result<Geom2dLine, Failure> {
    let point = read_point2(stream)?;
    let direction = read_direction2(stream)?;
    Ok(Geom2dLine::from_point(point, direction))
}

/// `IS >> P >> AX >> AY`: the center and the two directions of `gp_Ax22d(P, AX, AY)`.
fn read_conic_frame(stream: &mut IoStream) -> Result<(Point2, Direction2, Direction2), Failure> {
    let point = read_point2(stream)?;
    let x = read_direction2(stream)?;
    let y = read_direction2(stream)?;
    Ok((point, x, y))
}

/// `gp_Ax22d(P, AX, AY)`, built after the radii are read, as natively.
fn frame((point, x, y): (Point2, Direction2, Direction2)) -> Result<Axis22d, Failure> {
    Axis22d::with_directions(point, x, y)
}

fn read_circle(stream: &mut IoStream) -> Result<Geom2dCircle, Failure> {
    let directions = read_conic_frame(stream)?;
    let radius = get_real(stream)?;
    Geom2dCircle::new(frame(directions)?, radius)
}

fn read_ellipse(stream: &mut IoStream) -> Result<Geom2dEllipse, Failure> {
    let directions = read_conic_frame(stream)?;
    let (major, minor) = (get_real(stream)?, get_real(stream)?);
    Geom2dEllipse::new(frame(directions)?, major, minor)
}

fn read_parabola(stream: &mut IoStream) -> Result<Geom2dParabola, Failure> {
    let directions = read_conic_frame(stream)?;
    let focal = get_real(stream)?;
    Geom2dParabola::new(frame(directions)?, focal)
}

fn read_hyperbola(stream: &mut IoStream) -> Result<Geom2dHyperbola, Failure> {
    let directions = read_conic_frame(stream)?;
    let (major, minor) = (get_real(stream)?, get_real(stream)?);
    Geom2dHyperbola::new(frame(directions)?, major, minor)
}

fn read_bezier(stream: &mut IoStream) -> Result<Geom2dBezierCurve, Failure> {
    let rational = get_bool(stream);
    let degree = read_degree(stream)?;
    let (poles, weights) = read_poles(stream, degree + 1, rational, read_point2)?;
    if rational {
        Geom2dBezierCurve::with_weights(&poles, &weights)
    } else {
        Geom2dBezierCurve::new(&poles)
    }
}

fn read_bspline(stream: &mut IoStream) -> Result<Geom2dBSplineCurve, Failure> {
    let (rational, periodic) = (get_bool(stream), get_bool(stream));
    let degree = read_degree(stream)?;
    let nb_poles = get_integer(stream)?;
    let nb_knots = get_integer(stream)?;
    let (poles, weights) = read_poles(stream, nb_poles, rational, read_point2)?;
    let (knots, mults) = read_knots(stream, nb_knots)?;
    if rational {
        Geom2dBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, periodic)
    } else {
        Geom2dBSplineCurve::new(&poles, &knots, &mults, degree, periodic)
    }
}

fn read_trimmed(stream: &mut IoStream) -> Result<Geom2dTrimmedCurve, Failure> {
    let (first, last) = (get_real(stream)?, get_real(stream)?);
    let basis = read_curve2d(stream)?;
    Geom2dTrimmedCurve::new(&basis, first, last, true, true)
}

/// The offset, then the basis record (a 2D offset curve has no reference direction).
fn read_offset(stream: &mut IoStream) -> Result<Geom2dOffsetCurve, Failure> {
    let offset = get_real(stream)?;
    let basis = read_curve2d(stream)?;
    Geom2dOffsetCurve::new(&basis, offset, false)
}
