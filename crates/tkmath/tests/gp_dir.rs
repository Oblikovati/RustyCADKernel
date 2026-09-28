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

//! All gp_Dir legacy cases, including native mirror behavior.
#![allow(non_snake_case, unused_mut, unused_assignments)]
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Transform3, Vector3, Xyz};
#[test]
fn gp_DirTest__CoordinateConstructor() {
    let mut aDir = Direction3::new(0.0, 0.0, 1.0).unwrap();
    assert!((aDir.x() - 0.0).abs() <= 1e-7);
    assert!((aDir.y() - 0.0).abs() <= 1e-7);
    assert!((aDir.z() - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_DirTest__XYZConstructor() {
    let mut aXYZ = Xyz::new(0.0, 3.0, 4.0);
    let mut aDir = Direction3::from_xyz(aXYZ).unwrap();
    assert!((aDir.x() - 0.0).abs() <= 1e-7);
    assert!((aDir.y() - 0.6).abs() <= 1e-7);
    assert!((aDir.z() - 0.8).abs() <= 1e-7);
}

#[test]
fn gp_DirTest__VecConstructor() {
    let mut aVec = Vector3::new(3.0, 0.0, 0.0);
    let mut aDir = Direction3::from_vector(aVec).unwrap();
    assert!((aDir.x() - 1.0).abs() <= 1e-7);
    assert!((aDir.y() - 0.0).abs() <= 1e-7);
    assert!((aDir.z() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_DirTest__PredefinedDirections() {
    assert!(
        Direction3::new(1.0, 0.0, 0.0)
            .unwrap()
            .is_equal(&Direction3::X, 1e-12)
    );
    assert!(
        Direction3::new(0.0, 1.0, 0.0)
            .unwrap()
            .is_equal(&Direction3::Y, 1e-12)
    );
    assert!(
        Direction3::new(0.0, 0.0, 1.0)
            .unwrap()
            .is_equal(&Direction3::Z, 1e-12)
    );
}

#[test]
fn gp_DirTest__Angle() {
    let mut aD1 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aD2 = Direction3::new(0.0, 1.0, 0.0).unwrap();
    assert!((aD1.angle(&aD2) - std::f64::consts::FRAC_PI_2).abs() <= 1e-12);
}

#[test]
fn gp_DirTest__AngleWithRef() {
    let mut aD1 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aD2 = Direction3::new(0.0, 1.0, 0.0).unwrap();
    let mut aRef = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let anAngle = aD1.angle_with_ref(aD2, aRef);
    assert!((anAngle - std::f64::consts::FRAC_PI_2).abs() <= 1e-12);

    // Reverse reference direction should negate the angle
    let anAngleReversed = aD1.angle_with_ref(aD2, Direction3::new(0.0, 0.0, -1.0).unwrap());
    assert!((anAngleReversed - -std::f64::consts::FRAC_PI_2).abs() <= 1e-12);
}

#[test]
fn gp_DirTest__IsEqual() {
    let mut aD1 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aD2 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    assert!(aD1.is_equal(&aD2, 1e-12));
}

#[test]
fn gp_DirTest__IsNormal() {
    let mut aD1 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aD2 = Direction3::new(0.0, 1.0, 0.0).unwrap();
    assert!(aD1.is_normal(aD2, 1e-12));
}

#[test]
fn gp_DirTest__IsOpposite() {
    let mut aD1 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aD2 = Direction3::new(-1.0, 0.0, 0.0).unwrap();
    assert!(aD1.is_opposite(aD2, 1e-12));
}

#[test]
fn gp_DirTest__IsParallel() {
    let mut aD1 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aD2 = Direction3::new(-1.0, 0.0, 0.0).unwrap();
    assert!(aD1.is_parallel(aD2, 1e-12));
}

#[test]
fn gp_DirTest__Cross() {
    let mut aD1 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aD2 = Direction3::new(0.0, 1.0, 0.0).unwrap();
    let mut aCross = aD1.crossed(aD2).unwrap();
    assert!((aCross.x() - 0.0).abs() <= 1e-7);
    assert!((aCross.y() - 0.0).abs() <= 1e-7);
    assert!((aCross.z() - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_DirTest__CrossCross() {
    let mut aA = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aB = Direction3::new(1.0, 1.0, 0.0).unwrap();
    let mut aC = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let mut aCrossCross = aA.cross_crossed(aB, aC).unwrap();
    // Result should be a valid normalized direction
    let aMag = (aCrossCross.x() * aCrossCross.x()
        + aCrossCross.y() * aCrossCross.y()
        + aCrossCross.z() * aCrossCross.z())
    .sqrt();
    assert!((aMag - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_DirTest__Dot() {
    let mut aD1 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aD2 = Direction3::new(0.0, 1.0, 0.0).unwrap();
    assert!((aD1.dot(aD2) - 0.0).abs() <= 1e-7);

    let mut aD3 = Direction3::new(1.0, 0.0, 0.0).unwrap();
    assert!((aD1.dot(aD3) - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_DirTest__Mirror_Point() {
    let mut aDir = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
    );
    let mut aMirrored = aDir.mirrored_axis(anAxis);
    // Mirror X direction through Y axis: result should be (-1,0,0)...
    // Actually mirroring direction (1,0,0) through axis Y means reflecting about the Y axis line
    // The component along Y stays, the perpendicular component reverses: result is (-1,0,0)
    // But for axis mirror of a direction, it reflects in the axis:
    // d_mirror = 2*(d.axis)*axis - d
    // d.axis = (1,0,0).(0,1,0) = 0
    // result = 2*0*(0,1,0) - (1,0,0) = (-1,0,0)
    assert!((aMirrored.x() - -1.0).abs() <= 1e-7);
    assert!((aMirrored.y() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_DirTest__Rotate() {
    let mut aDir = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    let mut aRotated = aDir.rotated(anAxis, std::f64::consts::FRAC_PI_2).unwrap();
    assert!((aRotated.x() - 0.0).abs() <= 1e-7);
    assert!((aRotated.y() - 1.0).abs() <= 1e-7);
    assert!((aRotated.z() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_DirTest__Transform() {
    let mut aDir = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut aTrsf = Transform3::default();
    aTrsf = Transform3::rotation(
        Axis1::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        ),
        std::f64::consts::PI,
    )
    .unwrap();
    let mut aTransformed = aDir.transformed(aTrsf);
    assert!((aTransformed.x() - -1.0).abs() <= 1e-7);
    assert!((aTransformed.y() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_DirTest__ZeroMagnitudeInput_ThrowsException() {
    assert_eq!(
        Direction3::new(0.0, 0.0, 0.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}

// OCC22558: Mirror of gp_Dir(1,1,1) through gp_Ax2 at origin with main dir (1,0,0) gives
// (-1/sqrt(3), 1/sqrt(3), 1/sqrt(3)).  The TCL test checks int(res * 100000) == {-57735, 57735,
// 57735}.
#[test]
fn gp_DirTest__OCC22558_MirrorThroughAx2_CorrectResult() {
    let mut aToSym = Direction3::new(1.0, 1.0, 1.0).unwrap();
    let mut anAx2 = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    aToSym.mirror_plane(anAx2);

    // Expected: (-1/sqrt(3), 1/sqrt(3), 1/sqrt(3))
    let aExpected = 1.0 / 3.0_f64.sqrt();
    assert!((aToSym.x() - -aExpected).abs() <= 1e-7);
    assert!((aToSym.y() - aExpected).abs() <= 1e-7);
    assert!((aToSym.z() - aExpected).abs() <= 1e-7);
}
