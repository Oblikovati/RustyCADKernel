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

//! Complete gp_Pln legacy suite, preserving every assertion.
#![allow(non_snake_case, unused_assignments)]
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Plane3, Point3, Transform3, Vector3};
#[test]
fn gp_PlnTest__ConstructorFromAx3() {
    let anAx3 = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aPln = Plane3::from_position(anAx3);
    assert!(aPln.location().is_equal(&Point3::new(0.0, 0.0, 0.0), 1e-7));
    assert!(
        aPln.axis()
            .direction()
            .is_equal(&Direction3::new(0.0, 0.0, 1.0).unwrap(), 1e-12)
    );
}

#[test]
fn gp_PlnTest__ConstructorFromPointAndDir() {
    let aLoc = Point3::new(1.0, 2.0, 3.0);
    let aNorm = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let aPln = Plane3::new(aLoc, aNorm).unwrap();
    assert!(aPln.location().is_equal(&aLoc, 1e-7));
    assert!(aPln.axis().direction().is_equal(&aNorm, 1e-12));
}

#[test]
fn gp_PlnTest__ConstructorFromCoefficients() {
    // Plane Z = 5 => 0*X + 0*Y + 1*Z - 5 = 0
    let aPln = Plane3::from_equation(0.0, 0.0, 1.0, -5.0).unwrap();

    let [anA, aB, aC, aD] = aPln.coefficients().unwrap();
    assert!((anA - 0.0).abs() <= 1e-7);
    assert!((aB - 0.0).abs() <= 1e-7);
    assert!((aC - 1.0).abs() <= 1e-7);
    assert!((aD - -5.0).abs() <= 1e-7);
}

#[test]
fn gp_PlnTest__DistanceToPoint() {
    let aPln = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aPnt = Point3::new(3.0, 4.0, 5.0);
    assert!((aPln.distance(aPnt) - 5.0).abs() <= 1e-7);
}

#[test]
fn gp_PlnTest__Contains() {
    let aPln = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aOnPlane = Point3::new(3.0, 4.0, 0.0);
    let aOffPlane = Point3::new(3.0, 4.0, 1.0);
    assert!(aPln.contains(aOnPlane, 1e-7));
    assert!(!(aPln.contains(aOffPlane, 1e-7)));
}

#[test]
fn gp_PlnTest__Coefficients() {
    let aPln = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();

    let [anA, aB, aC, aD] = aPln.coefficients().unwrap();
    assert!((anA - 0.0).abs() <= 1e-7);
    assert!((aB - 0.0).abs() <= 1e-7);
    assert!((aC - 1.0).abs() <= 1e-7);
    assert!((aD - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_PlnTest__Mirror() {
    let aPln = Plane3::new(
        Point3::new(0.0, 0.0, 5.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aMirrorPlane = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aMirrored = aPln.mirrored_plane(aMirrorPlane);
    assert!((aMirrored.location().z() - -5.0).abs() <= 1e-7);
}

#[test]
fn gp_PlnTest__Rotate() {
    let aPln = Plane3::new(
        Point3::new(1.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    let aRotated = aPln.rotated(anAxis, std::f64::consts::FRAC_PI_2).unwrap();
    assert!((aRotated.location().x() - 0.0).abs() <= 1e-7);
    assert!((aRotated.location().y() - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_PlnTest__Scale() {
    let aPln = Plane3::new(
        Point3::new(1.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aCenter = Point3::new(0.0, 0.0, 0.0);
    let aScaled = aPln.scaled(aCenter, 3.0);
    assert!((aScaled.location().x() - 3.0).abs() <= 1e-7);
}

#[test]
fn gp_PlnTest__Translate() {
    let aPln = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aVec = Vector3::new(1.0, 2.0, 3.0);
    let aTranslated = aPln.translated(aVec);
    assert!((aTranslated.location().x() - 1.0).abs() <= 1e-7);
    assert!((aTranslated.location().y() - 2.0).abs() <= 1e-7);
    assert!((aTranslated.location().z() - 3.0).abs() <= 1e-7);
}

#[test]
fn gp_PlnTest__Transform() {
    let aPln = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let mut aTrsf = Transform3::default();
    aTrsf = Transform3::translation([0.0, 0.0, 10.0]);
    let aTransformed = aPln.transformed(aTrsf);
    assert!((aTransformed.location().z() - 10.0).abs() <= 1e-7);
}
