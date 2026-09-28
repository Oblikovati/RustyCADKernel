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
// Rust translation of GeomTools_CurveSet::PrintCurve and its static `Print` overloads
// (GeomTools_CurveSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text of a `Geom_Curve`: the dump layout and the compact layout of the BRep
//! ASCII `Curves` section, with the type codes 1 (line) to 9 (offset curve).
use super::curve_text::{
    BSplineCurveText, CurvePoles, KnotLayout, print_bezier, print_bspline, print_conic_head,
    print_line, print_offset_head, print_radii_tail, print_single_tail, print_trimmed_head,
};
use super::text_out::{TextOut, direction3, point3};
use super::type_codes::curve as code;
use super::{null_geometry, undefined_type_handler};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d_hash::class_dispatch::exact_class;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis1, Circle3, Ellipse3, Hyperbola3, Parabola3, Point3};

/// `GeomTools_CurveSet::PrintCurve`: dispatches on the exact class; any other class
/// goes to the active undefined-type handler. A null handle, which the native code
/// dereferences, is Standard_NullObject.
pub(crate) fn print_curve(
    curve: &Handle<dyn GeomCurve>,
    stream: &mut IoStream,
    compact: bool,
) -> Result<(), Failure> {
    if curve.is_null() {
        return Err(null_geometry(
            "GeomTools_CurveSet::PrintCurve",
            "Geom_Curve",
        ));
    }
    let handled = {
        let mut out = TextOut::new(stream, compact);
        print_conic(curve, &mut out)? || print_bounded(curve, &mut out)?
    };
    if handled {
        return Ok(());
    }
    undefined_type_handler().print_curve(curve, stream, compact)
}

/// The line and the conics, in the native test order; false for another class.
fn print_conic(curve: &Handle<dyn GeomCurve>, out: &mut TextOut<'_>) -> Result<bool, Failure> {
    if let Some(line) = exact_class::<GeomLine, _>(curve) {
        let lin = line.lin();
        print_line(out, &point3(lin.location()), &direction3(lin.direction()));
    } else if let Some(circle) = exact_class::<GeomCircle, _>(curve) {
        let circ = circle.circ()?;
        print_frame(out, code::CIRCLE, b"Circle", circ.location(), circ.axes());
        print_single_tail(out, b"\n  Radius :", circ.radius());
    } else if let Some(ellipse) = exact_class::<GeomEllipse, _>(curve) {
        let elips = ellipse.elips()?;
        print_frame(
            out,
            code::ELLIPSE,
            b"Ellipse",
            elips.location(),
            elips.axes(),
        );
        print_radii_tail(out, elips.major_radius(), elips.minor_radius());
    } else if let Some(parabola) = exact_class::<GeomParabola, _>(curve) {
        let parab = parabola.parab()?;
        print_frame(
            out,
            code::PARABOLA,
            b"Parabola",
            parab.location(),
            parab.axes(),
        );
        print_single_tail(out, b"\n  Focal  :", parab.focal());
    } else if let Some(hyperbola) = exact_class::<GeomHyperbola, _>(curve) {
        let hypr = hyperbola.hypr()?;
        print_frame(
            out,
            code::HYPERBOLA,
            b"Hyperbola",
            hypr.location(),
            hypr.axes(),
        );
        print_radii_tail(out, hypr.major_radius(), hypr.minor_radius());
    } else {
        return Ok(false);
    }
    Ok(true)
}

/// The splines and the trimmed and offset curves; false for another class.
fn print_bounded(curve: &Handle<dyn GeomCurve>, out: &mut TextOut<'_>) -> Result<bool, Failure> {
    if let Some(bezier) = exact_class::<GeomBezierCurve, _>(curve) {
        print_bezier_curve(&bezier, out)?;
    } else if let Some(bspline) = exact_class::<GeomBSplineCurve, _>(curve) {
        print_bspline_curve(&bspline, out)?;
    } else if let Some(trimmed) = exact_class::<GeomTrimmedCurve, _>(curve) {
        print_trimmed_head(out, trimmed.first_parameter(), trimmed.last_parameter());
        let compact = out.compact();
        print_curve(&trimmed.basis_curve(), out.stream(), compact)?;
    } else if let Some(offset) = exact_class::<GeomOffsetCurve, _>(curve) {
        print_offset(&offset, out)?;
    } else {
        return Ok(false);
    }
    Ok(true)
}

/// The main axis and the X and Y axes of a gp conic (`Axis()`, `XAxis()`, `YAxis()`).
trait ConicAxes {
    fn axes(&self) -> [Axis1; 3];
}

macro_rules! conic_axes {
    ($($conic:ty),+) => {
        $(impl ConicAxes for $conic {
            fn axes(&self) -> [Axis1; 3] {
                [self.axis(), self.x_axis(), self.y_axis()]
            }
        })+
    };
}
conic_axes!(Circle3, Ellipse3, Parabola3, Hyperbola3);

/// The head of a 3D conic record: center, then the three axis directions.
fn print_frame(out: &mut TextOut<'_>, code: i32, name: &[u8], center: Point3, axes: [Axis1; 3]) {
    let [main, x, y] = axes.map(|axis| direction3(axis.direction()));
    let fields: [(&[u8], &[f64]); 4] = [
        (b"\n  Center :", &point3(center)),
        (b"\n  Axis   :", &main),
        (b"\n  XAxis  :", &x),
        (b"\n  YAxis  :", &y),
    ];
    print_conic_head(out, code, name, &fields);
}

fn print_bezier_curve(
    bezier: &Handle<GeomBezierCurve>,
    out: &mut TextOut<'_>,
) -> Result<(), Failure> {
    let pole = |index: i32| bezier.pole(index).map(|pole| point3(pole).to_vec());
    let weight = |index: i32| bezier.weight(index);
    let poles = CurvePoles {
        rational: bezier.is_rational(),
        pole: &pole,
        weight: &weight,
    };
    print_bezier(out, bezier.degree(), &poles)
}

fn print_bspline_curve(
    bspline: &Handle<GeomBSplineCurve>,
    out: &mut TextOut<'_>,
) -> Result<(), Failure> {
    let pole = |index: i32| bspline.pole(index).map(|pole| point3(pole).to_vec());
    let weight = |index: i32| bspline.weight(index);
    let knot = |index: i32| bspline.knot(index);
    let multiplicity = |index: i32| bspline.multiplicity(index);
    let spline = BSplineCurveText {
        periodic: bspline.is_periodic(),
        degree: bspline.degree(),
        nb_poles: bspline.nb_poles(),
        nb_knots: bspline.nb_knots(),
        poles: CurvePoles {
            rational: bspline.is_rational(),
            pole: &pole,
            weight: &weight,
        },
        knot: &knot,
        multiplicity: &multiplicity,
    };
    print_bspline(out, &spline, KnotLayout::Curve3d)
}

/// `Print(Geom_OffsetCurve)`: the offset, the reference direction, then the basis
/// curve record.
fn print_offset(offset: &Handle<GeomOffsetCurve>, out: &mut TextOut<'_>) -> Result<(), Failure> {
    print_offset_head(out, offset.offset());
    out.label(b"Direction : ");
    out.coords(&direction3(offset.direction()));
    out.text(b"\n");
    out.label(b"Basis curve :\n");
    let compact = out.compact();
    print_curve(&offset.basis_curve(), out.stream(), compact)
}
