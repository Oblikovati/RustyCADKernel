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

//! Complete ElCLib legacy suite, preserving every assertion.
#![allow(non_snake_case)]
use rustycad_tkernel::precision::confusion;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Axis2, Axis22d, Circle2, Circle3, Direction2, Direction3, Ellipse3, Hyperbola3, Line3,
    Parabola3, Point2, Point3, Vector2, Vector3,
};
use std::f64::consts::PI;

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}
fn checkPointsEqual(theP1: Point3, theP2: Point3, theTolerance: f64) {
    near(theP1.x(), theP2.x(), theTolerance);
    near(theP1.y(), theP2.y(), theTolerance);
    near(theP1.z(), theP2.z(), theTolerance);
}
fn checkVectorsEqual(theV1: Vector3, theV2: Vector3, theTolerance: f64) {
    near(theV1.x(), theV2.x(), theTolerance);
    near(theV1.y(), theV2.y(), theTolerance);
    near(theV1.z(), theV2.z(), theTolerance);
}
fn checkDirectorsEqual(theD1: Direction3, theD2: Direction3, theTolerance: f64) {
    near(theD1.x(), theD2.x(), theTolerance);
    near(theD1.y(), theD2.y(), theTolerance);
    near(theD1.z(), theD2.z(), theTolerance);
}
fn standard_frame(location: Point3) -> Axis2 {
    Axis2::with_x_direction(location, Direction3::Z, Direction3::X).unwrap()
}

#[test]
fn ElClibTests__InPeriod() {
    let PI2 = 2.0 * PI;
    near(elclib::in_period(0.5, 0.0, PI2), 0.5, confusion());
    near(elclib::in_period(PI2 + 0.5, 0.0, PI2), 0.5, confusion());
    near(elclib::in_period(-0.5, 0.0, PI2), PI2 - 0.5, confusion());
    near(
        elclib::in_period(-PI2 - 0.5, 0.0, PI2),
        PI2 - 0.5,
        confusion(),
    );
    near(elclib::in_period(1.7, 1.5, 4.5), 1.7, confusion());
    near(elclib::in_period(4.7, 1.5, 4.5), 1.7, confusion());
    near(elclib::in_period(7.7, 1.5, 4.5), 1.7, confusion());
    near(elclib::in_period(1.3, 1.5, 4.5), 4.3, confusion());
}

#[test]
fn ElClibTests__AdjustPeriodic() {
    let PI2 = 2.0 * PI;
    let (U1, U2) = elclib::adjust_periodic(0.0, PI2, confusion(), 0.5, 0.7);
    near(U1, 0.5, confusion());
    near(U2, 0.7, confusion());
    let (U1, U2) = elclib::adjust_periodic(0.0, PI2, confusion(), 0.5, 0.5 + PI2 + 0.2);
    near(U1, 0.5, confusion());
    near(U2, 0.7, confusion());
    let (U1, U2) = elclib::adjust_periodic(0.0, PI2, confusion(), 0.5 + PI2, 0.7 + PI2);
    near(U1, 0.5, confusion());
    near(U2, 0.7, confusion());
    let (U1, U2) = elclib::adjust_periodic(0.0, PI2, confusion(), -0.5, 0.7);
    near(U1, PI2 - 0.5, confusion());
    near(U2, 0.7 + PI2, confusion());
    let (U1, U2) = elclib::adjust_periodic(0.0, PI2, confusion(), 1.0, 1.0 + 0.5 * confusion());
    near(U1, 1.0, confusion());
    near(U2, 1.0 + PI2, confusion());
}

#[test]
fn ElClibTests__Line3D() {
    let aLoc = Point3::new(1.0, 2.0, 3.0);
    let aDir = Direction3::Z;
    let aLin = Line3::new(aLoc, aDir);
    let aParam = 5.0;
    let aPoint = elclib::value(aParam, &aLin);
    let aExpectedPoint = Point3::new(1.0, 2.0, 8.0);
    checkPointsEqual(aPoint, aExpectedPoint, confusion());
    let (aPointD1, aVecD1) = elclib::d1(aParam, &aLin);
    checkPointsEqual(aPointD1, aExpectedPoint, confusion());
    checkVectorsEqual(aVecD1, Vector3::from(aDir), confusion());
    let aVecDN = elclib::dn(aParam, &aLin, 1);
    checkVectorsEqual(aVecDN, Vector3::from(aDir), confusion());
    let aVecDN2 = elclib::dn(aParam, &aLin, 2);
    checkVectorsEqual(aVecDN2, Vector3::new(0.0, 0.0, 0.0), confusion());
    let aTestPoint = Point3::new(1.0, 2.0, 10.0);
    let aCalculatedParam = elclib::parameter(&aLin, aTestPoint).unwrap();
    near(aCalculatedParam, 7.0, confusion());
}

