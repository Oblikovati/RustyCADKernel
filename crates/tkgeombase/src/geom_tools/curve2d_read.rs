// Created on: 1993-07-19
// Created by: Remi LEQUETTE
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
// Rust translation of GeomTools_Curve2dSet::ReadCurve2d and its static `operator>>`
// overloads (GeomTools_Curve2dSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading one compact `Geom2d_Curve` record back.
use super::text_in::{
    get_real, read_bezier_header, read_bspline_header, read_direction2, read_int, read_knots,
    read_point2, read_poles,
};
use super::type_codes::curve as code;
use super::undefined_type_handler;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2};

/// `GeomTools_Curve2dSet::ReadCurve2d`: the type code, then the record of that
/// class; an unknown code goes to the active undefined-type handler. Every failure
/// gives a null curve, with the stream left where the failure happened.
pub(crate) fn read_curve2d(stream: &mut IoStream) -> Handle<dyn Geom2dCurve> {
    // An uninitialised native `ctype` when the stream already failed reads 0 here.
    let ctype = read_int(stream);
    read_record(ctype, stream).unwrap_or_default()
}

fn read_record(ctype: i32, stream: &mut IoStream) -> Result<Handle<dyn Geom2dCurve>, Failure> {
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
        _ => undefined_type_handler().read_curve2d(ctype, stream),
    }
}

/// `occ::handle<Geom2d_Curve> C = CC;`.
fn upcast<C: Transient + RuntimeTyped>(curve: C) -> Handle<dyn Geom2dCurve> {
    Handle::new(curve).into_cast()
}

fn read_line(stream: &mut IoStream) -> Result<Geom2dLine, Failure> {
    let point = read_point2(stream);
    let direction = read_direction2(stream)?;
    Ok(Geom2dLine::from_point(point, direction))
}

/// `IS >> P >> AX >> AY`: the center and the two directions of `gp_Ax22d(P, AX, AY)`,
/// which keeps AX and recomputes AY on the side AY gives.
fn read_conic_frame(stream: &mut IoStream) -> Result<(Point2, Direction2, Direction2), Failure> {
    let point = read_point2(stream);
    let x = read_direction2(stream)?;
    let y = read_direction2(stream)?;
    Ok((point, x, y))
}

fn frame((point, x, y): (Point2, Direction2, Direction2)) -> Result<Axis22d, Failure> {
    Axis22d::with_directions(point, x, y)
}

fn read_circle(stream: &mut IoStream) -> Result<Geom2dCircle, Failure> {
    let directions = read_conic_frame(stream)?;
    let radius = get_real(stream);
    Geom2dCircle::new(frame(directions)?, radius)
}

fn read_ellipse(stream: &mut IoStream) -> Result<Geom2dEllipse, Failure> {
    let directions = read_conic_frame(stream)?;
    let (major, minor) = (get_real(stream), get_real(stream));
    Geom2dEllipse::new(frame(directions)?, major, minor)
}

fn read_parabola(stream: &mut IoStream) -> Result<Geom2dParabola, Failure> {
    let directions = read_conic_frame(stream)?;
    let focal = get_real(stream);
    Geom2dParabola::new(frame(directions)?, focal)
}

fn read_hyperbola(stream: &mut IoStream) -> Result<Geom2dHyperbola, Failure> {
    let directions = read_conic_frame(stream)?;
    let (major, minor) = (get_real(stream), get_real(stream));
    Geom2dHyperbola::new(frame(directions)?, major, minor)
}

/// `IS >> rational >> degree`, then `degree + 1` poles with their weights.
fn read_bezier(stream: &mut IoStream) -> Result<Geom2dBezierCurve, Failure> {
    let (rational, count) = read_bezier_header(stream);
    let (poles, weights) = read_poles(stream, count, rational, read_point2)?;
    if rational {
        Geom2dBezierCurve::with_weights(&poles, &weights)
    } else {
        Geom2dBezierCurve::new(&poles)
    }
}

/// The B-spline flags and counts, the poles with their weights, then the knots.
fn read_bspline(stream: &mut IoStream) -> Result<Geom2dBSplineCurve, Failure> {
    let header = read_bspline_header(stream);
    let (poles, weights) = read_poles(stream, header.nb_poles, header.rational, read_point2)?;
    let (knots, mults) = read_knots(stream, header.nb_knots)?;
    let (degree, periodic) = (header.degree, header.periodic);
    if header.rational {
        Geom2dBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, periodic)
    } else {
        Geom2dBSplineCurve::new(&poles, &knots, &mults, degree, periodic)
    }
}

/// `p1`, `p2`, then the basis record.
fn read_trimmed(stream: &mut IoStream) -> Result<Geom2dTrimmedCurve, Failure> {
    let (first, last) = (get_real(stream), get_real(stream));
    let basis = read_curve2d(stream);
    Geom2dTrimmedCurve::new(&basis, first, last, true, true)
}

/// The offset, then the basis record (a 2D offset has no reference direction).
fn read_offset(stream: &mut IoStream) -> Result<Geom2dOffsetCurve, Failure> {
    let offset = get_real(stream);
    let basis = read_curve2d(stream);
    Geom2dOffsetCurve::new(&basis, offset, false)
}
