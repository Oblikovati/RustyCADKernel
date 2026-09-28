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

//! Complete gp_Lin suite, including both gp_Lin2d regressions.
#![allow(
    non_snake_case,
    unused_mut,
    unused_assignments,
    unused_imports,
    clippy::excessive_precision
)]
use rustycad_tkmath::gp::{Axis1, Direction3, Line2, Line3, Point2, Point3, Transform3, Vector3};
#[test]
fn gp_LinTest__ConstructorFromAx1() {
    let mut anAx1 = Axis1::new(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aLin = Line3::from_axis(anAx1);
    assert!(aLin.location().is_equal(&Point3::new(1.0, 2.0, 3.0), 1e-7));
    assert!(
        aLin.direction()
            .is_equal(&Direction3::new(1.0, 0.0, 0.0).unwrap(), 1e-12)
    );
}

#[test]
fn gp_LinTest__ConstructorFromPointAndDir() {
    let mut aLin = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    assert!(aLin.location().is_equal(&Point3::new(0.0, 0.0, 0.0), 1e-7));
    assert!(
        aLin.direction()
            .is_equal(&Direction3::new(0.0, 0.0, 1.0).unwrap(), 1e-12)
    );
}

#[test]
fn gp_LinTest__DistanceToPoint() {
    let mut aLin = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aPnt = Point3::new(5.0, 3.0, 4.0);
    // Distance from point to X axis: sqrt(3^2 + 4^2) = 5
    assert!((aLin.distance(aPnt) - 5.0).abs() <= 1e-7);
}

#[test]
fn gp_LinTest__DistanceToLine_Parallel() {
    let mut aL1 = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aL2 = Line3::new(
        Point3::new(0.0, 3.0, 4.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    assert!((aL1.distance_line(aL2).unwrap() - 5.0).abs() <= 1e-7);
}

#[test]
fn gp_LinTest__DistanceToLine_Intersecting() {
    let mut aL1 = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aL2 = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
    );
    assert!((aL1.distance_line(aL2).unwrap() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_LinTest__Contains() {
    let mut aLin = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aOnLine = Point3::new(5.0, 0.0, 0.0);
    let mut aOffLine = Point3::new(5.0, 1.0, 0.0);
    assert!(aLin.contains(aOnLine, 1e-7));
    assert!(!(aLin.contains(aOffLine, 1e-7)));
}

#[test]
fn gp_LinTest__Angle() {
    let mut aL1 = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aL2 = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
    );
    assert!((aL1.angle(aL2) - std::f64::consts::FRAC_PI_2).abs() <= 1e-12);
}

#[test]
fn gp_LinTest__Translate() {
    let mut aLin = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aVec = Vector3::new(0.0, 5.0, 0.0);
    let mut aTranslated = aLin.translated(aVec);
    assert!((aTranslated.location().y() - 5.0).abs() <= 1e-7);
}

#[test]
fn gp_LinTest__Rotate() {
    let mut aLin = Line3::new(
        Point3::new(1.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    let mut aRotated = aLin.rotated(anAxis, std::f64::consts::FRAC_PI_2).unwrap();
    assert!((aRotated.location().x() - 0.0).abs() <= 1e-7);
    assert!((aRotated.location().y() - 1.0).abs() <= 1e-7);
    assert!(
        aRotated
            .direction()
            .is_equal(&Direction3::new(0.0, 1.0, 0.0).unwrap(), 1e-12)
    );
}

#[test]
fn gp_LinTest__Mirror() {
    let mut aLin = Line3::new(
        Point3::new(1.0, 1.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aMirrored = aLin.mirrored_axis(anAxis);
    assert!((aMirrored.location().y() - -1.0).abs() <= 1e-7);
}

#[test]
fn gp_LinTest__Transform() {
    let mut aLin = Line3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aTrsf = Transform3::default();
    aTrsf = Transform3::translation([0.0, 0.0, 10.0]);
    let mut aTransformed = aLin.transformed(aTrsf);
    assert!((aTransformed.location().z() - 10.0).abs() <= 1e-7);
}

// OCC15489: gp_Lin2d construction from implicit line equation A*x + B*y + C = 0.
// The constructor must throw Standard_ConstructionError when the direction
// vector is zero (sqrt(A*A + B*B) <= gp::Resolution()).

#[test]
fn gp_Lin2dTest__ConstructFromEquation_ZeroDirection_ThrowsException() {
    // A=0, B=0 -> direction vector is zero -> must throw

    assert_eq!(
        Line2::from_equation(0.0, 0.0, 1.0)
            .unwrap_err()
            .exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn gp_Lin2dTest__ConstructFromEquation_ValidCoefficients_CorrectOrigin() {
    // A=1e-20, B=-1, C=2 gives the line   1e-20 * x - y + 2 = 0
    // Origin (closest point to global origin on the line) must be
    // X_0 ~= -1.9999999999999999e-20, Y_0 ~= 2
    let mut aLin2d = Line2::from_equation(1e-20, -1.0, 2.0).unwrap();
    let mut anOrigin = aLin2d.location();
    assert!((anOrigin.x() - -1.9999999999999999e-20).abs() <= 1e-25);
    assert!((anOrigin.y() - 2.0).abs() <= 0.001);
}
