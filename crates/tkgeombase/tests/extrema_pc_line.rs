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

//! Direct translation of ExtremaPC_Line_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
#[allow(unused_imports)]
use common::*;
#[allow(unused_imports)]
use prelude::*;

// Test fixture for ExtremaPC_Line tests.
// With endpoint handling enabled, line extrema include:
// - 1 minimum (the projection or clamped point)
// - 0-2 maxima at endpoints (when they are local maxima)
const THE_TOL: f64 = 1.0e-10;

#[test]
fn ExtremaPC_LineTest__SingletonDomain_IsMinimumAndMaximum() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(2.0, 2.0));

    let aResult = anEval
        .perform_with_endpoints(Point3::new(5.0, 3.0, 0.0), THE_TOL, MM)
        .unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    assert!(aResult[0].is_minimum);
    assert!(aResult[0].is_maximum);
    EXPECT_NEAR(aResult[0].parameter, 2.0, THE_TOL);
}

#[test]
fn ExtremaPC_LineTest__SingletonProjection_IsMinimumAndMaximum() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(2.0, 2.0));

    let aResult = anEval
        .perform_with_endpoints(Point3::new(2.0, 3.0, 0.0), THE_TOL, MM)
        .unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    assert!(aResult[0].is_minimum);
    assert!(aResult[0].is_maximum);
}

#[test]
fn ExtremaPC_LineTest__EffectivelyUnboundedDomain_DoesNotAddEndpoints() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-1.0e100, 1.0e100));

    let aResult = anEval
        .perform_with_endpoints(Point3::new(2.0, 3.0, 0.0), THE_TOL, MM)
        .unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 2.0, THE_TOL);
}

#[test]
fn ExtremaPC_LineTest__LowerBoundedDomain_PreservesFiniteEndpoint() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(0.0, 1.0e100));

    let aResult = anEval
        .perform_with_endpoints(Point3::new(-2.0, 3.0, 0.0), THE_TOL, MM)
        .unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.0, THE_TOL);
    assert!(aResult[0].is_minimum);
}

#[test]
fn ExtremaPC_LineTest__UpperBoundedDomain_PreservesFiniteEndpoint() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-1.0e100, 0.0));

    let aResult = anEval
        .perform_with_endpoints(Point3::new(2.0, 3.0, 0.0), THE_TOL, MM)
        .unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.0, THE_TOL);
    assert!(aResult[0].is_minimum);
}

#[test]
fn ExtremaPC_LineTest__ReversedDomain_IsInvalidInput() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(1.0, -1.0));

    let aResult = anEval
        .perform(Point3::new(0.0, 0.0, 0.0), THE_TOL, MM)
        .unwrap();

    assert_eq!(aResult.status, Status::InvalidInput);
}

//==================================================================================================
// Basic projection tests
// When projection is inside bounds: 3 extrema (1 min projection + 2 max endpoints)
//==================================================================================================

#[test]
fn ExtremaPC_LineTest__PointOnLine_AtOrigin() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(0.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max (endpoints)

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 0.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 0.0, THE_TOL);
    assert!(aResult[aMinIdx].is_minimum);
}

#[test]
fn ExtremaPC_LineTest__PointOnLine_Positive() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(5.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max (endpoints)

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_LineTest__PointOnLine_Negative() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(-7.5, 0.0, 0.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max (endpoints)

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, -7.5, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_LineTest__PointOffLine_YOffset() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(5.0, 3.0, 0.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max (endpoints)

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 9.0, THE_TOL); // 3^2 = 9
}

#[test]
fn ExtremaPC_LineTest__PointOffLine_ZOffset() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(5.0, 0.0, 4.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max (endpoints)

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 16.0, THE_TOL); // 4^2 = 16
}

#[test]
fn ExtremaPC_LineTest__PointOffLine_YZOffset() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(5.0, 3.0, 4.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max (endpoints)

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 25.0, THE_TOL); // 3^2 + 4^2 = 25
}

