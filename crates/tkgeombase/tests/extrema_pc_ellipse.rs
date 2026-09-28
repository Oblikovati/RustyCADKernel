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

//! Direct translation of ExtremaPC_Ellipse_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, dead_code)]
mod common;
#[allow(unused_imports)]
use common::*;
#[allow(unused_imports)]
use prelude::*;

// Test fixture for ExtremaPC_Ellipse tests.
const THE_TOL: f64 = 1.0e-8;
const THE_2PI: f64 = 2.0 * PI;
const THE_PI: f64 = PI;
const THE_PI_2: f64 = PI / 2.0;

#[test]
fn ExtremaPC_EllipseTest__ShiftedFullPeriod_UniqueRepresentatives() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 0.0, 0.0);

    let aUMin: f64 = -10.0;
    let mut anEval =
        ExtremaPC::Ellipse::with_domain(anEllipse, Domain1D::new(aUMin, aUMin + THE_2PI));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2);
    for anIndex in 0..aResult.nb_ext() {
        EXPECT_GE(aResult[anIndex].parameter, aUMin);
        EXPECT_LE(aResult[anIndex].parameter, aUMin + THE_2PI);
    }
}

#[test]
fn ExtremaPC_EllipseTest__SingletonDomain_InvalidTolerance() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let mut anEval = ExtremaPC::Ellipse::with_domain(anEllipse, Domain1D::new(0.5, 0.5));

    let aResult = anEval
        .perform_with_endpoints(Point3::new(30.0, 0.0, 0.0), 0.0, MM)
        .unwrap();

    assert_eq!(aResult.status, Status::InvalidInput);
}

//==================================================================================================
// Basic tests - point on major axis
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__PointOnMajorAxis_Outside() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    let aMinSqDist: f64 = aResult.min_square_distance();
    // Closest point is (20, 0, 0), distance = 10
    EXPECT_NEAR((aMinSqDist).sqrt(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__PointOnMajorAxis_Inside() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(10.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // For a point inside an ellipse on the major axis, the closest point is NOT at the
    // major vertex. For ellipse a=20, b=10 and point (10,0,0):
    // The extremum condition gives cos(u) = 2/3, yielding:
    //   closest point at (40/3, +/-10*sqrt(5)/3, 0) with distance = sqrt(600/9) ~= 8.165
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 600.0 / 9.0, 0.01);
}

#[test]
fn ExtremaPC_EllipseTest__PointOnMajorAxis_NegativeSide() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(-30.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    let aMinSqDist: f64 = aResult.min_square_distance();
    // Closest point is (-20, 0, 0), distance = 10
    EXPECT_NEAR((aMinSqDist).sqrt(), 10.0, THE_TOL);
}

//==================================================================================================
// Point on minor axis
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__PointOnMinorAxis_Outside() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 20.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    let aMinSqDist: f64 = aResult.min_square_distance();
    // Closest point is (0, 10, 0), distance = 10
    EXPECT_NEAR((aMinSqDist).sqrt(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__PointOnMinorAxis_Inside() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 5.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    let aMinSqDist: f64 = aResult.min_square_distance();
    // Closest point is (0, 10, 0), distance = 5
    EXPECT_NEAR((aMinSqDist).sqrt(), 5.0, THE_TOL);
}

//==================================================================================================
// Point in quadrants
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__PointInFirstQuadrant() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(15.0, 15.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // Verify distance is reasonable
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_GT(aMinSqDist, 0.0);

    // Verify the closest point is actually on the ellipse
    let aMinIdx: usize = aResult.min_index();
    let aPtOnEllipse = elclib::value(aResult[aMinIdx].parameter, &anEllipse);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnEllipse), 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__PointInSecondQuadrant() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(-15.0, 8.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // Verify the point on ellipse
    let aMinIdx: usize = aResult.min_index();
    let aPtOnEllipse = elclib::value(aResult[aMinIdx].parameter, &anEllipse);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnEllipse), 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__PointInThirdQuadrant() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(-12.0, -7.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // Verify the closest point is on the ellipse and distance is reasonable
    let aMinIdx: usize = aResult.min_index();
    let aPtOnEllipse = elclib::value(aResult[aMinIdx].parameter, &anEllipse);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnEllipse), 0.0, THE_TOL);

    // Distance to closest point should be less than distance to origin
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_LT(aMinDist, aPoint.distance(&Point3::new(0.0, 0.0, 0.0)));
}

