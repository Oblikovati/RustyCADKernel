//! Direct translation of ExtremaPC_Parabola_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, dead_code)]
mod common;
#[allow(unused_imports)]
use common::*;
#[allow(unused_imports)]
use prelude::*;

// Test fixture for ExtremaPC_Parabola tests.
const THE_TOL: f64 = 1.0e-8;

//==================================================================================================
// Basic tests - point on axis
//==================================================================================================

#[test]
fn ExtremaPC_ParabolaTest__PointOnAxis_AtVertex() {
    // Parabola with focal length 5, vertex at origin
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_ParabolaTest__ZeroFocal_DegeneratesToLine() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(1.0, 2.0, 3.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        0.0,
    )
    .unwrap();
    let mut anEval = ExtremaPC::Parabola::with_domain(aParabola, Domain1D::new(-10.0, 10.0));

    let aResult = anEval
        .perform(Point3::new(5.0, 7.0, 3.0), THE_TOL, MM)
        .unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 4.0, THE_TOL);
    EXPECT_NEAR(
        aResult[0].point.distance(&Point3::new(5.0, 2.0, 3.0)),
        0.0,
        THE_TOL,
    );
}

#[test]
fn ExtremaPC_ParabolaTest__PointOnAxis_Positive() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(10.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // For parabola y^2 = 20x with point at (10,0,0) on axis,
    // the closest point is the vertex at (0,0,0) with distance = 10
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 100.0, THE_TOL); // Distance^2 to vertex
}

#[test]
fn ExtremaPC_ParabolaTest__PointOnAxis_Negative() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(-10.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Closest point is at vertex (0,0,0)
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 100.0, THE_TOL); // Distance to vertex
}

//==================================================================================================
// Point off axis
//==================================================================================================

#[test]
fn ExtremaPC_ParabolaTest__PointOffAxis_Positive() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(5.0, 10.0, 0.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify the point is on the parabola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnParab = elclib::value(aResult[aMinIdx].parameter, &aParabola);
    let aExpectedSq: f64 = aPoint.square_distance(&aPtOnParab);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, aExpectedSq, THE_TOL);
}

#[test]
fn ExtremaPC_ParabolaTest__PointOffAxis_Negative() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(5.0, -10.0, 0.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify the closest point is on the parabola and distance is consistent
    let aMinIdx: usize = aResult.min_index();
    let aPtOnParab = elclib::value(aResult[aMinIdx].parameter, &aParabola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnParab), 0.0, THE_TOL);
    EXPECT_NEAR(
        aResult[aMinIdx].square_distance,
        aPoint.square_distance(&aPtOnParab),
        THE_TOL,
    );

    // For negative Y, parameter should be negative
    EXPECT_LT(aResult[aMinIdx].parameter, 0.0);
}

#[test]
fn ExtremaPC_ParabolaTest__PointOffAxis_LargeY() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 50.0, 0.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify the closest point is on the parabola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnParab = elclib::value(aResult[aMinIdx].parameter, &aParabola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnParab), 0.0, THE_TOL);

    // Distance should be less than distance to vertex (50)
    EXPECT_LT((aResult.min_square_distance()).sqrt(), 50.0);

    // Parameter should be positive (point at positive Y)
    EXPECT_GT(aResult[aMinIdx].parameter, 0.0);
}

//==================================================================================================
// Point out of plane
//==================================================================================================

#[test]
fn ExtremaPC_ParabolaTest__PointAbovePlane() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(5.0, 10.0, 7.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Distance should include Z component
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_GE(aMinSqDist, 49.0); // At least 7^2
}

#[test]
fn ExtremaPC_ParabolaTest__PointBelowPlane() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 0.0, -5.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Closest point is at vertex, distance = 5
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 25.0, THE_TOL);
}

//==================================================================================================
// Bound constraint tests
//==================================================================================================

#[test]
fn ExtremaPC_ParabolaTest__BoundsPositive() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, -10.0, 0.0);

    // Create evaluator with positive domain [0, 10]
    let mut anEval = ExtremaPC::Parabola::with_domain(aParabola, Domain1D::new(0.0, 10.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Since bounds are positive and point is at negative Y,
    // closest point should be at u=0 (vertex)
    let aMinIdx: usize = aResult.min_index();
    EXPECT_GE(aResult[aMinIdx].parameter, 0.0);
}

#[test]
fn ExtremaPC_ParabolaTest__BoundsNegative() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(0.0, 10.0, 0.0);

    // Create evaluator with negative domain [-10, 0]
    let mut anEval = ExtremaPC::Parabola::with_domain(aParabola, Domain1D::new(-10.0, 0.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Since bounds are negative and point is at positive Y,
    // closest point should be at u=0 (vertex)
    let aMinIdx: usize = aResult.min_index();
    EXPECT_LE(aResult[aMinIdx].parameter, 0.0);
}

#[test]
fn ExtremaPC_ParabolaTest__NarrowBounds() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(5.0, 5.0, 0.0);

    // Create evaluator with narrow domain [4, 6]
    let mut anEval = ExtremaPC::Parabola::with_domain(aParabola, Domain1D::new(4.0, 6.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Parameter should be within bounds
    for i in 0..aResult.nb_ext() {
        EXPECT_GE(aResult[i].parameter, 4.0 - THE_TOL);
        EXPECT_LE(aResult[i].parameter, 6.0 + THE_TOL);
    }
}

//==================================================================================================
// Different focal lengths
//==================================================================================================

#[test]
fn ExtremaPC_ParabolaTest__SmallFocalLength() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        0.5,
    )
    .unwrap();
    let aPoint = Point3::new(1.0, 2.0, 0.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify point is on parabola and distance is consistent
    let aMinIdx: usize = aResult.min_index();
    let aPtOnParab = elclib::value(aResult[aMinIdx].parameter, &aParabola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnParab), 0.0, THE_TOL);
    EXPECT_NEAR(
        aResult[aMinIdx].square_distance,
        aPoint.square_distance(&aPtOnParab),
        THE_TOL,
    );
}

#[test]
fn ExtremaPC_ParabolaTest__LargeFocalLength() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        100.0,
    )
    .unwrap();
    let aPoint = Point3::new(50.0, 100.0, 0.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify point is on parabola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnParab = elclib::value(aResult[aMinIdx].parameter, &aParabola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnParab), 0.0, THE_TOL);
}

