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
// Rust translation of BinTools_CurveSet::ReadCurve and its static `operator>>`
// overloads (BinTools_CurveSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading one binary `Geom_Curve` record back.
use super::curve_write::code;
use super::errors::wrap_failure;
use super::geometry_in::{read_degree, read_direction3, read_knots, read_point3, read_poles};
use super::scalars::{get_bool, get_integer, get_real, stream_get};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};

/// `BinTools_CurveSet::ReadCurve`: the type code byte, then the record of that class.
/// An unknown code raises "UNKNOWN CURVE TYPE"; every failure (a short read, a null
/// direction, a refused construction) is wrapped in "EXCEPTION in
/// BinTools_CurveSet::ReadCurve(..)" with the stream left where it happened.
pub(crate) fn read_curve(stream: &mut IoStream) -> Result<Handle<dyn GeomCurve>, Failure> {
    wrap_failure("EXCEPTION in BinTools_CurveSet::ReadCurve(..)", || {
        let ctype = stream_get(stream) as u8;
        read_record(ctype, stream)
    })
}

fn read_record(ctype: u8, stream: &mut IoStream) -> Result<Handle<dyn GeomCurve>, Failure> {
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
        _ => Err(Failure::new("UNKNOWN CURVE TYPE")),
    }
}

/// `C = CC;`.
fn upcast<C: Transient + RuntimeTyped>(curve: C) -> Handle<dyn GeomCurve> {
    Handle::new(curve).into_cast()
}

fn read_line(stream: &mut IoStream) -> Result<GeomLine, Failure> {
    let point = read_point3(stream)?;
    let direction = read_direction3(stream)?;
    Ok(GeomLine::from_point(point, direction))
}

/// `IS >> P >> A >> AX >> AY`: the conic frame. The Y direction is read (and must not
/// be null) but dropped, since `gp_Ax2(P, A, AX)` recomputes it.
fn read_conic_frame(stream: &mut IoStream) -> Result<(Point3, Direction3, Direction3), Failure> {
    let point = read_point3(stream)?;
    let main = read_direction3(stream)?;
    let x = read_direction3(stream)?;
    read_direction3(stream)?;
    Ok((point, main, x))
}

/// `gp_Ax2(P, A, AX)`, built after the radii are read, as natively.
fn conic_axis((point, main, x): (Point3, Direction3, Direction3)) -> Result<Axis2, Failure> {
    Axis2::with_x_direction(point, main, x)
}

fn read_circle(stream: &mut IoStream) -> Result<GeomCircle, Failure> {
    let frame = read_conic_frame(stream)?;
    let radius = get_real(stream)?;
    GeomCircle::new(conic_axis(frame)?, radius)
}

fn read_ellipse(stream: &mut IoStream) -> Result<GeomEllipse, Failure> {
    let frame = read_conic_frame(stream)?;
    let (major, minor) = (get_real(stream)?, get_real(stream)?);
    GeomEllipse::new(conic_axis(frame)?, major, minor)
}

fn read_parabola(stream: &mut IoStream) -> Result<GeomParabola, Failure> {
    let frame = read_conic_frame(stream)?;
    let focal = get_real(stream)?;
    GeomParabola::new(conic_axis(frame)?, focal)
}

fn read_hyperbola(stream: &mut IoStream) -> Result<GeomHyperbola, Failure> {
    let frame = read_conic_frame(stream)?;
    let (major, minor) = (get_real(stream)?, get_real(stream)?);
    GeomHyperbola::new(conic_axis(frame)?, major, minor)
}

/// The rational flag, the `char16_t` degree, then `degree + 1` poles with their weights.
fn read_bezier(stream: &mut IoStream) -> Result<GeomBezierCurve, Failure> {
    let rational = get_bool(stream);
    let degree = read_degree(stream)?;
    let (poles, weights) = read_poles(stream, degree + 1, rational, read_point3)?;
    if rational {
        GeomBezierCurve::with_weights(&poles, &weights)
    } else {
        GeomBezierCurve::new(&poles)
    }
}

/// The flags, the degree and the counts, the poles with their weights, then each knot
/// with its multiplicity.
fn read_bspline(stream: &mut IoStream) -> Result<GeomBSplineCurve, Failure> {
    let (rational, periodic) = (get_bool(stream), get_bool(stream));
    let degree = read_degree(stream)?;
    let nb_poles = get_integer(stream)?;
    let nb_knots = get_integer(stream)?;
    let (poles, weights) = read_poles(stream, nb_poles, rational, read_point3)?;
    let (knots, mults) = read_knots(stream, nb_knots)?;
    if rational {
        GeomBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, periodic)
    } else {
        GeomBSplineCurve::new(&poles, &knots, &mults, degree, periodic)
    }
}

/// The parameters, then the basis record; `Geom_TrimmedCurve(BC, p1, p2)` with its
/// default sense and periodic adjustment.
fn read_trimmed(stream: &mut IoStream) -> Result<GeomTrimmedCurve, Failure> {
    let (first, last) = (get_real(stream)?, get_real(stream)?);
    let basis = read_curve(stream)?;
    GeomTrimmedCurve::new(&basis, first, last, true, true)
}

/// The offset, the reference direction, then the basis record; the basis continuity
/// is checked (`isNotCheckC0` defaults to false).
fn read_offset(stream: &mut IoStream) -> Result<GeomOffsetCurve, Failure> {
    let offset = get_real(stream)?;
    let direction = read_direction3(stream)?;
    let basis = read_curve(stream)?;
    GeomOffsetCurve::new(&basis, offset, direction, false)
}