//==================================================================================================
// Bound constraint tests
// When projection is outside bounds: 2 extrema (1 min at closer endpoint + 1 max at farther)
//==================================================================================================

#[test]
fn ExtremaPC_LineTest__ProjectionOutsideBounds_Left() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(-50.0, 3.0, 4.0);

    // Domain [0, 100] - projection at -50 is outside bounds (left of 0)
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(0.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2); // min at 0, max at 100

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 0.0, THE_TOL); // Clamped to lower bound
}

#[test]
fn ExtremaPC_LineTest__ProjectionOutsideBounds_Right() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(150.0, 3.0, 4.0);

    // Domain [0, 100] - projection at 150 is outside bounds (right of 100)
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(0.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2); // min at 100, max at 0

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 100.0, THE_TOL); // Clamped to upper bound
}

#[test]
fn ExtremaPC_LineTest__ProjectionAtLowerBound() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(10.0, 5.0, 0.0);

    // Domain [10, 100] - projection at 10 is exactly at lower bound
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(10.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2); // min at 10, max at 100

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_LineTest__ProjectionAtUpperBound() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(100.0, 5.0, 0.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2); // min at 100, max at 0

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 100.0, THE_TOL);
}

#[test]
fn ExtremaPC_LineTest__NarrowBounds() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(50.0, 1.0, 0.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // min at 50, max at endpoints

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 50.0, THE_TOL);
}

//==================================================================================================
// Different line orientations
//==================================================================================================

#[test]
fn ExtremaPC_LineTest__LineAlongY() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 1.0, 0.0));
    let aPoint = Point3::new(3.0, 5.0, 4.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 25.0, THE_TOL); // 3^2 + 4^2 = 25
}

#[test]
fn ExtremaPC_LineTest__LineAlongZ() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0));
    let aPoint = Point3::new(3.0, 4.0, 10.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 10.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 25.0, THE_TOL); // 3^2 + 4^2 = 25
}

#[test]
fn ExtremaPC_LineTest__LineDiagonal_XY() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 1.0, 0.0));
    let aPoint = Point3::new(5.0, 5.0, 3.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    // Parameter along diagonal: (5,5,0) projected = 5*sqrt(2)
    let aExpectedParam: f64 = 5.0 * (2.0_f64).sqrt();
    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, aExpectedParam, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 9.0, THE_TOL); // Z offset = 3
}

#[test]
fn ExtremaPC_LineTest__LineDiagonal_XYZ() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 1.0, 1.0));
    let aPoint = Point3::new(3.0, 3.0, 3.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max
    // Point is on the line
    EXPECT_NEAR(aResult.min_square_distance(), 0.0, THE_TOL);
}

//==================================================================================================
// Offset line origin tests
//==================================================================================================

#[test]
fn ExtremaPC_LineTest__LineWithOffset_XAxis() {
    let aLine = Line3::new(Point3::new(10.0, 20.0, 30.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(15.0, 20.0, 30.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL); // 15 - 10 = 5
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_LineTest__LineWithOffset_PointOff() {
    let aLine = Line3::new(Point3::new(10.0, 20.0, 30.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(15.0, 23.0, 34.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 25.0, THE_TOL); // 3^2 + 4^2 = 25
}

//==================================================================================================
// Negative parameter tests
//==================================================================================================

#[test]
fn ExtremaPC_LineTest__NegativeParameters() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(-25.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, -25.0, THE_TOL);
}

#[test]
fn ExtremaPC_LineTest__NegativeBoundsClamp() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(-5.0, 1.0, 0.0);

    // Domain [-50, -10] - projection at -5 is outside (right of -10), clamps to -10
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-50.0, -10.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2); // min at -10, max at -50

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, -10.0, THE_TOL); // Clamped to upper bound
}

//==================================================================================================
// Large coordinate tests
//==================================================================================================

#[test]
fn ExtremaPC_LineTest__LargeCoordinates() {
    let aLine = Line3::new(Point3::new(1e6, 1e6, 1e6), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(1e6 + 100.0, 1e6 + 3.0, 1e6 + 4.0);

    // Use domain [-100, 200] so projection at 100 is inside bounds
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 200.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 100.0, 1e-6);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 25.0, 1e-6);
}