#[test]
fn ExtremaPC_EllipseTest__PointInFourthQuadrant() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(18.0, -5.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // Verify the closest point is on the ellipse
    let aMinIdx: usize = aResult.min_index();
    let aPtOnEllipse = elclib::value(aResult[aMinIdx].parameter, &anEllipse);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnEllipse), 0.0, THE_TOL);

    // Distance to closest point should be less than distance to origin
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_LT(aMinDist, aPoint.distance(&Point3::new(0.0, 0.0, 0.0)));
}

//==================================================================================================
// Degenerate case - point at center
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__PointAtCenter_Degenerate() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    // For a non-circular ellipse (a != b) with point at center:
    // We get 4 extrema: 2 minima at minor axis vertices (+/-b), 2 maxima at major axis vertices
    // (+/-a)
    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 4);

    // Verify minimum distance equals minor radius (b = 10)
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 10.0, THE_TOL);

    // Verify maximum distance equals major radius (a = 20)
    let aMaxDist: f64 = (aResult.max_square_distance()).sqrt();
    EXPECT_NEAR(aMaxDist, 20.0, THE_TOL);

    // Verify all extrema are on the ellipse
    for i in 0..aResult.nb_ext() {
        let aPtOnEllipse = elclib::value(aResult[i].parameter, &anEllipse);
        EXPECT_NEAR(aResult[i].point.distance(&aPtOnEllipse), 0.0, THE_TOL);
    }
}

//==================================================================================================
// Point out of plane
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__PointAbovePlane() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 0.0, 5.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // Closest point is (20, 0, 0), distance = sqrt(10^2 + 5^2) = sqrt(125)
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 125.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__PointBelowPlane() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 20.0, -8.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // Closest point is (0, 10, 0), distance = sqrt(10^2 + 8^2) = sqrt(164)
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 164.0, THE_TOL);
}

