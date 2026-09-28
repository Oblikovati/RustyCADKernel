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

//! Direct translation of Geom2d_TrimmedCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBoundedCurve, Geom2dCircle, Geom2dCurve, Geom2dGeometry, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Point2, Transform2, Vector2};
use std::f64::consts::PI;

// Geom2d_TrimmedCurveTest fixture: a line along X and a circle of radius 5 at the origin.
struct Fixture {
    myLine: Handle<Geom2dLine>,
    myCircle: Handle<Geom2dCircle>,
}

fn setup() -> Fixture {
    // Line through origin along X
    let myLine = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    ));

    // Circle centered at origin, radius 5
    let aCirc = Circle2::from_x_axis(
        Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap()),
        5.0,
        true,
    )
    .unwrap();
    let myCircle = Handle::new(Geom2dCircle::from_circ2d(aCirc));
    Fixture { myLine, myCircle }
}

// `new Geom2d_TrimmedCurve(C, U1, U2)` with the default Sense and theAdjustPeriodic.
fn trim<T: Geom2dCurve + rustycad_tkernel::standard::RuntimeTyped>(
    curve: &Handle<T>,
    u1: f64,
    u2: f64,
) -> Handle<Geom2dTrimmedCurve> {
    Handle::new(Geom2dTrimmedCurve::new(&curve.cast(), u1, u2, true, true).unwrap())
}

#[test]
fn Geom2d_TrimmedCurveTest__ConstructFromLine() {
    let f = setup();
    let aTrim = trim(&f.myLine, 0.0, 10.0);

    EXPECT_DOUBLE_EQ(aTrim.first_parameter(), 0.0);
    EXPECT_DOUBLE_EQ(aTrim.last_parameter(), 10.0);
}

#[test]
fn Geom2d_TrimmedCurveTest__ConstructFromCircle() {
    let f = setup();
    let aTrim = trim(&f.myCircle, 0.0, PI);

    EXPECT_DOUBLE_EQ(aTrim.first_parameter(), 0.0);
    EXPECT_DOUBLE_EQ(aTrim.last_parameter(), PI);
}

#[test]
fn Geom2d_TrimmedCurveTest__StartAndEndPoints() {
    let f = setup();
    let aTrim = trim(&f.myLine, 2.0, 8.0);

    let aStart: Point2 = aTrim.start_point().unwrap();
    let anEnd: Point2 = aTrim.end_point().unwrap();

    EXPECT_NEAR(aStart.x(), 2.0, precision::confusion());
    EXPECT_NEAR(anEnd.x(), 8.0, precision::confusion());
}

#[test]
fn Geom2d_TrimmedCurveTest__BasisCurve() {
    let f = setup();
    let aTrim = trim(&f.myCircle, 0.0, PI);

    let aBasis: Handle<dyn Geom2dCurve> = aTrim.basis_curve();
    assert!(!aBasis.is_null());
}

#[test]
fn Geom2d_TrimmedCurveTest__EvalD0_MatchesBasisCurve() {
    let f = setup();
    let aTrim = trim(&f.myCircle, 0.0, PI);

    let mut u: f64 = 0.0;
    while u <= PI {
        let aTrimPnt: Point2 = aTrim.eval_d0(u).unwrap();
        let aBasePnt: Point2 = f.myCircle.eval_d0(u).unwrap();
        assert!(aTrimPnt.is_equal(aBasePnt, precision::confusion()));
        u += PI / 4.0;
    }
}

#[test]
fn Geom2d_TrimmedCurveTest__EvalD1_MatchesBasisCurve() {
    let f = setup();
    let aTrim = trim(&f.myLine, 0.0, 10.0);

    let aTrimRes = aTrim.eval_d1(5.0).unwrap();
    let aBaseRes = f.myLine.eval_d1(5.0).unwrap();
    let (aTrimPnt, aTrimD1): (Point2, Vector2) = (aTrimRes.point, aTrimRes.d1);
    let (aBasePnt, aBaseD1): (Point2, Vector2) = (aBaseRes.point, aBaseRes.d1);

    assert!(aTrimPnt.is_equal(aBasePnt, precision::confusion()));
    EXPECT_NEAR(aTrimD1.x(), aBaseD1.x(), precision::confusion());
    EXPECT_NEAR(aTrimD1.y(), aBaseD1.y(), precision::confusion());
}

