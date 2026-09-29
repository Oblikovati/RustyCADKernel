// Created on: 1993-07-07
// Created by: Jean Claude VAUTHIER
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (To3d)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::To3d`: the 3D curve of a Geom2d curve drawn on a plane.
use super::live;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2, Point2, Point3};

/// `GeomLib::To3d(Position, Curve2d)`: the Geom curve of the same kind whose
/// geometry is `curve_2d` placed in the plane of `position` (the 2D X and Y axes on
/// the frame's X and Y directions). Trimmed and offset curves are rebuilt on the
/// converted basis (the offset direction is the plane normal). The dispatch is on
/// the exact class, as the native DynamicType comparison; any other curve raises
/// Standard_NotImplemented;
/// `geom_lib::to_3d(frame, &curve_2d)?`.
pub fn to_3d(
    position: Axis2,
    curve_2d: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    live(curve_2d, "GeomLib::To3d")?;
    if let Some(ct) = curve_2d.cast::<Geom2dTrimmedCurve>().get() {
        let cc: Handle<dyn GeomCurve> = to_3d(position, &ct.basis_curve())?;
        let curve =
            GeomTrimmedCurve::new(&cc, ct.first_parameter(), ct.last_parameter(), true, true)?;
        return Ok(Handle::new(curve).cast());
    }
    if let Some(co) = curve_2d.cast::<Geom2dOffsetCurve>().get() {
        let cc: Handle<dyn GeomCurve> = to_3d(position, &co.basis_curve())?;
        return Ok(Handle::new(GeomOffsetCurve::new(
            &cc,
            co.offset(),
            position.direction(),
            false,
        )?)
        .cast());
    }
    if let Some(bezier) = curve_2d.cast::<Geom2dBezierCurve>().get() {
        return bezier_to_3d(position, bezier);
    }
    if let Some(bspline) = curve_2d.cast::<Geom2dBSplineCurve>().get() {
        return bspline_to_3d(position, bspline);
    }
    conic_to_3d(position, curve_2d)
}

// The poles placed in the plane (the native sized array; its bounds do not matter to
// the curve constructors, which copy the values in order).
fn poles_to_3d(
    position: Axis2,
    poles_2d: &Array1<'static, Point2>,
) -> Result<Array1<'static, Point3>, Failure> {
    let mut poles: Array1<'static, Point3> = one_based(poles_2d.length())?;
    for (target, source) in poles.iter_mut().zip(poles_2d.iter()) {
        *target = elclib::to3d_point(position, *source);
    }
    Ok(poles)
}

fn bezier_to_3d(
    position: Axis2,
    bezier: &Geom2dBezierCurve,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let poles: Array1<'static, Point3> = poles_to_3d(position, &bezier.poles())?;
    let curve = match bezier.weights() {
        Some(weights) if bezier.is_rational() => GeomBezierCurve::with_weights(&poles, &weights)?,
        _ => GeomBezierCurve::new(&poles)?,
    };
    Ok(Handle::new(curve).cast())
}

fn bspline_to_3d(
    position: Axis2,
    bspline: &Geom2dBSplineCurve,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let poles: Array1<'static, Point3> = poles_to_3d(position, &bspline.poles())?;
    let (knots, mults) = (bspline.knots(), bspline.multiplicities());
    let (degree, periodic) = (bspline.degree(), bspline.is_periodic());
    let curve = if bspline.is_rational() {
        let weights: Array1<'static, f64> = bspline.weights_array(|w| w.clone());
        GeomBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, periodic)?
    } else {
        GeomBSplineCurve::new(&poles, &knots, &mults, degree, periodic)?
    };
    Ok(Handle::new(curve).cast())
}

// The line and the four conics, through the ElCLib embeddings.
fn conic_to_3d(
    position: Axis2,
    curve_2d: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    if let Some(line) = curve_2d.cast::<Geom2dLine>().get() {
        return Ok(Handle::new(GeomLine::from_lin(elclib::to3d_line(
            position,
            line.lin2d(),
        )?))
        .cast());
    }
    if let Some(circle) = curve_2d.cast::<Geom2dCircle>().get() {
        return Ok(Handle::new(GeomCircle::from_circ(elclib::to3d_circle(
            position,
            circle.circ2d()?,
        )?))
        .cast());
    }
    if let Some(ellipse) = curve_2d.cast::<Geom2dEllipse>().get() {
        return Ok(Handle::new(GeomEllipse::from_elips(elclib::to3d_ellipse(
            position,
            ellipse.elips2d()?,
        )?))
        .cast());
    }
    if let Some(parabola) = curve_2d.cast::<Geom2dParabola>().get() {
        return Ok(Handle::new(GeomParabola::from_parab(elclib::to3d_parabola(
            position,
            parabola.parab2d()?,
        )?))
        .cast());
    }
    if let Some(hyperbola) = curve_2d.cast::<Geom2dHyperbola>().get() {
        return Ok(Handle::new(GeomHyperbola::from_hypr(elclib::to3d_hyperbola(
            position,
            hyperbola.hypr2d()?,
        )?))
        .cast());
    }
    Err(failure(
        "Standard_NotImplemented",
        "GeomLib::To3d: the curve class has no 3D counterpart; expected a Geom2d line, conic, Bezier, B-spline, trimmed or offset curve",
    ))
}
