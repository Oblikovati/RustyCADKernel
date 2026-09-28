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
fn gp_Vec2dTest__DefaultConstructor() {
    let mut aVec = Vector2::default();
    assert_eq!(aVec.x(), 0.0);
    assert_eq!(aVec.y(), 0.0);
}

#[test]
fn gp_Vec2dTest__CoordinateConstructor() {
    let mut aVec = Vector2::new(1.0, 2.0);
    assert_eq!(aVec.x(), 1.0);
    assert_eq!(aVec.y(), 2.0);
}

#[test]
fn gp_Vec2dTest__TwoPointConstructor() {
    let mut aP1 = Point2::new(1.0, 2.0);
    let mut aP2 = Point2::new(4.0, 6.0);
    let mut aVec = Vector2::between(aP1, aP2);
    assert_eq!(aVec.x(), 3.0);
    assert_eq!(aVec.y(), 4.0);
}

#[test]
fn gp_Vec2dTest__Dir2dConstructor() {
    let mut aDir = Direction2::new(0.0, 1.0).unwrap();
    let mut aVec = Vector2::from(aDir);
    assert!((aVec.magnitude() - 1.0).abs() <= 1e-7);
    assert!((aVec.y() - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_Vec2dTest__AddSubtract() {
    let mut aV1 = Vector2::new(1.0, 2.0);
    let mut aV2 = Vector2::new(4.0, 5.0);

    let mut aSum = aV1 + aV2;
    assert_eq!(aSum.x(), 5.0);
    assert_eq!(aSum.y(), 7.0);

    let mut aDiff = aV2 - aV1;
    assert_eq!(aDiff.x(), 3.0);
    assert_eq!(aDiff.y(), 3.0);
}

#[test]
fn gp_Vec2dTest__MultiplyDivide() {
    let mut aVec = Vector2::new(2.0, 4.0);

    let mut aScaled = aVec * 3.0;
    assert_eq!(aScaled.x(), 6.0);
    assert_eq!(aScaled.y(), 12.0);

    let mut aDivided = aVec / 2.0;
    assert_eq!(aDivided.x(), 1.0);
    assert_eq!(aDivided.y(), 2.0);
}

#[test]
fn gp_Vec2dTest__Dot() {
    let mut aV1 = Vector2::new(1.0, 0.0);
    let mut aV2 = Vector2::new(0.0, 1.0);
    assert_eq!(aV1.dot(aV2), 0.0);

    let mut aV3 = Vector2::new(1.0, 2.0);
    let mut aV4 = Vector2::new(4.0, 5.0);
    // 1*4 + 2*5 = 14
    assert_eq!(aV3.dot(aV4), 14.0);
}

#[test]
fn gp_Vec2dTest__Crossed() {
    let mut aV1 = Vector2::new(1.0, 0.0);
    let mut aV2 = Vector2::new(0.0, 1.0);
    // V1.X*V2.Y - V1.Y*V2.X = 1*1 - 0*0 = 1
    assert!((aV1.crossed(aV2) - 1.0).abs() <= 1e-7);

    let mut aV3 = Vector2::new(3.0, 4.0);
    let mut aV4 = Vector2::new(2.0, 5.0);
    // 3*5 - 4*2 = 15 - 8 = 7
    assert!((aV3.crossed(aV4) - 7.0).abs() <= 1e-7);
}

#[test]
fn gp_Vec2dTest__Magnitude() {
    let mut aVec = Vector2::new(3.0, 4.0);
    assert!((aVec.magnitude() - 5.0).abs() <= 1e-7);
    assert!((aVec.square_magnitude() - 25.0).abs() <= 1e-7);
}

#[test]
fn gp_Vec2dTest__Normalize() {
    let mut aVec = Vector2::new(3.0, 4.0);
    let mut aNormalized = aVec.normalized().unwrap();
    assert!((aNormalized.magnitude() - 1.0).abs() <= 1e-7);
    assert!((aNormalized.x() - 0.6).abs() <= 1e-7);
    assert!((aNormalized.y() - 0.8).abs() <= 1e-7);
}

#[test]
fn gp_Vec2dTest__Angle() {
    let mut aV1 = Vector2::new(1.0, 0.0);
    let mut aV2 = Vector2::new(0.0, 1.0);
    assert!((aV1.angle(aV2).unwrap() - std::f64::consts::FRAC_PI_2).abs() <= 1e-12);
}

#[test]
fn gp_Vec2dTest__IsParallel() {
    let mut aV1 = Vector2::new(1.0, 0.0);
    let mut aV2 = Vector2::new(5.0, 0.0);
    assert!(aV1.is_parallel(aV2, 1e-12).unwrap());
}

#[test]
fn gp_Vec2dTest__IsNormal() {
    let mut aV1 = Vector2::new(1.0, 0.0);
    let mut aV2 = Vector2::new(0.0, 1.0);
    assert!(aV1.is_normal(aV2, 1e-12).unwrap());
}

#[test]
fn gp_Vec2dTest__IsOpposite() {
    let mut aV1 = Vector2::new(1.0, 0.0);
    let mut aV2 = Vector2::new(-3.0, 0.0);
    assert!(aV1.is_opposite(aV2, 1e-12).unwrap());
}

#[test]
fn gp_Vec2dTest__Reverse() {
    let mut aVec = Vector2::new(1.0, 2.0);
    let mut aReversed = aVec.reversed();
    assert_eq!(aReversed.x(), -1.0);
    assert_eq!(aReversed.y(), -2.0);
}

#[test]
fn gp_Vec2dTest__SetLinearForm() {
    let mut aV1 = Vector2::new(1.0, 0.0);
    let mut aV2 = Vector2::new(0.0, 1.0);
    let mut aResult = Vector2::default();
    aResult.set_linear_form(2.0, aV1, 3.0, aV2);
    assert!((aResult.x() - 2.0).abs() <= 1e-7);
    assert!((aResult.y() - 3.0).abs() <= 1e-7);
}

#[test]
fn gp_Vec2dTest__OCC26750_IsNormal_NegativeAngle() {
    // OCC26750: gp_Vec2d::IsNormal() returned FALSE when the angle is -PI/2 (not only +PI/2).
    let mut aVec1 = Vector2::new(1.0, 0.0);
    let mut aVec2 = Vector2::new(0.0, -1.0); // -90 degrees
    assert!(aVec1.is_normal(aVec2, 1e-12).unwrap());

    // Verify gp_Dir2d as well
    let mut aD1 = Direction2::X;
    let mut aD2 = Direction2::NEG_Y;
    assert!(aD1.is_normal(aD2, 1e-12));
}
