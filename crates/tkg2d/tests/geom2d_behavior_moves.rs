// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Geom2d_Geometry moves, their copying twins and DumpJson of every class. The
//! expected JSON text is the output of the native oracle
//! (target/oracle-tkg2d/numeric/native-probe, `g2_curve` requests).
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dConic, Geom2dCurve, Geom2dEllipse,
    Geom2dGeometry, Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola,
    Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2, Vector2};
use std::f64::consts::FRAC_PI_2;

fn axis(x: f64, y: f64) -> Axis2d {
    Axis2d::new(Point2::new(x, y), Direction2::new(1.0, 0.0).unwrap())
}

fn circle() -> Handle<Geom2dCircle> {
    Handle::new(Geom2dCircle::from_axis(axis(0.0, 0.0), 2.0, true).unwrap())
}

fn dump(geometry: &dyn Geom2dGeometry, depth: i32) -> String {
    let mut stream = DumpStream::default();
    geometry.dump_json(&mut stream, depth).unwrap();
    stream.text().to_string()
}

fn center(geometry: &Handle<dyn Geom2dGeometry>) -> Point2 {
    geometry.cast::<dyn Geom2dConic>().location()
}

fn assert_point(actual: Point2, x: f64, y: f64) {
    EXPECT_NEAR(actual.x(), x, 1e-14);
    EXPECT_NEAR(actual.y(), y, 1e-14);
}

#[test]
fn copying_moves_leave_the_source_unchanged() {
    let c = circle();
    let p = Point2::new(1.0, 1.0);
    assert_point(center(&c.mirrored_point(p).unwrap()), 2.0, 2.0);
    let mirror = Axis2d::new(Point2::new(0.0, 1.0), Direction2::new(1.0, 0.0).unwrap());
    assert_point(center(&c.mirrored_axis(mirror).unwrap()), 0.0, 2.0);
    assert_point(center(&c.rotated(p, FRAC_PI_2).unwrap()), 2.0, 0.0);
    let scaled = c.scaled(p, 3.0).unwrap();
    assert_point(center(&scaled), -2.0, -2.0);
    assert_eq!(scaled.cast::<Geom2dCircle>().radius(), 6.0);
    assert_point(
        center(&c.translated(Vector2::new(1.0, -1.0)).unwrap()),
        1.0,
        -1.0,
    );
    assert_point(
        center(&c.translated_between(p, Point2::new(3.0, 1.0)).unwrap()),
        2.0,
        0.0,
    );
    let t = Transform2::translation([0.5, 0.5]);
    assert_point(center(&c.transformed(t).unwrap()), 0.5, 0.5);
    assert_point(c.location(), 0.0, 0.0);
}

#[test]
fn in_place_moves_change_every_handle() {
    let c = circle();
    let shared: Handle<dyn Geom2dGeometry> = c.cast();
    shared.mirror_point(Point2::new(1.0, 0.0)).unwrap();
    assert_point(c.location(), 2.0, 0.0);
    shared.mirror_axis(axis(0.0, 1.0)).unwrap();
    assert_point(c.location(), 2.0, 2.0);
    shared.rotate(Point2::new(0.0, 0.0), FRAC_PI_2).unwrap();
    assert_point(c.location(), -2.0, 2.0);
    // Scaling by a negative factor multiplies the radius by its absolute value.
    shared.scale(Point2::new(0.0, 0.0), -0.5).unwrap();
    assert_point(c.location(), 1.0, -1.0);
    assert_eq!(c.radius(), 1.0);
    shared.translate(Vector2::new(1.0, 1.0)).unwrap();
    shared
        .translate_between(Point2::new(0.0, 0.0), Point2::new(0.0, 3.0))
        .unwrap();
    assert_point(c.location(), 2.0, 3.0);
}