//==================================================================================================
// Point on parabola
//==================================================================================================

#[test]
fn ExtremaPC_ParabolaTest__PointOnParabola_AtVertex() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = elclib::value(0.0, &aParabola);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_ParabolaTest__PointOnParabola_Positive() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = elclib::value(10.0, &aParabola);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_ParabolaTest__PointOnParabola_Negative() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = elclib::value(-15.0, &aParabola);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

//==================================================================================================
// Parabola with offset
//==================================================================================================

#[test]
fn ExtremaPC_ParabolaTest__ParabolaWithOffset() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(100.0, 200.0, 50.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(100.0, 200.0, 50.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());

    // Point is at vertex
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR(aMinSqDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_ParabolaTest__ParabolaWithOffset_PointOff() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(100.0, 200.0, 50.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(110.0, 210.0, 50.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify point is on parabola and distance is consistent
    let aMinIdx: usize = aResult.min_index();
    let aPtOnParab = elclib::value(aResult[aMinIdx].parameter, &aParabola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnParab), 0.0, THE_TOL);
    EXPECT_NEAR(
        aResult[aMinIdx].square_distance,
        aPoint.square_distance(&aPtOnParab),
        THE_TOL,
    );
}

//==================================================================================================
// Different orientations
//==================================================================================================

#[test]
fn ExtremaPC_ParabolaTest__ParabolaInYZPlane() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(1.0, 0.0, 0.0),
            dir3(0.0, 1.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(3.0, 10.0, 5.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify extremum point is on the parabola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnParab = elclib::value(aResult[aMinIdx].parameter, &aParabola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnParab), 0.0, THE_TOL);

    // Distance should include X component (out of plane)
    EXPECT_GE(aResult.min_square_distance(), 9.0); // At least 3^2
}

#[test]
fn ExtremaPC_ParabolaTest__ParabolaInXZPlane() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 1.0, 0.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(10.0, 3.0, 5.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify extremum point is on the parabola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnParab = elclib::value(aResult[aMinIdx].parameter, &aParabola);
    EXPECT_NEAR(aResult[aMinIdx].point.distance(&aPtOnParab), 0.0, THE_TOL);

    // Distance should include Y component (out of plane)
    EXPECT_GE(aResult.min_square_distance(), 9.0); // At least 3^2
}

//==================================================================================================
// Verification tests
//==================================================================================================

#[test]
fn ExtremaPC_ParabolaTest__VerifyProjectedPoint() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(10.0, 15.0, 0.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify the projected point is on the parabola
    let aMinIdx: usize = aResult.min_index();
    let aPtOnCurve = elclib::value(aResult[aMinIdx].parameter, &aParabola);
    EXPECT_NEAR(aResult[aMinIdx].point.x(), aPtOnCurve.x(), THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.y(), aPtOnCurve.y(), THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.z(), aPtOnCurve.z(), THE_TOL);
}

#[test]
fn ExtremaPC_ParabolaTest__VerifyDistanceConsistency() {
    let aParabola = Parabola3::new(
        Axis2::with_x_direction(
            Point3::new(0.0, 0.0, 0.0),
            dir3(0.0, 0.0, 1.0),
            dir3(1.0, 0.0, 0.0),
        )
        .unwrap(),
        5.0,
    )
    .unwrap();
    let aPoint = Point3::new(8.0, 12.0, 3.0);

    let mut anEval = ExtremaPC::Parabola::new(aParabola);
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
fn ExtremaPC_ParabolaTest__PlanarDelegation_PreservesParameterAndHeight() {
    let mut anEvaluator3d = ExtremaPC::Parabola::new(
        Parabola3::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            2.0,
        )
        .unwrap(),
    );
    let mut anEvaluator2d = ExtremaPC2d::Parabola::new(
        Parabola2::new(
            Axis22d::new(Point2::new(0.0, 0.0), dir2(1.0, 0.0), true).unwrap(),
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
