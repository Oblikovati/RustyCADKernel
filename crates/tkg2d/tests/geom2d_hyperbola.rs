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

//! Direct translation of Geom2d_Hyperbola_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dConic, Geom2dCurve, Geom2dGeometry, Geom2dHyperbola};
use rustycad_tkmath::gp::{Axis2d, Direction2, Hyperbola2, Point2, Transform2, Vector2};

// Geom2d_HyperbolaTest::SetUp: hyperbola centered at origin, a=5, b=3.
fn setup() -> Handle<Geom2dHyperbola> {
    let aHypr = Hyperbola2::from_x_axis(
        Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap()),
        5.0,
        3.0,
        true,
    )
    .unwrap();
    Handle::new(Geom2dHyperbola::from_hypr2d(aHypr))
}

#[test]
fn Geom2d_HyperbolaTest__Radii() {
    let myHyperbola = setup();
    EXPECT_NEAR(myHyperbola.major_radius(), 5.0, precision::confusion());
    EXPECT_NEAR(myHyperbola.minor_radius(), 3.0, precision::confusion());
}

#[test]
fn Geom2d_HyperbolaTest__IsNotClosedNotPeriodic() {
    let myHyperbola = setup();
    assert!(!myHyperbola.is_closed().unwrap());
    assert!(!myHyperbola.is_periodic());
}

#[test]
fn Geom2d_HyperbolaTest__Eccentricity() {
    let myHyperbola = setup();
    // e = c/a where c = sqrt(a^2 + b^2) = sqrt(25+9) = sqrt(34)
    let anExpected: f64 = 34.0_f64.sqrt() / 5.0;
    EXPECT_NEAR(
        myHyperbola.eccentricity().unwrap(),
        anExpected,
        precision::confusion(),
    );
}

#[test]
fn Geom2d_HyperbolaTest__Focal() {
    let myHyperbola = setup();
    // 2c = 2*sqrt(a^2 + b^2)
    let anExpected: f64 = 2.0 * (25.0 + 9.0_f64).sqrt();
    EXPECT_NEAR(myHyperbola.focal(), anExpected, precision::confusion());
}

#[test]
fn Geom2d_HyperbolaTest__Foci() {
    let myHyperbola = setup();
    let c: f64 = (25.0 + 9.0_f64).sqrt();
    let aF1: Point2 = myHyperbola.focus1();
    let aF2: Point2 = myHyperbola.focus2();

    EXPECT_NEAR(aF1.x(), c, precision::confusion());
    EXPECT_NEAR(aF2.x(), -c, precision::confusion());
}

#[test]
fn Geom2d_HyperbolaTest__Parameter() {
    let myHyperbola = setup();
    // Semi-latus rectum = b^2/a = 9/5 = 1.8
    EXPECT_NEAR(
        myHyperbola.parameter().unwrap(),
        1.8,
        precision::confusion(),
    );
}

#[test]
fn Geom2d_HyperbolaTest__EvalD0_AtZero() {
    let myHyperbola = setup();
    // At U=0: vertex = (a, 0) = (5, 0) since x = a*cosh(0) = a, y = b*sinh(0) = 0
    let aPnt: Point2 = myHyperbola.eval_d0(0.0).unwrap();
    EXPECT_NEAR(aPnt.x(), 5.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_HyperbolaTest__EvalD1_AtZero() {
    let myHyperbola = setup();
    let aD1: Vector2 = myHyperbola.eval_d1(0.0).unwrap().d1;
    // At U=0: dx/du = a*sinh(0) = 0, dy/du = b*cosh(0) = b = 3
    EXPECT_NEAR(aD1.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aD1.y(), 3.0, precision::confusion());
}

#[test]
fn Geom2d_HyperbolaTest__PointOnHyperbola_FociDistanceDifference() {
    let myHyperbola = setup();
    // For any point on hyperbola: |d(P,F1) - d(P,F2)| = 2a
    let aF1: Point2 = myHyperbola.focus1();
    let aF2: Point2 = myHyperbola.focus2();

    let mut u: f64 = -2.0;
    while u <= 2.0 {
        let aPnt: Point2 = myHyperbola.eval_d0(u).unwrap();
        let aDiff: f64 = (aPnt.distance(aF1) - aPnt.distance(aF2)).abs();
        EXPECT_NEAR(aDiff, 2.0 * 5.0, precision::confusion());
        u += 0.5;
    }
}

#[test]
fn Geom2d_HyperbolaTest__SetMajorRadius() {
    let myHyperbola = setup();
    myHyperbola.set_major_radius(8.0).unwrap();
    EXPECT_NEAR(myHyperbola.major_radius(), 8.0, precision::confusion());
}

#[test]
fn Geom2d_HyperbolaTest__SetMinorRadius() {
    let myHyperbola = setup();
    myHyperbola.set_minor_radius(4.0).unwrap();
    EXPECT_NEAR(myHyperbola.minor_radius(), 4.0, precision::confusion());
}

#[test]
fn Geom2d_HyperbolaTest__Copy() {
    let myHyperbola = setup();
    let aCopyGeom: Handle<dyn Geom2dGeometry> = myHyperbola.copy().unwrap();
    let aCopy: Handle<Geom2dHyperbola> = aCopyGeom.cast::<Geom2dHyperbola>();

    assert!(!aCopy.is_null());
    EXPECT_NEAR(aCopy.major_radius(), 5.0, precision::confusion());
    EXPECT_NEAR(aCopy.minor_radius(), 3.0, precision::confusion());
}

#[test]
fn Geom2d_HyperbolaTest__Transform_Translation() {
    let myHyperbola = setup();
    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(10.0, 20.0));
    myHyperbola.transform(aTrsf).unwrap();

    EXPECT_NEAR(myHyperbola.location().x(), 10.0, precision::confusion());
    EXPECT_NEAR(myHyperbola.location().y(), 20.0, precision::confusion());
}

#[test]
fn Geom2d_HyperbolaTest__Asymptotes() {
    let myHyperbola = setup();
    let anAsym1: Axis2d = myHyperbola.asymptote1().unwrap();
    let anAsym2: Axis2d = myHyperbola.asymptote2().unwrap();

    // Asymptotes pass through center
    EXPECT_NEAR(anAsym1.location().x(), 0.0, precision::confusion());
    EXPECT_NEAR(anAsym2.location().x(), 0.0, precision::confusion());
}