#[test]
fn conics_dump_like_the_native_kernel() {
    let pos = r#""pos": {"Location": [0, 0], "XAxis": [1, 0], "YAxis": [-0, 1]}"#;
    let head = |class: &str| {
        format!(
            r#""className": "{class}", "className": "Geom2d_Conic", "className": "Geom2d_Curve", "className": "Geom2d_Geometry", {pos}"#
        )
    };
    assert_eq!(
        dump(&*circle(), -1),
        format!(r#"{}, "radius": 2"#, head("Geom2d_Circle"))
    );
    assert_eq!(
        dump(&*circle(), 1),
        r#""className": "Geom2d_Circle", "className": "Geom2d_Conic", "radius": 2"#
    );
    let ellipse = Geom2dEllipse::from_axis(axis(0.0, 0.0), 3.0, 1.0, true).unwrap();
    let radii = r#""majorRadius": 3, "minorRadius": 1"#;
    assert_eq!(
        dump(&ellipse, -1),
        format!("{}, {radii}", head("Geom2d_Ellipse"))
    );
    let hyperbola = Geom2dHyperbola::from_axis(axis(0.0, 0.0), 3.0, 1.0, true).unwrap();
    assert_eq!(
        dump(&hyperbola, -1),
        format!("{}, {radii}", head("Geom2d_Hyperbola"))
    );
    let parabola = Geom2dParabola::from_axis(axis(0.0, 0.0), 2.0, true).unwrap();
    assert_eq!(
        dump(&parabola, -1),
        format!(r#"{}, "focalLength": 2"#, head("Geom2d_Parabola"))
    );
}

#[test]
fn line_trimmed_and_offset_dump_like_the_native_kernel() {
    let line = Geom2dLine::new(axis(1.0, 2.0));
    assert_eq!(
        dump(&line, -1),
        r#""className": "Geom2d_Line", "className": "Geom2d_Curve", "className": "Geom2d_Geometry", "pos": {"className": "gp_Ax2d", "loc": {"gp_Pnt2d": [1, 2]}, "vdir": {"gp_Dir2d": [1, 0]}}"#
    );
    assert_eq!(
        dump(&line, 1),
        r#""className": "Geom2d_Line", "className": "Geom2d_Curve", "pos": {"className": "gp_Ax2d"}"#
    );
    let basis: Handle<dyn Geom2dCurve> = circle().cast();
    let trimmed = Geom2dTrimmedCurve::new(&basis, 0.5, 1.5, true, true).unwrap();
    assert_eq!(
        dump(&trimmed, 1),
        r#""className": "Geom2d_TrimmedCurve", "className": "Geom2d_BoundedCurve", "basisCurve": {"className": "Geom2d_Circle", "radius": 2}, "uTrim1": 0.5, "uTrim2": 1.5"#
    );
    let offset = Geom2dOffsetCurve::new(&basis, 0.5, false).unwrap();
    assert_eq!(
        dump(&offset, 1),
        r#""className": "Geom2d_OffsetCurve", "className": "Geom2d_Curve", "basisCurve": {"className": "Geom2d_Circle", "radius": 2}, "offsetValue": 0.5, "BasisCurveContinuity": 6"#
    );
    assert!(dump(&offset, -1).starts_with(
        r#""className": "Geom2d_OffsetCurve", "className": "Geom2d_Curve", "className": "Geom2d_Geometry", "basisCurve": {"className": "Geom2d_Circle", "className": "Geom2d_Conic""#
    ));
}

fn poles(values: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut poles = Array1::<Point2>::with_bounds(1, values.len() as i32).unwrap();
    for (i, (x, y)) in values.iter().enumerate() {
        poles[i as i32 + 1] = Point2::new(*x, *y);
    }
    poles
}

#[test]
fn bezier_and_bspline_dump_like_the_native_kernel() {
    let mut weights = Array1::<f64>::with_bounds(1, 3).unwrap();
    weights.as_mut_slice().copy_from_slice(&[1.0, 2.0, 1.0]);
    let bezier =
        Geom2dBezierCurve::with_weights(&poles(&[(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)]), &weights)
            .unwrap();
    assert_eq!(
        dump(&bezier, 1),
        r#""className": "Geom2d_BezierCurve", "className": "Geom2d_BoundedCurve", "Rational": 1, "Closed": 0, "Poles.Length": 3, "Weights.Length": 3, "MaxDerivInv": 0, "MaxDerivInvOk": 0"#
    );
    let mut knots = Array1::<f64>::with_bounds(1, 2).unwrap();
    knots.as_mut_slice().copy_from_slice(&[0.0, 1.0]);
    let mut mults = Array1::<i32>::with_bounds(1, 2).unwrap();
    mults.init(3);
    let bspline = Geom2dBSplineCurve::new(
        &poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]),
        &knots,
        &mults,
        2,
        false,
    )
    .unwrap();
    assert_eq!(
        dump(&bspline, 1),
        r#""className": "Geom2d_BSplineCurve", "className": "Geom2d_BoundedCurve", "Rational": 0, "Periodic": 0, "KnotSet": 3, "Smooth": 6, "Deg": 2, "Poles.Size": 3, "FlatKnots.Size": 6, "Knots.Size": 2, "Mults.Size": 2, "MaxDerivInv": 0, "MaxDerivInvOk": 0"#
    );
}
