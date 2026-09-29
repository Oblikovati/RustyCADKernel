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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (GTransform)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::GTransform`: a Geom2d curve through a general 2D transformation.
use super::live;
use crate::geom2d_convert;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dBoundedCurve, Geom2dCircle, Geom2dCurve,
    Geom2dEllipse, Geom2dHyperbola, Geom2dLine, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, GeneralTransform2, Line2, Point2, TransformForm, Xy};

/// `GeomLib::GTransform(Curve, GTrsf)`: the image of `curve` by `gtrsf`, null when
/// it cannot be computed.
///
/// A transformation of form other than gp_Other is applied as a gp_Trsf2d (the
/// geometry keeps its kind). Otherwise the poles of a Bezier or B-spline curve are
/// transformed, a line's location is transformed and its direction multiplied by
/// the vectorial part, circles and ellipses (whole or trimmed) are converted to
/// B-splines first, a trimmed line is re-trimmed at the projections of its
/// transformed ends, and parabolas, hyperbolas and offset curves (whole or as a
/// trimmed basis) give a null handle; `geom_lib::gtransform(&line, &gtrsf)?`.
pub fn gtransform(
    curve: &Handle<dyn Geom2dCurve>,
    gtrsf: &GeneralTransform2,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let source: &dyn Geom2dCurve = live(curve, "GeomLib::GTransform")?;
    if gtrsf.form() != TransformForm::Other {
        // Then, the GTrsf is actually a Trsf: the curve geometry remains unchanged.
        return Ok(source.transformed(gtrsf.trsf2d()?)?.cast());
    }
    if curve.cast::<Geom2dTrimmedCurve>().is_some() {
        return trimmed(curve, gtrsf);
    }
    if curve.cast::<Geom2dLine>().is_some() {
        let line: Handle<Geom2dLine> = source.copy()?.cast();
        transform_line(live(&line, "GeomLib::GTransform")?, gtrsf)?;
        return Ok(line.cast());
    }
    if curve.cast::<Geom2dBezierCurve>().is_some() {
        let c: Handle<Geom2dBezierCurve> = source.copy()?.cast();
        let bezier: &Geom2dBezierCurve = live(&c, "GeomLib::GTransform")?;
        transform_poles(&bezier.poles(), gtrsf, |index, pole| {
            bezier.set_pole(index, pole)
        })?;
        return Ok(c.cast());
    }
    if curve.cast::<Geom2dBSplineCurve>().is_some() {
        let c: Handle<Geom2dBSplineCurve> = source.copy()?.cast();
        let bspline: &Geom2dBSplineCurve = live(&c, "GeomLib::GTransform")?;
        transform_poles(&bspline.poles(), gtrsf, |index, pole| {
            bspline.set_pole(index, pole)
        })?;
        return Ok(c.cast());
    }
    if curve.cast::<Geom2dCircle>().is_some() || curve.cast::<Geom2dEllipse>().is_some() {
        // The curve geometry is not preserved: convert it to BSpline first.
        let c = geom2d_convert::curve_to_bspline_curve(curve, ParameterisationType::TgtThetaOver2)?;
        return gtransform(&c.cast(), gtrsf);
    }
    // Parabolas, hyperbolas and offset curves are not supported: a null Handle.
    Ok(Handle::default())
}

// A trimmed curve: recursion on the basis for B-spline and Bezier bases (their
// parameterization is preserved), a re-trimmed line, a B-spline of a conic basis.
fn trimmed(
    curve: &Handle<dyn Geom2dCurve>,
    gtrsf: &GeneralTransform2,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let c: Handle<Geom2dTrimmedCurve> = live(curve, "GeomLib::GTransform")?.copy()?.cast();
    let ctrim: &Geom2dTrimmedCurve = live(&c, "GeomLib::GTransform")?;
    let basis: Handle<dyn Geom2dCurve> = ctrim.basis_curve();
    if basis.cast::<Geom2dBSplineCurve>().is_some() || basis.cast::<Geom2dBezierCurve>().is_some() {
        let (u1, u2): (f64, f64) = (ctrim.first_parameter(), ctrim.last_parameter());
        let image: Handle<dyn Geom2dCurve> = gtransform(&basis, gtrsf)?;
        return Ok(Handle::new(Geom2dTrimmedCurve::new(&image, u1, u2, true, true)?).cast());
    }
    if basis.cast::<Geom2dLine>().is_some() {
        return trimmed_line(ctrim, &basis, gtrsf);
    }
    let conic: bool = basis.cast::<Geom2dCircle>().is_some()
        || basis.cast::<Geom2dEllipse>().is_some()
        || basis.cast::<Geom2dParabola>().is_some()
        || basis.cast::<Geom2dHyperbola>().is_some();
    if conic {
        // The curve geometry is not preserved: convert it to BSpline first.
        let bs =
            geom2d_convert::curve_to_bspline_curve(&c.cast(), ParameterisationType::TgtThetaOver2)?;
        return gtransform(&bs.cast(), gtrsf);
    }
    // The transform of an OffsetCurve is not supported.
    Ok(Handle::default())
}

// The trimmed line: its parameterization is not preserved, so the trimming
// parameters are recomputed by projecting the transformed ends on the image line.
fn trimmed_line(
    ctrim: &Geom2dTrimmedCurve,
    basis: &Handle<dyn Geom2dCurve>,
    gtrsf: &GeneralTransform2,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let l: Handle<Geom2dLine> = gtransform(basis, gtrsf)?.cast();
    let lin: Line2 = live(&l, "GeomLib::GTransform")?.lin2d();
    let p1: Point2 = transformed_point(ctrim.start_point()?, gtrsf);
    let p2: Point2 = transformed_point(ctrim.end_point()?, gtrsf);
    let u1: f64 = elclib::line_parameter_2d(lin.position(), p1);
    let u2: f64 = elclib::line_parameter_2d(lin.position(), p2);
    Ok(Handle::new(Geom2dTrimmedCurve::new(&l.cast(), u1, u2, true, true)?).cast())
}

fn transformed_point(point: Point2, gtrsf: &GeneralTransform2) -> Point2 {
    let xy: Xy = gtrsf.transformed(point.xy());
    Point2::new(xy.x(), xy.y())
}

// The location transformed; the direction multiplied by the vectorial part only.
fn transform_line(line: &Geom2dLine, gtrsf: &GeneralTransform2) -> Result<(), Failure> {
    let lin: Line2 = line.lin2d();
    let location: Point2 = transformed_point(lin.location(), gtrsf);
    let direction: Xy = lin
        .direction()
        .xy()
        .multiplied_matrix(gtrsf.vectorial_part());
    line.set_location(location);
    line.set_direction(Direction2::new(direction.x(), direction.y())?);
    Ok(())
}

// Since GTrsf is a linear operation, the transform of a pole-based curve is the
// curve whose poles are the transforms of the base curve's poles.
fn transform_poles(
    poles: &rustycad_tkernel::ncollection::Array1<'static, Point2>,
    gtrsf: &GeneralTransform2,
    mut set_pole: impl FnMut(i32, Point2) -> Result<(), Failure>,
) -> Result<(), Failure> {
    for (index, pole) in (1..).zip(poles.iter()) {
        set_pole(index, transformed_point(*pole, gtrsf))?;
    }
    Ok(())
}
