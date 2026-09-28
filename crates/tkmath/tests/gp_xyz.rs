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

//! All gp_XYZ legacy declarations; original scalar expectations retained.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::gp::Xyz;
#[test]
fn gp_XYZTest__DefaultConstructor() {
    let mut aXYZ = Xyz::default();
    assert_eq!(aXYZ.x(), 0.0);
    assert_eq!(aXYZ.y(), 0.0);
    assert_eq!(aXYZ.z(), 0.0);
}

#[test]
fn gp_XYZTest__CoordinateConstructor() {
    let mut aXYZ = Xyz::new(1.0, 2.0, 3.0);
    assert_eq!(aXYZ.x(), 1.0);
    assert_eq!(aXYZ.y(), 2.0);
    assert_eq!(aXYZ.z(), 3.0);
}

#[test]
fn gp_XYZTest__SetCoord() {
    let mut aXYZ = Xyz::default();
    aXYZ.set_coordinates(7.0, 8.0, 9.0);
    assert_eq!(aXYZ.x(), 7.0);
    assert_eq!(aXYZ.y(), 8.0);
    assert_eq!(aXYZ.z(), 9.0);
}

#[test]
fn gp_XYZTest__Accessors() {
    let mut aXYZ = Xyz::new(1.0, 2.0, 3.0);
    assert_eq!(aXYZ.coord(1).unwrap(), 1.0);
    assert_eq!(aXYZ.coord(2).unwrap(), 2.0);
    assert_eq!(aXYZ.coord(3).unwrap(), 3.0);
}

#[test]
fn gp_XYZTest__Add() {
    let mut aA = Xyz::new(1.0, 2.0, 3.0);
    let mut aB = Xyz::new(4.0, 5.0, 6.0);
    let mut aSum = aA + aB;
    assert_eq!(aSum.x(), 5.0);
    assert_eq!(aSum.y(), 7.0);
    assert_eq!(aSum.z(), 9.0);
}

#[test]
fn gp_XYZTest__Subtract() {
    let mut aA = Xyz::new(4.0, 5.0, 6.0);
    let mut aB = Xyz::new(1.0, 2.0, 3.0);
    let mut aDiff = aA - aB;
    assert_eq!(aDiff.x(), 3.0);
    assert_eq!(aDiff.y(), 3.0);
    assert_eq!(aDiff.z(), 3.0);
}

#[test]
fn gp_XYZTest__Multiply() {
    let mut aXYZ = Xyz::new(1.0, 2.0, 3.0);
    let mut aScaled = aXYZ * 2.0;
    assert_eq!(aScaled.x(), 2.0);
    assert_eq!(aScaled.y(), 4.0);
    assert_eq!(aScaled.z(), 6.0);
}

#[test]
fn gp_XYZTest__Divide() {
    let mut aXYZ = Xyz::new(4.0, 6.0, 8.0);
    let mut aDivided = aXYZ / 2.0;
    assert_eq!(aDivided.x(), 2.0);
    assert_eq!(aDivided.y(), 3.0);
    assert_eq!(aDivided.z(), 4.0);
}

#[test]
fn gp_XYZTest__Cross() {
    let mut aA = Xyz::new(1.0, 0.0, 0.0);
    let mut aB = Xyz::new(0.0, 1.0, 0.0);
    let mut aCross = aA.crossed(aB);
    assert!((aCross.x() - 0.0).abs() <= 1e-7);
    assert!((aCross.y() - 0.0).abs() <= 1e-7);
    assert!((aCross.z() - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_XYZTest__Dot() {
    let mut aA = Xyz::new(1.0, 2.0, 3.0);
    let mut aB = Xyz::new(4.0, 5.0, 6.0);
    // 1*4 + 2*5 + 3*6 = 32
    assert_eq!(aA.dot(aB), 32.0);
}

#[test]
fn gp_XYZTest__CrossMagnitude() {
    let mut aA = Xyz::new(1.0, 0.0, 0.0);
    let mut aB = Xyz::new(0.0, 1.0, 0.0);
    assert!((aA.cross_magnitude(aB) - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_XYZTest__CrossSquareMagnitude() {
    let mut aA = Xyz::new(1.0, 0.0, 0.0);
    let mut aB = Xyz::new(0.0, 1.0, 0.0);
    assert!((aA.cross_square_magnitude(aB) - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_XYZTest__DotCross() {
    let mut aA = Xyz::new(1.0, 0.0, 0.0);
    let mut aB = Xyz::new(0.0, 1.0, 0.0);
    let mut aC = Xyz::new(0.0, 0.0, 1.0);
    assert!((aA.dot_cross(aB, aC) - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_XYZTest__Modulus() {
    let mut aXYZ = Xyz::new(3.0, 4.0, 0.0);
    assert!((aXYZ.modulus() - 5.0).abs() <= 1e-7);
}

#[test]
fn gp_XYZTest__SquareModulus() {
    let mut aXYZ = Xyz::new(3.0, 4.0, 0.0);
    assert!((aXYZ.square_modulus() - 25.0).abs() <= 1e-7);
}

#[test]
fn gp_XYZTest__SetLinearForm() {
    let mut aA = Xyz::new(1.0, 0.0, 0.0);
    let mut aB = Xyz::new(0.0, 1.0, 0.0);
    let mut aResult = Xyz::default();
    aResult.set_linear_form(2.0, aA, 3.0, aB);
    assert!((aResult.x() - 2.0).abs() <= 1e-7);
    assert!((aResult.y() - 3.0).abs() <= 1e-7);
    assert!((aResult.z() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_XYZTest__IsEqual() {
    let mut aA = Xyz::new(1.0, 2.0, 3.0);
    let mut aB = Xyz::new(1.0, 2.0, 3.0);
    let mut aC = Xyz::new(1.0, 2.0, 4.0);
    assert!(aA.is_equal(&aB, 1e-7));
    assert!(!(aA.is_equal(&aC, 1e-7)));
}