#[test]
fn ExtremaPC_LineTest__SmallCoordinates() {
    let aLine = Line3::new(Point3::new(1e-6, 1e-6, 1e-6), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(1e-6 + 1e-3, 1e-6 + 3e-4, 1e-6 + 4e-4);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 1e-3, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 25e-8, THE_TOL); // (3e-4)^2 + (4e-4)^2
}

//==================================================================================================
// Point verification tests
//==================================================================================================

#[test]
fn ExtremaPC_LineTest__VerifyProjectedPoint() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(7.0, 5.0, 0.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    // Verify the minimum point (projected point)
    let aMinIdx: usize = aResult.min_index();
    let aExpectedPt = Point3::new(7.0, 0.0, 0.0);
    EXPECT_NEAR(aResult[aMinIdx].point.x(), aExpectedPt.x(), THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.y(), aExpectedPt.y(), THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.z(), aExpectedPt.z(), THE_TOL);
}

#[test]
fn ExtremaPC_LineTest__VerifyDistanceConsistency() {
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(10.0, 6.0, 8.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(-100.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max

    // Verify distance matches point distance for minimum
    let aMinIdx: usize = aResult.min_index();
    let aComputedDist: f64 = aPoint.square_distance(&aResult[aMinIdx].point);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, aComputedDist, THE_TOL);
    EXPECT_NEAR(aResult.min_square_distance(), 100.0, THE_TOL); // 6^2 + 8^2 = 100
}

//==================================================================================================
// Endpoint handling verification
//==================================================================================================

#[test]
fn ExtremaPC_LineTest__EndpointsAsMaxima() {
    // For a line segment with projection inside, endpoints should be local maxima
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));
    let aPoint = Point3::new(5.0, 1.0, 0.0); // Point above line at X=5

    // Domain [0, 10] for specific endpoint checks
    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(0.0, 10.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    // Should have 3 extrema: 1 minimum (projection) + 2 maxima (endpoints)
    assert_eq!(aResult.nb_ext(), 3);

    // Verify minimum is at the projection
    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 1.0, THE_TOL); // 1^2

    // Verify maximum is at one of the endpoints
    let aMaxIdx: usize = aResult.max_index();
    let aMaxParam: f64 = aResult[aMaxIdx].parameter;
    assert!((aMaxParam - 0.0).abs() < THE_TOL || (aMaxParam - 10.0).abs() < THE_TOL);
}

//=================================================================================================

#[test]
fn ExtremaPC_LineTest__PlanarDelegation_PreservesParameterAndHeight() {
    let mut anEvaluator3d =
        ExtremaPC::Line::new(Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0)));
    let mut anEvaluator2d =
        ExtremaPC2d::Line::new(Line2::new(Point2::new(0.0, 0.0), dir2(1.0, 0.0)));

    let aResult3d = anEvaluator3d
        .perform(Point3::new(8.0, 3.0, 6.0), THE_TOL, MM)
        .unwrap();
    let aResult2d = anEvaluator2d.perform(Point2::new(8.0, 3.0), THE_TOL, MM);
    assert_eq!(aResult3d.nb_ext(), aResult2d.nb_ext());
    assert_eq!(aResult3d.nb_ext(), 1);
    EXPECT_NEAR(aResult3d[0].parameter, aResult2d[0].parameter, THE_TOL);
    EXPECT_NEAR(
        aResult3d[0].square_distance,
        aResult2d[0].square_distance + 36.0,
        THE_TOL,
    );
}
