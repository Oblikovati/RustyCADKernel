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

//! Direct translation of ExtremaPC_Hyperbola_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, dead_code)]
mod common;
#[allow(unused_imports)]
use common::*;
#[allow(unused_imports)]
use prelude::*;

// Test fixture for ExtremaPC_Hyperbola tests.
const THE_TOL: f64 = 1.0e-8;

//==================================================================================================
// Basic tests - point on major axis
//==================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__PointOnMajorAxis_Outside() {
    // Hyperbola with a=20, b=10
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(50.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // For point (50, 0) on major axis, the closest point is NOT the vertex!
    // The extremum condition gives cosh(u) = 2 (u ~= 1.317), yielding
    // closest point at (40, ~17.3, 0) with distance ~= 20.
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 400.0, 1.0);
}

#[test]
fn ExtremaPC_HyperbolaTest__PointOnMajorAxis_AtVertex() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_HyperbolaTest__PointOnMajorAxis_BetweenBranches() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 0.0, 0.0); // Center

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    // Closest point is at vertex (20, 0, 0), distance = 20
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR((aMinSqDist).sqrt(), 20.0, THE_TOL);
}

//==================================================================================================
// Point off axis tests
//==================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__PointOffAxis_FirstQuadrant() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 15.0, 0.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify the point is on the hyperbola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnHyper = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    let aExpectedSq: f64 = aPoint.square_distance(&aPtOnHyper);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, aExpectedSq, THE_TOL);
}

#[test]
fn ExtremaPC_HyperbolaTest__PointOffAxis_FourthQuadrant() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, -15.0, 0.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify the closest point is on the hyperbola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnHyper = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnHyper), 0.0, THE_TOL);
    EXPECT_NEAR(
        aResult[aMinIdx].square_distance,
        aPoint.square_distance(&aPtOnHyper),
        THE_TOL,
    );

    // For negative Y, parameter should be negative
    EXPECT_LT(aResult[aMinIdx].parameter, 0.0);
}

#[test]
fn ExtremaPC_HyperbolaTest__PointInsideAsymptotes() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(25.0, 5.0, 0.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify the closest point is on the hyperbola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnHyper = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnHyper), 0.0, THE_TOL);

    // Distance should be less than distance to center (sqrt(25^2+5^2) = 25.5)
    EXPECT_LT((aResult.min_square_distance()).sqrt(), 26.0);
}

//==================================================================================================
// Point out of plane
//==================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__PointAbovePlane() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 0.0, 5.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Distance should include Z component
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_GE(aMinSqDist, 25.0); // At least 5^2
}

#[test]
fn ExtremaPC_HyperbolaTest__PointBelowPlane() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(20.0, 0.0, -8.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    // Closest point is at vertex (20, 0, 0), distance = 8
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 64.0, THE_TOL);
}

//==================================================================================================
// Bound constraint tests
//==================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__BoundsPositive() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, -20.0, 0.0);

    // Create evaluator with positive parameter domain [0, 10]
    let mut anEval = ExtremaPC::Hyperbola::with_domain(aHyperbola, Domain1D::new(0.0, 10.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // All parameters should be positive
    for i in 0..aResult.nb_ext() {
        EXPECT_GE(aResult[i].parameter, 0.0 - THE_TOL);
    }
}

#[test]
fn ExtremaPC_HyperbolaTest__BoundsNegative() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 20.0, 0.0);

    // Create evaluator with negative parameter domain [-10, 0]
    let mut anEval = ExtremaPC::Hyperbola::with_domain(aHyperbola, Domain1D::new(-10.0, 0.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // All parameters should be negative or zero
    for i in 0..aResult.nb_ext() {
        EXPECT_LE(aResult[i].parameter, 0.0 + THE_TOL);
    }
}

#[test]
fn ExtremaPC_HyperbolaTest__NarrowBounds() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 10.0, 0.0);

    // Create evaluator with narrow domain [0.5, 1.5]
    let mut anEval = ExtremaPC::Hyperbola::with_domain(aHyperbola, Domain1D::new(0.5, 1.5));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Parameter should be within bounds
    for i in 0..aResult.nb_ext() {
        EXPECT_GE(aResult[i].parameter, 0.5 - THE_TOL);
        EXPECT_LE(aResult[i].parameter, 1.5 + THE_TOL);
    }
}