#[test]
fn ElClibTests__Circle3D() {
    let anAxis = standard_frame(Point3::new(0.0, 0.0, 0.0));
    let aRadius = 2.0;
    let aCircle = Circle3::new(anAxis, aRadius).unwrap();
    let aParam = PI / 4.0;
    let aPoint = elclib::value(aParam, &aCircle);
    let aExpectedPoint = Point3::new(aRadius * aParam.cos(), aRadius * aParam.sin(), 0.0);
    checkPointsEqual(aPoint, aExpectedPoint, confusion());
    let (aPointD1, aVecD1) = elclib::d1(aParam, &aCircle);
    checkPointsEqual(aPointD1, aExpectedPoint, confusion());
    let aExpectedVecD1 = Vector3::new(-aRadius * aParam.sin(), aRadius * aParam.cos(), 0.0);
    checkVectorsEqual(aVecD1, aExpectedVecD1, confusion());
    let (aPointD2, aVecD2_1, aVecD2_2) = elclib::d2(aParam, &aCircle);
    checkPointsEqual(aPointD2, aExpectedPoint, confusion());
    checkVectorsEqual(aVecD2_1, aExpectedVecD1, confusion());
    let aExpectedVecD2_2 = Vector3::new(-aRadius * aParam.cos(), -aRadius * aParam.sin(), 0.0);
    checkVectorsEqual(aVecD2_2, aExpectedVecD2_2, confusion());
    let (aPointD3, aVecD3_1, aVecD3_2, aVecD3_3) = elclib::d3(aParam, &aCircle);
    checkPointsEqual(aPointD3, aExpectedPoint, confusion());
    checkVectorsEqual(aVecD3_1, aExpectedVecD1, confusion());
    checkVectorsEqual(aVecD3_2, aExpectedVecD2_2, confusion());
    let aExpectedVecD3_3 = Vector3::new(aRadius * aParam.sin(), -aRadius * aParam.cos(), 0.0);
    checkVectorsEqual(aVecD3_3, aExpectedVecD3_3, confusion());
    let aVecDN1 = elclib::dn(aParam, &aCircle, 1);
    checkVectorsEqual(aVecDN1, aExpectedVecD1, confusion());
    let aVecDN2 = elclib::dn(aParam, &aCircle, 2);
    checkVectorsEqual(aVecDN2, aExpectedVecD2_2, confusion());
    let aVecDN3 = elclib::dn(aParam, &aCircle, 3);
    checkVectorsEqual(aVecDN3, aExpectedVecD3_3, confusion());
    let aCalculatedParam = elclib::parameter(&aCircle, aExpectedPoint).unwrap();
    near(aCalculatedParam, aParam, confusion());
}

#[test]
fn ElClibTests__Ellipse3D() {
    let anAxis = standard_frame(Point3::new(0.0, 0.0, 0.0));
    let aMajorRadius = 3.0;
    let aMinorRadius = 2.0;
    let anEllipse = Ellipse3::new(anAxis, aMajorRadius, aMinorRadius).unwrap();
    let aParam = PI / 4.0;
    let aPoint = elclib::value(aParam, &anEllipse);
    let aExpectedPoint = Point3::new(
        aMajorRadius * aParam.cos(),
        aMinorRadius * aParam.sin(),
        0.0,
    );
    checkPointsEqual(aPoint, aExpectedPoint, confusion());
    let (aPointD1, aVecD1) = elclib::d1(aParam, &anEllipse);
    checkPointsEqual(aPointD1, aExpectedPoint, confusion());
    let aExpectedVecD1 = Vector3::new(
        -aMajorRadius * aParam.sin(),
        aMinorRadius * aParam.cos(),
        0.0,
    );
    checkVectorsEqual(aVecD1, aExpectedVecD1, confusion());
    let aCalculatedParam = elclib::parameter(&anEllipse, aExpectedPoint).unwrap();
    near(aCalculatedParam, aParam, confusion());
}

