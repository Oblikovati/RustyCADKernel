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

//! Direct translation of gce_MakeCone_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gce::{ErrorType, MakeCone, Root};
use rustycad_tkmath::gp::{Axis2, Axis3, Cone3, Direction3, Line3, Point3};
use std::f64::consts::PI;

fn base_cone() -> Cone3 {
    let frame = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Cone3::new(Axis3::from_axis2(frame), PI / 6.0, 10.0).unwrap()
}

fn expect_orthogonal_x_axis(aCone: &Cone3, aP1: Point3, aP2: Point3) {
    let aAxis = Direction3::from_xyz(aP2.xyz() - aP1.xyz()).unwrap();
    assert!(
        aCone
            .axis()
            .direction()
            .is_parallel(aAxis, precision::angular())
    );
    let aDot: f64 = aCone
        .axis()
        .direction()
        .xyz()
        .dot(aCone.position().x_direction().xyz());
    EXPECT_NEAR(aDot, 0.0, precision::angular());
}

#[test]
fn gce_MakeConeTest__FromConeAndDist_Valid_Done() {
    let aBaseCone = base_cone();
    let aDist: f64 = 2.0;

    let aMaker = MakeCone::from_cone_distance(aBaseCone, aDist).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aCone: Cone3 = aMaker.value().unwrap();
    let anExpectedRadius: f64 = aBaseCone.ref_radius() + aDist / aBaseCone.semi_angle().cos();
    EXPECT_NEAR(aCone.ref_radius(), anExpectedRadius, precision::confusion());
    EXPECT_NEAR(
        aCone.semi_angle(),
        aBaseCone.semi_angle(),
        precision::confusion(),
    );
    assert!(
        aCone
            .location()
            .is_equal(&aBaseCone.location(), precision::confusion())
    );
    assert!(
        aCone
            .axis()
            .direction()
            .is_parallel(aBaseCone.axis().direction(), precision::angular())
    );
}

#[test]
fn gce_MakeConeTest__FromConeAndDist_TooNegative_NegativeRadius() {
    let aBaseCone = base_cone();
    let aDist: f64 = -(aBaseCone.ref_radius() * aBaseCone.semi_angle().cos() + 1.0);

    let aMaker = MakeCone::from_cone_distance(aBaseCone, aDist).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::NegativeRadius);
}

#[test]
fn gce_MakeConeTest__FromConeAndPoint_Valid_Done() {
    let aBaseCone = base_cone();
    let aVParam: f64 = 4.0;
    let aTargetRadius: f64 = 12.0;
    let aPointRadius: f64 = aTargetRadius + aVParam * aBaseCone.semi_angle().tan();
    let aPoint = Point3::new(aPointRadius, 0.0, aVParam);

    let aMaker = MakeCone::from_cone_point(aBaseCone, aPoint).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aCone: Cone3 = aMaker.value().unwrap();
    EXPECT_NEAR(aCone.ref_radius(), aTargetRadius, precision::confusion());
}

#[test]
fn gce_MakeConeTest__FromConeAndPoint_NoValidParallelCone_NegativeRadius() {
    let aBaseCone = base_cone();
    let aPoint = Point3::new(1.0, 0.0, 20.0);

    let aMaker = MakeCone::from_cone_point(aBaseCone, aPoint).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::NegativeRadius);
}

#[test]
fn gce_MakeConeTest__FromConeAndPoint_TwoCandidates_ChoosesNearestToBaseCone() {
    let aBaseCone = base_cone();
    let aPoint = Point3::new(5.0, 0.0, -20.0);

    let aMaker = MakeCone::from_cone_point(aBaseCone, aPoint).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    let aRadius: f64 = Line3::from_axis(aBaseCone.axis()).distance(aPoint);
    let aVParam: f64 =
        (aPoint.xyz() - aBaseCone.location().xyz()).dot(aBaseCone.axis().direction().xyz());
    let aTan: f64 = aBaseCone.semi_angle().tan();
    let aCos: f64 = aBaseCone.semi_angle().cos();
    let aCand1: f64 = aRadius - aVParam * aTan;
    let aCand2: f64 = -aRadius - aVParam * aTan;
    let aDist1: f64 = ((aCand1 - aBaseCone.ref_radius()) * aCos).abs();
    let aDist2: f64 = ((aCand2 - aBaseCone.ref_radius()) * aCos).abs();
    let anExpR: f64 = if aDist1 <= aDist2 { aCand1 } else { aCand2 };

    EXPECT_NEAR(
        aMaker.value().unwrap().ref_radius(),
        anExpR,
        precision::confusion(),
    );
}

#[test]
fn gce_MakeConeTest__FromTwoPointsAndTwoRadii_YBranch_DoneAndOrthogonalXAxis() {
    let aP1 = Point3::new(0.0, 0.0, 0.0);
    let aP2 = Point3::new(0.0, 10.0, 10.0);

    let aMaker = MakeCone::from_points_radii(aP1, aP2, 3.0, 2.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    expect_orthogonal_x_axis(&aMaker.value().unwrap(), aP1, aP2);
}

#[test]
fn gce_MakeConeTest__FromTwoPointsAndTwoRadii_ZBranch_DoneAndOrthogonalXAxis() {
    let aP1 = Point3::new(0.0, 0.0, 0.0);
    let aP2 = Point3::new(0.0, 0.0, 10.0);

    let aMaker = MakeCone::from_points_radii(aP1, aP2, 4.0, 2.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    expect_orthogonal_x_axis(&aMaker.value().unwrap(), aP1, aP2);
}

#[test]
fn gce_MakeConeTest__FromFourPoints_YBranch_DoneAndOrthogonalXAxis() {
    let aP1 = Point3::new(0.0, 0.0, 0.0);
    let aP2 = Point3::new(0.0, 10.0, 10.0);
    let aP3 = Point3::new(3.0, 0.0, 0.0);
    let aP4 = Point3::new(5.0, 4.0, 4.0);

    let aMaker = MakeCone::through_points(aP1, aP2, aP3, aP4).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    expect_orthogonal_x_axis(&aMaker.value().unwrap(), aP1, aP2);
}

#[test]
fn gce_MakeConeTest__FromFourPoints_ZBranch_DoneAndOrthogonalXAxis() {
    let aP1 = Point3::new(0.0, 0.0, 0.0);
    let aP2 = Point3::new(0.0, 0.0, 10.0);
    let aP3 = Point3::new(4.0, 0.0, 0.0);
    let aP4 = Point3::new(2.0, 0.0, 5.0);

    let aMaker = MakeCone::through_points(aP1, aP2, aP3, aP4).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);

    expect_orthogonal_x_axis(&aMaker.value().unwrap(), aP1, aP2);
}