//==================================================================================================
// Different semi-axes
//==================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__SmallSemiAxes() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        2.0,
        1.0,
    )
    .unwrap();
    let aPoint = Point3::new(5.0, 2.0, 0.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify point is on hyperbola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnHyper = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnHyper), 0.0, THE_TOL);
    EXPECT_NEAR(
        aResult[aMinIdx].square_distance,
        aPoint.square_distance(&aPtOnHyper),
        THE_TOL,
    );
}

#[test]
fn ExtremaPC_HyperbolaTest__LargeSemiAxes() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        200.0,
        100.0,
    )
    .unwrap();
    let aPoint = Point3::new(300.0, 100.0, 0.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify point is on hyperbola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnHyper = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnHyper), 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_HyperbolaTest__NearlyRectangular() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        10.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(20.0, 15.0, 0.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify point is on hyperbola and distance is consistent
    let aMinIdx: usize = aResult.min_index();
    let aPtOnHyper = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnHyper), 0.0, THE_TOL);
    EXPECT_NEAR(
        aResult[aMinIdx].square_distance,
        aPoint.square_distance(&aPtOnHyper),
        THE_TOL,
    );
}

//==================================================================================================
// Point on hyperbola
//==================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__PointOnHyperbola_AtVertex() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = elclib::value(0.0, &aHyperbola);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_HyperbolaTest__PointOnHyperbola_Positive() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = elclib::value(1.5, &aHyperbola);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_HyperbolaTest__PointOnHyperbola_Negative() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = elclib::value(-2.0, &aHyperbola);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

//==================================================================================================
// Hyperbola with offset
//==================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__HyperbolaWithOffset() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(100.0, 200.0, 50.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(120.0, 200.0, 50.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    // Point is at vertex
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_HyperbolaTest__HyperbolaWithOffset_PointOff() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(100.0, 200.0, 50.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(150.0, 210.0, 50.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify point is on hyperbola and distance is consistent
    let aMinIdx: usize = aResult.min_index();
    let aPtOnHyper = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnHyper), 0.0, THE_TOL);
    EXPECT_NEAR(
        aResult[aMinIdx].square_distance,
        aPoint.square_distance(&aPtOnHyper),
        THE_TOL,
    );
}

//==================================================================================================
// Different orientations
//==================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__HyperbolaInYZPlane() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(5.0, 30.0, 10.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify point is on hyperbola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnHyper = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnHyper), 0.0, THE_TOL);

    // Distance should include X component (out of plane)
    EXPECT_GE(aResult.min_square_distance(), 25.0); // At least 5^2
}

#[test]
fn ExtremaPC_HyperbolaTest__HyperbolaInXZPlane() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 1.0, 0.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(30.0, 5.0, 10.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify point is on hyperbola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnHyper = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnHyper), 0.0, THE_TOL);

    // Distance should include Y component (out of plane)
    EXPECT_GE(aResult.min_square_distance(), 25.0); // At least 5^2
}

//==================================================================================================
// Verification tests
//==================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__VerifyProjectedPoint() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(35.0, 20.0, 0.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify the projected point is on the hyperbola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnCurve = elclib::value(aResult[aMinIdx].parameter, &aHyperbola);
    EXPECT_NEAR(aResult[aMinIdx].point.x(), aPtOnCurve.x(), THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.y(), aPtOnCurve.y(), THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.z(), aPtOnCurve.z(), THE_TOL);
}

#[test]
fn ExtremaPC_HyperbolaTest__VerifyDistanceConsistency() {
    let aHyperbola = Hyperbola3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        20.0,
        10.0,
    )
    .unwrap();
    let aPoint = Point3::new(40.0, 25.0, 3.0);

    let mut anEval = ExtremaPC::Hyperbola::new(aHyperbola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify distance matches point distance
    let aMinIdx: usize = aResult.min_index();
    let aComputedSq: f64 = aPoint.square_distance(&aResult[aMinIdx].point);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, aComputedSq, THE_TOL);
}

//=================================================================================================

#[test]
fn ExtremaPC_HyperbolaTest__PlanarDelegation_PreservesParameterAndHeight() {
    let mut anEvaluator3d = ExtremaPC::Hyperbola::new(
        Hyperbola3::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            3.0,
            1.5,
        )
        .unwrap(),
    );
    let mut anEvaluator2d = ExtremaPC2d::Hyperbola::new(
        Hyperbola2::new(
            Axis22d::new(Point2::new(0.0, 0.0), dir2(1.0, 0.0), true).unwrap(),
            3.0,
            1.5,
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
