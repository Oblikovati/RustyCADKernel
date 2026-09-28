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

//! Complete legacy planar primitive suite.
#![allow(non_snake_case, unused_mut, unused_assignments, unused_imports)]
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2, Vector2, Xy};
#[test]
fn gp_Pnt2dTest__DefaultConstructor() {
    let mut aPnt = Point2::default();
    assert_eq!(aPnt.x(), 0.0);
    assert_eq!(aPnt.y(), 0.0);
}

#[test]
fn gp_Pnt2dTest__CoordinateConstructor() {
    let mut aPnt = Point2::new(1.0, 2.0);
    assert_eq!(aPnt.x(), 1.0);
    assert_eq!(aPnt.y(), 2.0);
}

#[test]
fn gp_Pnt2dTest__XYConstructor() {
    let mut aXY = Xy::new(3.0, 4.0);
    let mut aPnt = Point2::from(aXY);
    assert_eq!(aPnt.x(), 3.0);
    assert_eq!(aPnt.y(), 4.0);
}

#[test]
fn gp_Pnt2dTest__SetCoord() {
    let mut aPnt = Point2::default();
    aPnt.set_x(10.0);
    aPnt.set_y(20.0);
    assert_eq!(aPnt.x(), 10.0);
    assert_eq!(aPnt.y(), 20.0);

    aPnt.set_coordinates(7.0, 8.0);
    assert_eq!(aPnt.x(), 7.0);
    assert_eq!(aPnt.y(), 8.0);
}

#[test]
fn gp_Pnt2dTest__Distance() {
    let mut aP1 = Point2::new(0.0, 0.0);
    let mut aP2 = Point2::new(3.0, 4.0);
    assert!((aP1.distance(aP2) - 5.0).abs() <= 1e-7);
}

#[test]
fn gp_Pnt2dTest__SquareDistance() {
    let mut aP1 = Point2::new(1.0, 2.0);
    let mut aP2 = Point2::new(4.0, 6.0);
    // (4-1)^2 + (6-2)^2 = 9 + 16 = 25
    assert!((aP1.square_distance(aP2) - 25.0).abs() <= 1e-7);
}

#[test]
fn gp_Pnt2dTest__IsEqual() {
    let mut aP1 = Point2::new(1.0, 2.0);
    let mut aP2 = Point2::new(1.0, 2.0);
    let mut aP3 = Point2::new(1.0, 3.0);
    assert!(aP1.is_equal(aP2, 1e-7));
    assert!(!(aP1.is_equal(aP3, 1e-7)));
}

#[test]
fn gp_Pnt2dTest__Translate_ByVec2d() {
    let mut aPnt = Point2::new(1.0, 2.0);
    let mut aVec = Vector2::new(10.0, 20.0);
    let mut aResult = aPnt.translated(aVec);
    assert!((aResult.x() - 11.0).abs() <= 1e-7);
    assert!((aResult.y() - 22.0).abs() <= 1e-7);
}

#[test]
fn gp_Pnt2dTest__Scale() {
    let mut aPnt = Point2::new(2.0, 4.0);
    let mut aCenter = Point2::new(0.0, 0.0);
    let mut aResult = aPnt.scaled(aCenter, 2.0);
    assert!((aResult.x() - 4.0).abs() <= 1e-7);
    assert!((aResult.y() - 8.0).abs() <= 1e-7);
}

#[test]
fn gp_Pnt2dTest__Rotate() {
    let mut aPnt = Point2::new(1.0, 0.0);
    let mut aCenter = Point2::new(0.0, 0.0);
    let mut aResult = aPnt.rotated(aCenter, std::f64::consts::FRAC_PI_2);
    assert!((aResult.x() - 0.0).abs() <= 1e-7);
    assert!((aResult.y() - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_Pnt2dTest__Mirror_Point() {
    let mut aPnt = Point2::new(1.0, 0.0);
    let mut aCenter = Point2::new(0.0, 0.0);
    let mut aResult = aPnt.mirrored_point(aCenter);
    assert!((aResult.x() - -1.0).abs() <= 1e-7);
    assert!((aResult.y() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_Pnt2dTest__Mirror_Axis() {
    let mut aPnt = Point2::new(1.0, 1.0);
    // Mirror through the X axis
    let mut anAxis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    let mut aResult = aPnt.mirrored_axis(anAxis);
    assert!((aResult.x() - 1.0).abs() <= 1e-7);
    assert!((aResult.y() - -1.0).abs() <= 1e-7);
}

#[test]
fn gp_Pnt2dTest__Transformed() {
    let mut aPnt = Point2::new(1.0, 0.0);
    let mut aTrsf = Transform2::default();
    aTrsf = Transform2::translation([0.0, 5.0]);
    let mut aResult = aPnt.transformed(aTrsf);
    assert!((aResult.x() - 1.0).abs() <= 1e-7);
    assert!((aResult.y() - 5.0).abs() <= 1e-7);
}

// OCC22736: Verifies that composing two identical mirror transformations through the same axis
// is equivalent to the identity transform. Mirror p1 once through the axis to get p2, then
// mirror p2 to get back p1. Composing M1*M2 and applying to p1 must also return p1.
#[test]
fn gp_Pnt2dTest__OCC22736_TwoIdenticalMirrorCompositionIsIdentity() {
    let mut aMirrorFirst = Point2::new(2.0, 1.0);
    let mut aMirrorSecond = Point2::new(3.0, 1.0);
    let mut aMirrorAxis = Axis2d::new(
        aMirrorFirst,
        Direction2::from_vector(Vector2::between(aMirrorFirst, aMirrorSecond)).unwrap(),
    );

    let mut aP1 = Point2::new(1.0, 0.0);
    let mut aP2 = Point2::new(1.0, 2.0);

    let mut aM1 = Transform2::default();
    aM1 = Transform2::mirror_axis(aMirrorAxis);
    let mut aM2 = Transform2::default();
    aM2 = Transform2::mirror_axis(aMirrorAxis);

    // Applying M1 to p1 should give p2
    let mut aP1MirrorM1 = aP1.transformed(aM1);
    assert!((aP1MirrorM1.x() - aP2.x()).abs() <= 1e-7);
    assert!((aP1MirrorM1.y() - aP2.y()).abs() <= 1e-7);

    // Applying M2 to the mirrored point should give back p1
    let mut aP1MirrorM1M2 = aP1MirrorM1.transformed(aM2);
    assert!((aP1MirrorM1M2.x() - aP1.x()).abs() <= 1e-7);
    assert!((aP1MirrorM1M2.y() - aP1.y()).abs() <= 1e-7);

    // Composing M2*M1 must also be identity on p1
    let mut aComp = aM2.multiplied(aM1);
    let mut aP1Comp = aP1.transformed(aComp);
    assert!((aP1Comp.x() - aP1.x()).abs() <= 1e-7);
    assert!((aP1Comp.y() - aP1.y()).abs() <= 1e-7);
}
