//! Direct translation of ExtremaPC_OffsetCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, dead_code)]
mod common;
#[allow(unused_imports)]
use common::*;
#[allow(unused_imports)]
use prelude::*;

// Test fixture for ExtremaPC_OffsetCurve tests.
const THE_TOL: f64 = 1.0e-6;
const THE_2PI: f64 = 2.0 * PI;
const THE_PI: f64 = PI;

//==================================================================================================
// Offset of Circle tests
//==================================================================================================

#[test]
fn ExtremaPC_OffsetCurveTest__OffsetCircle_PointOutside() {
    // Create a circle of radius 10 in XY plane
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with offset 5 (outer circle of radius 15)
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    // Point outside the offset circle
    let aPoint = Point3::new(30.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Minimum distance should be 30 - 15 = 15
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 15.0, THE_TOL);
}

#[test]
fn ExtremaPC_OffsetCurveTest__OffsetCircle_PointInside() {
    // Create a circle of radius 10 in XY plane
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with offset 5 (outer circle of radius 15)
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    // Point inside the offset circle but outside original
    let aPoint = Point3::new(12.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Minimum distance should be 15 - 12 = 3
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 3.0, THE_TOL);
}

#[test]
fn ExtremaPC_OffsetCurveTest__OffsetCircle_NegativeOffset() {
    // Create a circle of radius 10 in XY plane
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with negative offset (inner circle of radius 5)
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), -5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    // Point outside the inner circle
    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Minimum distance should be 20 - 5 = 15
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 15.0, THE_TOL);
}

#[test]
fn ExtremaPC_OffsetCurveTest__OffsetCircle_PointOnOffsetCurve() {
    // Create a circle of radius 10 in XY plane
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with offset 5 (outer circle of radius 15)
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    // Point exactly on the offset curve
    let aPoint = Point3::new(15.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Minimum distance should be ~0
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 0.0, THE_TOL);
}

//==================================================================================================
// Offset of Ellipse tests
//==================================================================================================