#[test]
fn ElClibTests__Hyperbola3D() {
    let anAxis = standard_frame(Point3::new(0.0, 0.0, 0.0));
    let aMajorRadius = 3.0;
    let aMinorRadius = 2.0;
    let aHyperbola = Hyperbola3::new(anAxis, aMajorRadius, aMinorRadius).unwrap();
    let aParam = 0.5;
    let aPoint = elclib::value(aParam, &aHyperbola);
    let aExpectedPoint = Point3::new(
        aMajorRadius * aParam.cosh(),
        aMinorRadius * aParam.sinh(),
        0.0,
    );
    checkPointsEqual(aPoint, aExpectedPoint, confusion());
    let (aPointD1, aVecD1) = elclib::d1(aParam, &aHyperbola);
    checkPointsEqual(aPointD1, aExpectedPoint, confusion());
    let aExpectedVecD1 = Vector3::new(
        aMajorRadius * aParam.sinh(),
        aMinorRadius * aParam.cosh(),
        0.0,
    );
    checkVectorsEqual(aVecD1, aExpectedVecD1, confusion());
    let aCalculatedParam = elclib::parameter(&aHyperbola, aExpectedPoint).unwrap();
    near(aCalculatedParam, aParam, confusion());
}

#[test]
fn ElClibTests__Parabola3D() {
    let anAxis = standard_frame(Point3::new(0.0, 0.0, 0.0));
    let aFocal = 2.0;
    let aParabola = Parabola3::new(anAxis, aFocal).unwrap();
    let aParam = 3.0;
    let aPoint = elclib::value(aParam, &aParabola);
    let aExpectedPoint = Point3::new(aParam * aParam / (4.0 * aFocal), aParam, 0.0);
    checkPointsEqual(aPoint, aExpectedPoint, confusion());
    let (aPointD1, aVecD1) = elclib::d1(aParam, &aParabola);
    checkPointsEqual(aPointD1, aExpectedPoint, confusion());
    let aExpectedVecD1 = Vector3::new(aParam / (2.0 * aFocal), 1.0, 0.0);
    checkVectorsEqual(aVecD1, aExpectedVecD1, confusion());
    let aCalculatedParam = elclib::parameter(&aParabola, aExpectedPoint).unwrap();
    near(aCalculatedParam, aParam, confusion());
}

#[test]
fn ElClibTests__To3dConversion() {
    let aLoc = Point3::new(1.0, 2.0, 3.0);
    let anAxis = standard_frame(aLoc);
    let aPnt2d = Point2::new(2.0, 3.0);
    let aPnt3d = elclib::to3d_point(anAxis, aPnt2d);
    let aExpectedPnt3d = Point3::new(3.0, 5.0, 3.0);
    checkPointsEqual(aPnt3d, aExpectedPnt3d, confusion());
    let aVec2d = Vector2::new(1.0, 2.0);
    let aVec3d = elclib::to3d_vector(anAxis, aVec2d);
    let aExpectedVec3d = Vector3::new(1.0, 2.0, 0.0);
    checkVectorsEqual(aVec3d, aExpectedVec3d, confusion());
    let aDir2d = Direction2::new(1.0, 2.0).unwrap();
    let aDir3d = elclib::to3d_direction(anAxis, aDir2d).unwrap();
    let aExpectedDir3d = Direction3::new(1.0 / 5.0_f64.sqrt(), 2.0 / 5.0_f64.sqrt(), 0.0).unwrap();
    checkDirectorsEqual(aDir3d, aExpectedDir3d, confusion());
    let aLoc2d = Point2::new(0.0, 0.0);
    let aDir2dX = Direction2::X;
    let anAxis2d = Axis22d::new(aLoc2d, aDir2dX, true);
    let aRadius = 2.0;
    let aCirc2d = Circle2::new(anAxis2d, aRadius).unwrap();
    let aCirc3d = elclib::to3d_circle(anAxis, aCirc2d).unwrap();
    near(aCirc3d.radius(), aRadius, confusion());
    checkPointsEqual(aCirc3d.location(), aLoc, confusion());
}