#[test]
fn Geom2d_TrimmedCurveTest__Continuity() {
    let f = setup();
    let aTrim = trim(&f.myLine, 0.0, 10.0);
    assert_eq!(aTrim.continuity(), Shape::CN);
}

#[test]
fn Geom2d_TrimmedCurveTest__IsCN() {
    let f = setup();
    let aTrim = trim(&f.myLine, 0.0, 10.0);
    assert!(aTrim.is_cn(10).unwrap());
}

#[test]
fn Geom2d_TrimmedCurveTest__IsClosed_OpenArc() {
    let f = setup();
    let aTrim = trim(&f.myCircle, 0.0, PI);
    assert!(!aTrim.is_closed().unwrap());
}

#[test]
fn Geom2d_TrimmedCurveTest__IsClosed_FullCircle() {
    let f = setup();
    // Full-period trim on a periodic curve should be reported as closed
    let aTrim = trim(&f.myCircle, 0.0, 2.0 * PI);

    assert!(aTrim.is_closed().unwrap());
}

#[test]
fn Geom2d_TrimmedCurveTest__IsPeriodic_PartialArc() {
    let f = setup();
    let aTrim = trim(&f.myCircle, 0.0, PI);
    assert!(!aTrim.is_periodic());
}

#[test]
fn Geom2d_TrimmedCurveTest__IsPeriodic_FullPeriod() {
    let f = setup();
    // Full-period trim on a periodic curve should be reported as periodic
    let aTrim = trim(&f.myCircle, 0.0, 2.0 * PI);
    assert!(aTrim.is_periodic());
}

#[test]
fn Geom2d_TrimmedCurveTest__IsPeriodic_Line() {
    let f = setup();
    // Line is not periodic, so trimmed line should not be periodic
    let aTrim = trim(&f.myLine, 0.0, 10.0);
    assert!(!aTrim.is_periodic());
}

#[test]
fn Geom2d_TrimmedCurveTest__SetTrim() {
    let f = setup();
    let aTrim = trim(&f.myLine, 0.0, 10.0);
    aTrim.set_trim(3.0, 7.0, true, true).unwrap();

    EXPECT_DOUBLE_EQ(aTrim.first_parameter(), 3.0);
    EXPECT_DOUBLE_EQ(aTrim.last_parameter(), 7.0);
}

#[test]
fn Geom2d_TrimmedCurveTest__Copy() {
    let f = setup();
    let aTrim = trim(&f.myCircle, 0.0, PI);
    let aCopyGeom: Handle<dyn Geom2dGeometry> = aTrim.copy().unwrap();
    let aCopy: Handle<Geom2dTrimmedCurve> = aCopyGeom.cast::<Geom2dTrimmedCurve>();

    assert!(!aCopy.is_null());
    EXPECT_DOUBLE_EQ(aCopy.first_parameter(), 0.0);
    EXPECT_DOUBLE_EQ(aCopy.last_parameter(), PI);
}

#[test]
fn Geom2d_TrimmedCurveTest__Transform_Translation() {
    let f = setup();
    let aTrim = trim(&f.myLine, 0.0, 10.0);

    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(5.0, 5.0));
    aTrim.transform(aTrsf).unwrap();

    let aStart: Point2 = aTrim.start_point().unwrap();
    EXPECT_NEAR(aStart.x(), 5.0, precision::confusion());
    EXPECT_NEAR(aStart.y(), 5.0, precision::confusion());
}

#[test]
fn Geom2d_TrimmedCurveTest__CircleArc_StartEndPoints() {
    let f = setup();
    // Quarter circle arc
    let aTrim = trim(&f.myCircle, 0.0, PI / 2.0);

    let aStart: Point2 = aTrim.start_point().unwrap();
    let anEnd: Point2 = aTrim.end_point().unwrap();

    EXPECT_NEAR(aStart.x(), 5.0, precision::confusion());
    EXPECT_NEAR(aStart.y(), 0.0, precision::confusion());
    EXPECT_NEAR(anEnd.x(), 0.0, precision::confusion());
    EXPECT_NEAR(anEnd.y(), 5.0, precision::confusion());
}
