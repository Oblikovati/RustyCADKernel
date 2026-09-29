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
// Rust translation of BinTools_Curve2dSet::WriteCurve2d and its static `operator<<`
// overloads (BinTools_Curve2dSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Writing one `Geom2d_Curve` record: a type code byte (the 3D curve codes) and the
//! fields of that class.
use super::curve_write::code;
use super::errors::{null_geometry, wrap_failure};
use super::ostream::BinOStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_hash::class_dispatch::exact_class;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `BinTools_Curve2dSet::WriteCurve2d`: dispatches on the exact class (the circle is
/// tested first natively); another class raises "UNKNOWN CURVE2d TYPE". Every failure
/// is wrapped in "EXCEPTION in BinTools_Curve2dSet::WriteCurve2d(..)"; a null curve
/// raises Standard_NullObject unwrapped (a crash natively).
pub(crate) fn write_curve2d(
    curve: &Handle<dyn Geom2dCurve>,
    out: &mut BinOStream<'_>,
) -> Result<(), Failure> {
    if curve.is_null() {
        return Err(null_geometry(
            "BinTools_Curve2dSet::WriteCurve2d",
            "Geom2d_Curve",
        ));
    }
    wrap_failure("EXCEPTION in BinTools_Curve2dSet::WriteCurve2d(..)", || {
        if write_conic(curve, out)? || write_bounded(curve, out)? {
            return Ok(());
        }
        Err(Failure::new("UNKNOWN CURVE2d TYPE"))
    })
}

/// The head of a 2D conic: the center, then the X and Y directions.
fn write_frame(out: &mut BinOStream<'_>, code: u8, center: Point2, axes: [Direction2; 2]) {
    out.write_byte(code);
    out.write_point2(center);
    for direction in axes {
        out.write_direction2(direction);
    }
}

fn write_conic(curve: &Handle<dyn Geom2dCurve>, out: &mut BinOStream<'_>) -> Result<bool, Failure> {
    if let Some(circle) = exact_class::<Geom2dCircle, _>(curve) {
        let circ = circle.circ2d()?;
        let axes = [circ.x_axis().direction(), circ.y_axis().direction()];
        write_frame(out, code::CIRCLE, circ.location(), axes);
        out.write_real(circ.radius());
    } else if let Some(line) = exact_class::<Geom2dLine, _>(curve) {
        let lin = line.lin2d();
        out.write_byte(code::LINE);
        out.write_point2(lin.location());
        out.write_direction2(lin.direction());
    } else if let Some(ellipse) = exact_class::<Geom2dEllipse, _>(curve) {
        let elips = ellipse.elips2d()?;
        let axes = [elips.x_axis().direction(), elips.y_axis().direction()];
        write_frame(out, code::ELLIPSE, elips.location(), axes);
        out.write_real(elips.major_radius());
        out.write_real(elips.minor_radius());
    } else if let Some(parabola) = exact_class::<Geom2dParabola, _>(curve) {
        let parab = parabola.parab2d()?;
        let frame = parab.axis();
        let axes = [frame.x_axis().direction(), frame.y_axis().direction()];
        write_frame(out, code::PARABOLA, parab.location(), axes);
        out.write_real(parab.focal());
    } else if let Some(hyperbola) = exact_class::<Geom2dHyperbola, _>(curve) {
        let hypr = hyperbola.hypr2d()?;
        let axes = [hypr.x_axis().direction(), hypr.y_axis().direction()];
        write_frame(out, code::HYPERBOLA, hypr.location(), axes);
        out.write_real(hypr.major_radius());
        out.write_real(hypr.minor_radius());
    } else {
        return Ok(false);
    }
    Ok(true)
}

fn write_bounded(
    curve: &Handle<dyn Geom2dCurve>,
    out: &mut BinOStream<'_>,
) -> Result<bool, Failure> {
    if let Some(bezier) = exact_class::<Geom2dBezierCurve, _>(curve) {
        write_bezier(&bezier, out)?;
    } else if let Some(bspline) = exact_class::<Geom2dBSplineCurve, _>(curve) {
        write_bspline(&bspline, out)?;
    } else if let Some(trimmed) = exact_class::<Geom2dTrimmedCurve, _>(curve) {
        out.write_byte(code::TRIMMED);
        out.write_real(trimmed.first_parameter());
        out.write_real(trimmed.last_parameter());
        write_curve2d(&trimmed.basis_curve(), out)?;
    } else if let Some(offset) = exact_class::<Geom2dOffsetCurve, _>(curve) {
        out.write_byte(code::OFFSET);
        out.write_real(offset.offset());
        write_curve2d(&offset.basis_curve(), out)?;
    } else {
        return Ok(false);
    }
    Ok(true)
}

fn write_bezier(
    bezier: &Handle<Geom2dBezierCurve>,
    out: &mut BinOStream<'_>,
) -> Result<(), Failure> {
    out.write_byte(code::BEZIER);
    let rational = bezier.is_rational();
    out.write_bool(rational);
    let degree = bezier.degree();
    out.write_ext_char(degree as u16);
    for index in 1..=degree + 1 {
        out.write_point2(bezier.pole(index)?);
        if rational {
            out.write_real(bezier.weight(index)?);
        }
    }
    Ok(())
}

fn write_bspline(
    bspline: &Handle<Geom2dBSplineCurve>,
    out: &mut BinOStream<'_>,
) -> Result<(), Failure> {
    out.write_byte(code::BSPLINE);
    let rational = bspline.is_rational();
    out.write_bool(rational);
    out.write_bool(bspline.is_periodic());
    out.write_ext_char(bspline.degree() as u16);
    out.write_integer(bspline.nb_poles());
    out.write_integer(bspline.nb_knots());
    for index in 1..=bspline.nb_poles() {
        out.write_point2(bspline.pole(index)?);
        if rational {
            out.write_real(bspline.weight(index)?);
        }
    }
    for index in 1..=bspline.nb_knots() {
        out.write_real(bspline.knot(index)?);
        out.write_integer(bspline.multiplicity(index)?);
    }
    Ok(())
}