#[test]
fn ExtremaPC_OffsetCurveTest__OffsetEllipse_PointOutside() {
    // Create an ellipse with major radius 20, minor radius 10
    let anEllipse = Handle::new(
        GeomEllipse::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            20.0,
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with offset 5
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&anEllipse.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    // Point on major axis outside
    let aPoint = Point3::new(40.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Offset ellipse at u=0 is at (25, 0, 0)
    // Minimum distance should be 40 - 25 = 15
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 15.0, THE_TOL);
}

#[test]
fn ExtremaPC_OffsetCurveTest__OffsetEllipse_PointOnMinorAxis() {
    // Create an ellipse with major radius 20, minor radius 10
    let anEllipse = Handle::new(
        GeomEllipse::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            20.0,
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with offset 5
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&anEllipse.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    // Point on minor axis
    let aPoint = Point3::new(0.0, 25.0, 0.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Offset ellipse at u=PI/2 is at (0, 15, 0)
    // Minimum distance should be 25 - 15 = 10
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 10.0, THE_TOL);
}

//==================================================================================================
// Offset of Line tests
//==================================================================================================

#[test]
fn ExtremaPC_OffsetCurveTest__OffsetLine_PointNear() {
    // Create a line along X axis
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));

    // Create offset curve with offset 5 in Y direction
    // For a line in XY plane with reference direction Z, offset is in +Y
    let anOffsetCurve =
        Handle::new(GeomOffsetCurve::new(&aLine.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap());
    let anAdaptor = GeomAdaptorCurve::with_range(&anOffsetCurve.cast(), 0.0, 100.0).unwrap();

    // Query where the offset curve actually is
    let aOffsetPt = anAdaptor.value(50.0).unwrap();

    // Point near the offset line
    let aPoint = Point3::new(50.0, aOffsetPt.y() + 5.0, 0.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Distance should be approximately 5
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 5.0, THE_TOL);
}

//==================================================================================================
// Partial range tests
//==================================================================================================

#[test]
fn ExtremaPC_OffsetCurveTest__PartialRange_FirstQuadrant() {
    // Create a circle of radius 10 in XY plane
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with offset 5
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    // Point in first quadrant
    let aPoint = Point3::new(20.0, 20.0, 0.0);

    // Create evaluator with domain limited to first quadrant [0, PI/2]
    let mut anEval =
        ExtremaPC::OffsetCurve::with_domain(&anAdaptor, Domain1D::new(0.0, THE_PI / 2.0)).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Check that extremum is in the first quadrant
    for i in 0..aResult.nb_ext() {
        EXPECT_GE(aResult[i].parameter, -THE_TOL);
        EXPECT_LE(aResult[i].parameter, THE_PI / 2.0 + THE_TOL);
    }
}

//==================================================================================================
// 3D offset curve tests (out of plane)
//==================================================================================================

#[test]
fn ExtremaPC_OffsetCurveTest__OffsetCircle_PointOutOfPlane() {
    // Create a circle of radius 10 in XY plane
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with offset 5
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    // Point above the plane
    let aPoint = Point3::new(15.0, 0.0, 10.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Closest point on offset curve is (15, 0, 0)
    // Distance = sqrt(0^2 + 0^2 + 10^2) = 10
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 10.0, THE_TOL);
}

//==================================================================================================
// Verify extremum condition tests
//==================================================================================================

#[test]
fn ExtremaPC_OffsetCurveTest__VerifyExtremumCondition() {
    // Create a circle of radius 10
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with offset 5
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    // Point on the X axis - simple case with known extremum
    let aPoint = Point3::new(25.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify that stored distance matches computed distance
    for i in 0..aResult.nb_ext() {
        let aStoredSqDist: f64 = aResult[i].square_distance;
        let aComputedSqDist: f64 = aPoint.square_distance(&aResult[i].point);
        EXPECT_NEAR(aStoredSqDist, aComputedSqDist, THE_TOL);
    }
}

#[test]
fn ExtremaPC_OffsetCurveTest__VerifyDistanceConsistency() {
    // Create a circle of radius 10
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    // Create offset curve with offset 5
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    let aPoint = Point3::new(18.0, 12.0, 3.0);

    let mut anEval = ExtremaPC::OffsetCurve::new(&anAdaptor).unwrap();
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Verify that stored distance matches computed distance
    for i in 0..aResult.nb_ext() {
        let aStoredSqDist: f64 = aResult[i].square_distance;
        let aComputedSqDist: f64 = aPoint.square_distance(&aResult[i].point);

        // "Distance mismatch at extremum " << i
        EXPECT_NEAR(aStoredSqDist, aComputedSqDist, THE_TOL);
    }
}

//=================================================================================================

#[test]
fn ExtremaPC_OffsetCurveTest__PlanarDelegation_PreservesNormalDirectionSign() {
    let aBasis = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 2.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            3.0,
        )
        .unwrap(),
    );
    let anOffset = Handle::new(
        GeomOffsetCurve::new(&aBasis.cast(), 1.0, dir3(0.0, 0.0, -1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffset.cast()).unwrap();
    let mut anEvaluator =
        ExtremaPC::OffsetCurve::with_domain(&anAdaptor, Domain1D::new(0.0, 2.0 * PI)).unwrap();
    let aQuery = Point3::new(8.0, 0.0, 7.0);
    let aResult = anEvaluator.perform(aQuery, THE_TOL, MM).unwrap();
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2);
    EXPECT_NEAR(aResult[aResult.min_index()].parameter, 0.0, THE_TOL);
    EXPECT_NEAR(
        aResult.min_square_distance(),
        aQuery.square_distance(&anOffset.value(0.0).unwrap()),
        THE_TOL,
    );
}

//=================================================================================================

#[test]
fn ExtremaPC_OffsetCurveTest__ScaledTransformedOffsetUsesParameterSafeFallback() {
    let aLine = Handle::new(GeomLine::from_point(Point3::default(), dir3(1.0, 0.0, 0.0)));
    let anOffset =
        Handle::new(GeomOffsetCurve::new(&aLine.cast(), 2.0, dir3(0.0, 0.0, 1.0), false).unwrap());
    let mut aScale = Transform3::default();
    aScale.set_scale(Point3::default(), 3.0).unwrap();
    let anAdaptor =
        GeomAdaptorTransformedCurve::with_range(&anOffset.cast(), -2.0, 2.0, aScale).unwrap();
    let mut anEvaluator =
        ExtremaPC::OffsetCurve::with_domain(&anAdaptor, Domain1D::new(-2.0, 2.0)).unwrap();

    let aQuery = anAdaptor
        .value(1.25)
        .unwrap()
        .translated(Vector3::new(0.0, 4.0, 0.0));
    let aResult = anEvaluator
        .perform(aQuery, THE_TOL, SearchMode::Min)
        .unwrap();
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 1.25, THE_TOL);
    let (aPoint, aDerivative): (Point3, Vector3) = anAdaptor.d1(aResult[0].parameter).unwrap();
    EXPECT_NEAR(
        Vector3::between(aQuery, aPoint).dot(aDerivative),
        0.0,
        THE_TOL,
    );
}
