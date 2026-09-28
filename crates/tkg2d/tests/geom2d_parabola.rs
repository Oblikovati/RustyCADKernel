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

//! Direct translation of Geom2d_Parabola_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dConic, Geom2dCurve, Geom2dGeometry, Geom2dParabola};
use rustycad_tkmath::gp::{Axis2d, Direction2, Line2, Parabola2, Point2, Transform2, Vector2};

// Geom2d_ParabolaTest::SetUp: parabola centered at origin with focal distance 4.
fn setup() -> Handle<Geom2dParabola> {
    let aParab = Parabola2::from_mirror_axis(
        Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap()),
        4.0,
        true,
    )
    .unwrap();
    Handle::new(Geom2dParabola::from_parab2d(aParab))
}

#[test]
fn Geom2d_ParabolaTest__Focal() {
    let myParabola = setup();
    EXPECT_NEAR(myParabola.focal(), 4.0, precision::confusion());
}

#[test]
fn Geom2d_ParabolaTest__IsNotClosedNotPeriodic() {
    let myParabola = setup();
    assert!(!myParabola.is_closed().unwrap());
    assert!(!myParabola.is_periodic());
}

#[test]
fn Geom2d_ParabolaTest__Eccentricity() {
    let myParabola = setup();
    EXPECT_NEAR(
        myParabola.eccentricity().unwrap(),
        1.0,
        precision::confusion(),
    );
}

#[test]
fn Geom2d_ParabolaTest__Focus() {
    let myParabola = setup();
    let aFocus: Point2 = myParabola.focus();
    EXPECT_NEAR(aFocus.x(), 4.0, precision::confusion());
    EXPECT_NEAR(aFocus.y(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_ParabolaTest__Parameter() {
    let myParabola = setup();
    // p = 2*Focal = 8
    EXPECT_NEAR(myParabola.parameter(), 8.0, precision::confusion());
}

#[test]
fn Geom2d_ParabolaTest__EvalD0_AtZero() {
    let myParabola = setup();
    // At U=0: vertex
    let aPnt: Point2 = myParabola.eval_d0(0.0).unwrap();
    EXPECT_NEAR(aPnt.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_ParabolaTest__EvalD0_Symmetric() {
    let myParabola = setup();
    // Parabola is symmetric: P(U) and P(-U) should have same X, opposite Y
    let aU: f64 = 3.0;
    let aPntPos: Point2 = myParabola.eval_d0(aU).unwrap();
    let aPntNeg: Point2 = myParabola.eval_d0(-aU).unwrap();

    EXPECT_NEAR(aPntPos.x(), aPntNeg.x(), precision::confusion());
    EXPECT_NEAR(aPntPos.y(), -aPntNeg.y(), precision::confusion());
}

#[test]
fn Geom2d_ParabolaTest__EvalD1_AtZero() {
    let myParabola = setup();
    let aD1: Vector2 = myParabola.eval_d1(0.0).unwrap().d1;
    // At vertex: tangent is along Y axis
    EXPECT_NEAR(aD1.x(), 0.0, precision::confusion());
    assert!(aD1.magnitude() > precision::confusion());
}

#[test]
fn Geom2d_ParabolaTest__EvalD2_AtZero() {
    let myParabola = setup();
    let aD2: Vector2 = myParabola.eval_d2(0.0).unwrap().d2;
    // Second derivative at vertex should be along X (opening direction)
    assert!(aD2.x() > 0.0);
}

#[test]
fn Geom2d_ParabolaTest__PointOnParabola_FocusDirectrixProperty() {
    let myParabola = setup();
    // For any point P on parabola: distance(P, Focus) = distance(P, Directrix)
    let aFocus: Point2 = myParabola.focus();
    let aDirectrix: Axis2d = myParabola.directrix().unwrap();
    let aDirLine = Line2::from(aDirectrix);

    let mut u: f64 = -5.0;
    while u <= 5.0 {
        let aPnt: Point2 = myParabola.eval_d0(u).unwrap();
        let aDistFocus: f64 = aPnt.distance(aFocus);
        let aDistDir: f64 = aDirLine.distance(aPnt);
        EXPECT_NEAR(aDistFocus, aDistDir, 1e-6);
        u += 1.0;
    }
}

#[test]
fn Geom2d_ParabolaTest__SetFocal() {
    let myParabola = setup();
    myParabola.set_focal(10.0).unwrap();
    EXPECT_NEAR(myParabola.focal(), 10.0, precision::confusion());
}

#[test]
fn Geom2d_ParabolaTest__Copy() {
    let myParabola = setup();
    let aCopyGeom: Handle<dyn Geom2dGeometry> = myParabola.copy().unwrap();
    let aCopy: Handle<Geom2dParabola> = aCopyGeom.cast::<Geom2dParabola>();

    assert!(!aCopy.is_null());
    EXPECT_NEAR(aCopy.focal(), 4.0, precision::confusion());
}

#[test]
fn Geom2d_ParabolaTest__Transform_Translation() {
    let myParabola = setup();
    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(5.0, 10.0));
    myParabola.transform(aTrsf).unwrap();

    EXPECT_NEAR(myParabola.location().x(), 5.0, precision::confusion());
    EXPECT_NEAR(myParabola.location().y(), 10.0, precision::confusion());
}