//==================================================================================================
// Partial arc tests
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__PartialArc_FirstQuadrant() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Minimum should be at u=0
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR((aMinSqDist).sqrt(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__PartialArc_SecondHalf() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(-30.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Minimum should be at u=PI
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR((aMinSqDist).sqrt(), 10.0, THE_TOL);
}

//==================================================================================================
// Different ellipse configurations
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__EllipseWithOffset() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(100.0, 200.0, 50.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(130.0, 200.0, 50.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // Closest point is at (120, 200, 50), distance = 10
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR((aMinSqDist).sqrt(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__EllipseInYZPlane() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(5.0, 30.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify that valid extrema are found and projected points are on the ellipse
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_GT(aMinSqDist, 0.0);

    // Verify extremum point is on the ellipse (project back and check)
    for i in 0..aResult.nb_ext() {
        let anExt = &aResult[i];
        let aCheck = elclib::value(anExt.parameter, &anEllipse);
        EXPECT_NEAR(aCheck.distance(&anExt.point), 0.0, 1e-7);
    }
}

#[test]
fn ExtremaPC_EllipseTest__NearlyCircularEllipse() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        10.0,
        9.9,
    )
    .unwrap();
    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // Closest point is at (10, 0, 0), distance = 10
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR((aMinSqDist).sqrt(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__VeryFlatEllipse() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        2.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 10.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 2);

    // Closest point is at (0, 2, 0), distance = 8
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR((aMinSqDist).sqrt(), 8.0, THE_TOL);
}

//==================================================================================================
// Point on ellipse
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__PointOnEllipse_MajorVertex() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__PointOnEllipse_MinorVertex() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 10.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_EllipseTest__PointOnEllipse_General() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    // Point at u = PI/4
    let aPoint = elclib::value(THE_PI / 4.0, &anEllipse);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

//==================================================================================================
// Number of extrema tests
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__FourExtrema_PointInsideEllipse() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(5.0, 3.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    // For a point inside an ellipse, there should be at least 4 extrema
    // (2 minima and 2 maxima)
    EXPECT_GE(aResult.nb_ext(), 4);
}

#[test]
fn ExtremaPC_EllipseTest__TwoExtrema_PointOnAxis() {
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    // For a point on the major axis outside, there should be 2 extrema
    assert_eq!(aResult.nb_ext(), 2);
}

//==================================================================================================
// Narrow parameter range tests - verify relative step works correctly
//==================================================================================================

#[test]
fn ExtremaPC_EllipseTest__NarrowRange_VerySmall() {
    // Test with a very narrow parameter range (0.05 radians ~ 3 degrees)
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(22.0, 2.0, 0.0);

    let aUMin: f64 = 0.0;
    let aUMax: f64 = 0.05; // Very small range

    // Create evaluator with restricted domain
    let mut anEval = ExtremaPC::Ellipse::with_domain(anEllipse, Domain1D::new(aUMin, aUMax));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    // Should find at least the endpoint extremum
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify all extrema are within parameter bounds
    for i in 0..aResult.nb_ext() {
        EXPECT_GE(aResult[i].parameter, aUMin - THE_TOL);
        EXPECT_LE(aResult[i].parameter, aUMax + THE_TOL);
    }
}

#[test]
fn ExtremaPC_EllipseTest__NarrowRange_MediumSmall() {
    // Test with a moderately narrow parameter range (0.2 radians ~ 11 degrees)
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(25.0, 5.0, 0.0);

    let aUMin: f64 = THE_PI / 6.0 - 0.1;
    let aUMax: f64 = THE_PI / 6.0 + 0.1;

    // Create evaluator with restricted domain
    let mut anEval = ExtremaPC::Ellipse::with_domain(anEllipse, Domain1D::new(aUMin, aUMax));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    // Verify distance calculation is consistent
    for i in 0..aResult.nb_ext() {
        let aPtOnEllipse = elclib::value(aResult[i].parameter, &anEllipse);
        EXPECT_NEAR(aPtOnEllipse.distance(&aResult[i].point), 0.0, THE_TOL);
        let aExpectedSqDist: f64 = aPoint.square_distance(&aPtOnEllipse);
        EXPECT_NEAR(aResult[i].square_distance, aExpectedSqDist, THE_TOL);
    }
}

#[test]
fn ExtremaPC_EllipseTest__NarrowRange_TinyArc() {
    // Test with an extremely narrow parameter range (0.001 radians ~ 0.06 degrees)
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(21.0, 0.5, 0.0);

    let aUMin: f64 = 0.0;
    let aUMax: f64 = 0.001; // Extremely narrow

    // Create evaluator with restricted domain
    let mut anEval = ExtremaPC::Ellipse::with_domain(anEllipse, Domain1D::new(aUMin, aUMax));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2);
    assert_eq!(aResult[0].parameter, aUMin);
    assert!(aResult[0].is_maximum);
    assert_eq!(aResult[1].parameter, aUMax);
    assert!(aResult[1].is_minimum);
}

#[test]
fn ExtremaPC_EllipseTest__LargeRange_FullCircle() {
    // Verify full circle case still works correctly
    let anEllipse = Ellipse3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(5.0, 3.0, 0.0);

    let mut anEval = ExtremaPC::Ellipse::new(anEllipse);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    // For point inside ellipse, expect 4 extrema
    EXPECT_GE(aResult.nb_ext(), 4);

    // Verify all distances are positive
    for i in 0..aResult.nb_ext() {
        EXPECT_GT(aResult[i].square_distance, 0.0);
    }
}

//=================================================================================================

#[test]
fn ExtremaPC_EllipseTest__PlanarDelegation_PreservesParameterAndHeight() {
    let mut anEvaluator3d = ExtremaPC::Ellipse::new(
        Ellipse3::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            5.0,
            2.0,
        )
        .unwrap(),
    );
    let mut anEvaluator2d = ExtremaPC2d::Ellipse::new(
        Ellipse2::new(
            Axis22d::new(Point2::new(0.0, 0.0), dir2(1.0, 0.0), true).unwrap(),
            5.0,
            2.0,
        )
        .unwrap(),
    );
    let aResult3d = anEvaluator3d
        .perform(Point3::new(8.0, 3.0, 6.0), THE_TOL, MM)
        .unwrap();
    let aResult2d = anEvaluator2d.perform(Point2::new(8.0, 3.0), THE_TOL, MM);
    assert_eq!(aResult3d.nb_ext(), aResult2d.nb_ext());
    for anIndex in 0..aResult3d.nb_ext() {
        EXPECT_NEAR(
            aResult3d[anIndex].parameter,
            aResult2d[anIndex].parameter,
            THE_TOL,
        );
        EXPECT_NEAR(
            aResult3d[anIndex].square_distance,
            aResult2d[anIndex].square_distance + 36.0,
            THE_TOL,
        );
    }
}
