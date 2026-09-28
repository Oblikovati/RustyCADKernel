// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Direct translation of gce_MakeHypr_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gce::{ErrorType, MakeHypr, MakeHypr2d, Root};
use rustycad_tkmath::gp::{self, Axis2, Axis2d, Axis22d, Hyperbola2, Hyperbola3, Point2, Point3};

// ==================== gce_MakeHypr (3D) ====================

#[test]
fn gce_MakeHyprTest__FromAxis_ValidRadii_Done() {
    let aMaker = MakeHypr::new(Axis2::default(), 10.0, 5.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aHypr: Hyperbola3 = aMaker.value().unwrap();
    EXPECT_NEAR(aHypr.major_radius(), 10.0, precision::confusion());
    EXPECT_NEAR(aHypr.minor_radius(), 5.0, precision::confusion());
}

#[test]
fn gce_MakeHyprTest__FromAxis_MajorLessThanMinor_Done() {
    let aMaker = MakeHypr::new(Axis2::default(), 3.0, 7.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aHypr: Hyperbola3 = aMaker.value().unwrap();
    EXPECT_NEAR(aHypr.major_radius(), 3.0, precision::confusion());
    EXPECT_NEAR(aHypr.minor_radius(), 7.0, precision::confusion());
}

#[test]
fn gce_MakeHyprTest__FromAxis_EqualRadii_Done() {
    let aMaker = MakeHypr::new(Axis2::default(), 5.0, 5.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
}

#[test]
fn gce_MakeHyprTest__FromAxis_ZeroRadii_Done() {
    let aMaker = MakeHypr::new(Axis2::default(), 0.0, 0.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
}

#[test]
fn gce_MakeHyprTest__FromAxis_NegativeMajor_NegativeRadius() {
    let aMaker = MakeHypr::new(Axis2::default(), -1.0, 5.0).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::NegativeRadius);
}

#[test]
fn gce_MakeHyprTest__FromAxis_NegativeMinor_NegativeRadius() {
    let aMaker = MakeHypr::new(Axis2::default(), 5.0, -1.0).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::NegativeRadius);
}

#[test]
fn gce_MakeHyprTest__FromAxis_BothNegative_NegativeRadius() {
    let aMaker = MakeHypr::new(Axis2::default(), -3.0, -2.0).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::NegativeRadius);
}

#[test]
fn gce_MakeHyprTest__FromPoints_Valid_Done() {
    let aCenter = Point3::new(0.0, 0.0, 0.0);
    let aS1 = Point3::new(10.0, 0.0, 0.0);
    let aS2 = Point3::new(0.0, 5.0, 0.0);

    let aMaker = MakeHypr::from_points(aS1, aS2, aCenter).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aHypr: Hyperbola3 = aMaker.value().unwrap();
    EXPECT_NEAR(aHypr.major_radius(), 10.0, precision::confusion());
    EXPECT_NEAR(aHypr.minor_radius(), 5.0, precision::confusion());
}

#[test]
fn gce_MakeHyprTest__FromPoints_MajorLessThanMinor_Done() {
    let aCenter = Point3::new(0.0, 0.0, 0.0);
    let aS1 = Point3::new(3.0, 0.0, 0.0);
    let aS2 = Point3::new(0.0, 7.0, 0.0);

    let aMaker = MakeHypr::from_points(aS1, aS2, aCenter).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aHypr: Hyperbola3 = aMaker.value().unwrap();
    EXPECT_NEAR(aHypr.major_radius(), 3.0, precision::confusion());
    EXPECT_NEAR(aHypr.minor_radius(), 7.0, precision::confusion());
}

#[test]
fn gce_MakeHyprTest__FromPoints_ConfusedS1Center_ConfusedPoints() {
    let aCenter = Point3::new(1.0, 2.0, 3.0);
    let aS1 = Point3::new(1.0, 2.0, 3.0);
    let aS2 = Point3::new(4.0, 5.0, 6.0);

    let aMaker = MakeHypr::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}

#[test]
fn gce_MakeHyprTest__FromPoints_ConfusedS2Center_ConfusedPoints() {
    let aCenter = Point3::new(1.0, 2.0, 3.0);
    let aS1 = Point3::new(4.0, 5.0, 6.0);
    let aS2 = Point3::new(1.0, 2.0, 3.0);

    let aMaker = MakeHypr::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}

#[test]
fn gce_MakeHyprTest__FromPoints_ConfusedS1S2_ConfusedPoints() {
    let aCenter = Point3::new(0.0, 0.0, 0.0);
    let aS1 = Point3::new(5.0, 5.0, 5.0);
    let aS2 = Point3::new(5.0, 5.0, 5.0);

    let aMaker = MakeHypr::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}

#[test]
fn gce_MakeHyprTest__FromPoints_Collinear_ColinearPoints() {
    let aCenter = Point3::new(0.0, 0.0, 0.0);
    let aS1 = Point3::new(10.0, 0.0, 0.0);
    let aS2 = Point3::new(5.0, 0.0, 0.0);

    let aMaker = MakeHypr::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ColinearPoints);
}