#[test]
fn ElClibTests__Parabola3D_NearZeroFocal_DegeneratesToLine() {
    let anAxis = Axis2::with_x_direction(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let aFocal = 0.5 * f64::MIN_POSITIVE;
    let aU = 4.0;
    let aValue = elclib::parabola_value(aU, anAxis, aFocal);
    checkPointsEqual(aValue, Point3::new(1.0 + aU, 2.0, 3.0), confusion());
    let (aP, aD1) = elclib::parabola_d1(aU, anAxis, aFocal);
    checkPointsEqual(aP, Point3::new(1.0 + aU, 2.0, 3.0), confusion());
    checkVectorsEqual(aD1, Vector3::new(1.0, 0.0, 0.0), confusion());
    let (_aP, _aD1, aD2) = elclib::parabola_d2(aU, anAxis, aFocal);
    checkVectorsEqual(aD2, Vector3::new(0.0, 0.0, 0.0), confusion());
    checkVectorsEqual(
        elclib::parabola_dn(aU, anAxis, aFocal, 1),
        Vector3::new(1.0, 0.0, 0.0),
        confusion(),
    );
    checkVectorsEqual(
        elclib::parabola_dn(aU, anAxis, aFocal, 2),
        Vector3::new(0.0, 0.0, 0.0),
        confusion(),
    );
}

#[test]
fn ElClibTests__Parabola2D_NearZeroFocal_DegeneratesToLine() {
    let anAxis = Axis22d::new(
        Point2::new(1.0, 2.0),
        Direction2::new(1.0, 0.0).unwrap(),
        true,
    );
    let aFocal = 0.5 * f64::MIN_POSITIVE;
    let aU = 3.0;
    let aValue = elclib::parabola_value_2d(aU, anAxis, aFocal);
    near(aValue.x(), 1.0 + aU, confusion());
    near(aValue.y(), 2.0, confusion());
    let (aP, aD1) = elclib::parabola_d1_2d(aU, anAxis, aFocal);
    near(aP.x(), 1.0 + aU, confusion());
    near(aP.y(), 2.0, confusion());
    near(aD1.x(), 1.0, confusion());
    near(aD1.y(), 0.0, confusion());
    let (_aP, _aD1, aD2) = elclib::parabola_d2_2d(aU, anAxis, aFocal);
    near(aD2.x(), 0.0, confusion());
    near(aD2.y(), 0.0, confusion());
    let aDN1 = elclib::parabola_dn_2d(aU, anAxis, aFocal, 1);
    let aDN2 = elclib::parabola_dn_2d(aU, anAxis, aFocal, 2);
    near(aDN1.x(), 1.0, confusion());
    near(aDN1.y(), 0.0, confusion());
    near(aDN2.x(), 0.0, confusion());
    near(aDN2.y(), 0.0, confusion());
}

#[test]
fn ElClibTests__Parabola3D_AboveResolutionFocal_NotDegenerate() {
    let anAxis = Axis2::with_x_direction(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let aFocal = 2.0 * f64::MIN_POSITIVE;
    let aU = 1.0;
    let aValue = elclib::parabola_value(aU, anAxis, aFocal);
    assert!((aValue.x() - (1.0 + aU)).abs() > 1.0);
    let aD2 = elclib::parabola_dn(aU, anAxis, aFocal, 2);
    assert!(aD2.magnitude() > 1.0);
}

#[test]
fn ElClibTests__Parabola2D_AboveResolutionFocal_NotDegenerate() {
    let anAxis = Axis22d::new(
        Point2::new(1.0, 2.0),
        Direction2::new(1.0, 0.0).unwrap(),
        true,
    );
    let aFocal = 2.0 * f64::MIN_POSITIVE;
    let aU = 1.0;
    let aValue = elclib::parabola_value_2d(aU, anAxis, aFocal);
    assert!((aValue.x() - (1.0 + aU)).abs() > 1.0);
    let aD2 = elclib::parabola_dn_2d(aU, anAxis, aFocal, 2);
    assert!(aD2.magnitude() > 1.0);
}