#[test]
fn gce_MakeHyprTest__FromPoints_TinyScale_ConfusedPoints() {
    let aRes: f64 = gp::resolution();
    let aCenter = Point3::new(0.0, 0.0, 0.0);
    let aS1 = Point3::new(2.0 * aRes, 0.0, 0.0);
    let aS2 = Point3::new(0.0, 1.1 * aRes, 0.0);

    let aMaker = MakeHypr::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}

// ==================== gce_MakeHypr2d (2D) ====================

#[test]
fn gce_MakeHypr2dTest__FromMajorAxis_ValidRadii_Done() {
    let aMaker = MakeHypr2d::new(Axis2d::default(), 10.0, 5.0, true).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aHypr: Hyperbola2 = aMaker.value().unwrap();
    EXPECT_NEAR(aHypr.major_radius(), 10.0, precision::confusion());
    EXPECT_NEAR(aHypr.minor_radius(), 5.0, precision::confusion());
}

#[test]
fn gce_MakeHypr2dTest__FromMajorAxis_MajorLessThanMinor_Done() {
    let aMaker = MakeHypr2d::new(Axis2d::default(), 3.0, 7.0, true).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aHypr: Hyperbola2 = aMaker.value().unwrap();
    EXPECT_NEAR(aHypr.major_radius(), 3.0, precision::confusion());
    EXPECT_NEAR(aHypr.minor_radius(), 7.0, precision::confusion());
}

#[test]
fn gce_MakeHypr2dTest__FromMajorAxis_NegativeRadius_NegativeRadius() {
    let aMaker = MakeHypr2d::new(Axis2d::default(), -1.0, 5.0, true).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::NegativeRadius);
}

#[test]
fn gce_MakeHypr2dTest__FromAx22d_ValidRadii_Done() {
    let aMaker = MakeHypr2d::from_frame(Axis22d::default(), 8.0, 4.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
}

#[test]
fn gce_MakeHypr2dTest__FromAx22d_NegativeRadius_NegativeRadius() {
    let aMaker = MakeHypr2d::from_frame(Axis22d::default(), 5.0, -2.0).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::NegativeRadius);
}

#[test]
fn gce_MakeHypr2dTest__FromPoints_Valid_Done() {
    let aCenter = Point2::new(0.0, 0.0);
    let aS1 = Point2::new(10.0, 0.0);
    let aS2 = Point2::new(0.0, 5.0);

    let aMaker = MakeHypr2d::from_points(aS1, aS2, aCenter).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aHypr: Hyperbola2 = aMaker.value().unwrap();
    EXPECT_NEAR(aHypr.major_radius(), 10.0, precision::confusion());
    EXPECT_NEAR(aHypr.minor_radius(), 5.0, precision::confusion());
}

#[test]
fn gce_MakeHypr2dTest__FromPoints_MajorLessThanMinor_Done() {
    let aCenter = Point2::new(0.0, 0.0);
    let aS1 = Point2::new(3.0, 0.0);
    let aS2 = Point2::new(0.0, 7.0);

    let aMaker = MakeHypr2d::from_points(aS1, aS2, aCenter).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aHypr: Hyperbola2 = aMaker.value().unwrap();
    EXPECT_NEAR(aHypr.major_radius(), 3.0, precision::confusion());
    EXPECT_NEAR(aHypr.minor_radius(), 7.0, precision::confusion());
}

#[test]
fn gce_MakeHypr2dTest__FromPoints_ConfusedS1Center_ConfusedPoints() {
    let aCenter = Point2::new(1.0, 2.0);
    let aS1 = Point2::new(1.0, 2.0);
    let aS2 = Point2::new(4.0, 5.0);

    let aMaker = MakeHypr2d::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}

#[test]
fn gce_MakeHypr2dTest__FromPoints_ConfusedS2Center_ConfusedPoints() {
    let aCenter = Point2::new(1.0, 2.0);
    let aS1 = Point2::new(4.0, 5.0);
    let aS2 = Point2::new(1.0, 2.0);

    let aMaker = MakeHypr2d::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}

#[test]
fn gce_MakeHypr2dTest__FromPoints_Collinear_ColinearPoints() {
    let aCenter = Point2::new(0.0, 0.0);
    let aS1 = Point2::new(10.0, 0.0);
    let aS2 = Point2::new(5.0, 0.0);

    let aMaker = MakeHypr2d::from_points(aS1, aS2, aCenter).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ColinearPoints);
}
