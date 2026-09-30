//! Direct translation of ExtremaPC_Curve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
// The result tests fill default-built results field by field, in the native order.
#![allow(non_snake_case, dead_code, clippy::field_reassign_with_default)]
mod common;
#[allow(unused_imports)]
use common::*;
#[allow(unused_imports)]
use prelude::*;

// Test fixture for ExtremaPC tests.
const THE_TOL: f64 = 1.0e-6;

//==================================================================================================
// Line tests
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__Line_PointOnLine() {
    // Line along X axis with bounded domain
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));

    // Point on line
    let aPoint = Point3::new(5.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(0.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max (endpoints)

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 0.0, THE_TOL);
    assert!(aResult[aMinIdx].is_minimum);
}

#[test]
fn ExtremaPC_CurveTest__Line_PointOffLine() {
    // Line along X axis with bounded domain
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));

    // Point off line
    let aPoint = Point3::new(5.0, 3.0, 4.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(0.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max (endpoints)

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 25.0, THE_TOL); // 3^2 + 4^2 = 25
    assert!(aResult[aMinIdx].is_minimum);
}

#[test]
fn ExtremaPC_CurveTest__Line_ProjectionOutsideBounds() {
    // Line along X axis with bounded domain
    let aLine = Line3::new(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0));

    // Point that projects outside bounds
    let aPoint = Point3::new(-50.0, 3.0, 4.0);

    let mut anEval = ExtremaPC::Line::with_domain(aLine, Domain1D::new(0.0, 100.0));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2); // min at 0, max at 100

    let aMinIdx: usize = aResult.min_index();
    // Should be at endpoint U=0
    EXPECT_NEAR(aResult[aMinIdx].parameter, 0.0, THE_TOL);
}

//==================================================================================================
// Circle tests
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__Circle_PointOutside() {
    // Circle with radius 10 in XY plane
    let aCircle = Circle3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        10.0,
    )
    .unwrap();

    // Point outside circle
    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Circle::with_domain(aCircle, Domain1D::new(0.0, 2.0 * PI));
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2);

    // Find min and max
    let aMinIdx: usize = if aResult[0].square_distance < aResult[1].square_distance {
        0
    } else {
        1
    };
    let aMaxIdx: usize = 1 - aMinIdx;

    // Minimum should be at (10, 0, 0), distance = 10
    EXPECT_NEAR((aResult[aMinIdx].square_distance).sqrt(), 10.0, THE_TOL);
    assert!(aResult[aMinIdx].is_minimum);

    // Maximum should be at (-10, 0, 0), distance = 30
    EXPECT_NEAR((aResult[aMaxIdx].square_distance).sqrt(), 30.0, THE_TOL);
    assert!(!(aResult[aMaxIdx].is_minimum));
}

#[test]
fn ExtremaPC_CurveTest__Circle_PointInside() {
    // Circle with radius 10 in XY plane
    let aCircle = Circle3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        10.0,
    )
    .unwrap();

    // Point inside circle
    let aPoint = Point3::new(3.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Circle::new(aCircle);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2);

    // Find min distance
    let aMinDist: f64 =
        ((aResult[0].square_distance).sqrt()).min((aResult[1].square_distance).sqrt());

    // Minimum distance should be 10 - 3 = 7
    EXPECT_NEAR(aMinDist, 7.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__Circle_PointAtCenter_Degenerate() {
    // Circle with radius 10 in XY plane
    let aCircle = Circle3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        10.0,
    )
    .unwrap();

    // Point at center - degenerate case
    let aPoint = Point3::new(0.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Circle::new(aCircle);
    let aResult = anEval.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    // Should return infinite solutions
    assert!(aResult.is_infinite());
    EXPECT_NEAR((aResult.infinite_square_distance).sqrt(), 10.0, THE_TOL);
}

//==================================================================================================
// Aggregator tests with GeomAdaptor_Curve
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__Aggregator_Line() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomLine.cast(), -100.0, 100.0).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    let aPoint = Point3::new(5.0, 3.0, 4.0);
    let aResult = anExtPC.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 3); // 1 min + 2 max (endpoints)

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 25.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_Circle() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&aGeomCircle.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    let aPoint = Point3::new(20.0, 0.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Check minimum distance
    let aMinSqDist: f64 = aResult.min_square_distance();
    EXPECT_NEAR((aMinSqDist).sqrt(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_BSpline() {
    // Create a simple cubic BSpline
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(2.0, 2.0, 0.0);
    aPoles[4] = Point3::new(3.0, 0.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 4;
    aMults[2] = 4;

    let aBSpline = Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap());
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    // Point near the curve
    let aPoint = Point3::new(1.5, 3.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    // Should find at least one extremum
    EXPECT_GE(aResult.nb_ext(), 0);
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_Ellipse() {
    let aGeomEllipse = Handle::new(
        GeomEllipse::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            20.0,
            10.0,
        )
        .unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&aGeomEllipse.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    // Point on major axis
    let aPoint = Point3::new(30.0, 0.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Minimum distance should be 30 - 20 = 10
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_Parabola() {
    // y^2 = 4*p*x with p = 2, so y^2 = 8x
    let aGeomParabola = Handle::new(
        GeomParabola::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            2.0,
        )
        .unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomParabola.cast(), -10.0, 10.0).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    // Point on X axis at focus
    let aPoint = Point3::new(2.0, 0.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Focus is at (p, 0) = (2, 0), vertex is at origin
    // Minimum distance should be 2.0
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 2.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_Hyperbola() {
    // Hyperbola with major radius 10, minor radius 5
    // x = a*cosh(u), y = b*sinh(u)
    let aGeomHyperbola = Handle::new(
        GeomHyperbola::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
            5.0,
        )
        .unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomHyperbola.cast(), -2.0, 2.0).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    // Point near the vertex
    let aPoint = Point3::new(10.0, 5.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

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
fn ExtremaPC_CurveTest__Aggregator_Bezier() {
    // Create a cubic Bezier curve
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 3.0, 0.0);
    aPoles[3] = Point3::new(3.0, 3.0, 0.0);
    aPoles[4] = Point3::new(4.0, 0.0, 0.0);

    let aBezier = Handle::new(GeomBezierCurve::new(&aPoles).unwrap());
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBezier.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    // Point above the curve's peak
    let aPoint = Point3::new(2.0, 5.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

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
fn ExtremaPC_CurveTest__Aggregator_OffsetCurve() {
    // Create a circle and its offset
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    // Point outside the offset circle
    let aPoint = Point3::new(25.0, 0.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Offset circle has radius 15, distance should be 25 - 15 = 10
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_TrimmedCurve() {
    // Create a trimmed circle (quarter arc)
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );
    let aTrimmedCurve =
        Handle::new(GeomTrimmedCurve::new(&aCircle.cast(), 0.0, PI / 2.0, true, true).unwrap());
    let anAdaptor = GeomAdaptorCurve::with_curve(&aTrimmedCurve.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    // Point in first quadrant
    let aPoint = Point3::new(15.0, 15.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Check that all extrema are within bounds
    for i in 0..aResult.nb_ext() {
        EXPECT_GE(aResult[i].parameter, -THE_TOL);
        EXPECT_LE(aResult[i].parameter, PI / 2.0 + THE_TOL);
    }
}

//==================================================================================================
// Edge case tests
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__Aggregator_PointOnCurve() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&aGeomCircle.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    // Point exactly on the circle
    let aPoint = Point3::new(10.0, 0.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    // Distance should be ~0
    let aMinDist: f64 = (aResult.min_square_distance()).sqrt();
    EXPECT_NEAR(aMinDist, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_IsInitialized() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomLine.cast(), 0.0, 100.0).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    assert!(anExtPC.is_initialized());

    let aPoint = Point3::new(50.0, 5.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 50.0, THE_TOL);
}

//==================================================================================================
// Result utility tests
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__Result_MinSquareDistance() {
    let mut aResult = ExtremaPC::ExtremaResult::default();
    aResult.status = Status::OK;

    // Add some extrema
    let mut anExt1 = ExtremaPC::ExtremumResult::default();
    anExt1.square_distance = 100.0;
    aResult.extrema.push(anExt1);

    let mut anExt2 = ExtremaPC::ExtremumResult::default();
    anExt2.square_distance = 50.0;
    aResult.extrema.push(anExt2);

    let mut anExt3 = ExtremaPC::ExtremumResult::default();
    anExt3.square_distance = 75.0;
    aResult.extrema.push(anExt3);

    EXPECT_NEAR(aResult.min_square_distance(), 50.0, THE_TOL);
    assert_eq!(aResult.min_index(), 1);
}

#[test]
fn ExtremaPC_CurveTest__Result_Empty() {
    let aResult = ExtremaPC::ExtremaResult::default();

    assert!(!(aResult.is_done()));
    assert_eq!(aResult.nb_ext(), 0);
    assert_eq!(aResult.min_index(), ExtremaPC::ExtremaResult::INVALID_INDEX);
    assert!((aResult.min_square_distance()).is_infinite());
}

//==================================================================================================
// SearchMode tests
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__Circle_SearchMode_MinMax() {
    // Circle with radius 10 in XY plane
    let aCircle = Circle3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        10.0,
    )
    .unwrap();

    // Point outside circle
    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Circle::new(aCircle);
    let aResult = anEval
        .perform_with_endpoints(aPoint, THE_TOL, SearchMode::MinMax)
        .unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 2); // Both min and max
}

#[test]
fn ExtremaPC_CurveTest__Circle_SearchMode_Min() {
    // Circle with radius 10 in XY plane
    let aCircle = Circle3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        10.0,
    )
    .unwrap();

    // Point outside circle
    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Circle::new(aCircle);
    let aResult = anEval
        .perform_with_endpoints(aPoint, THE_TOL, SearchMode::Min)
        .unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1); // Only min
    assert!(aResult[0].is_minimum);
    EXPECT_NEAR((aResult[0].square_distance).sqrt(), 10.0, THE_TOL); // 20 - 10 = 10
}

#[test]
fn ExtremaPC_CurveTest__Circle_SearchMode_Max() {
    // Circle with radius 10 in XY plane
    let aCircle = Circle3::new(
        Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
        10.0,
    )
    .unwrap();

    // Point outside circle
    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anEval = ExtremaPC::Circle::new(aCircle);
    let aResult = anEval
        .perform_with_endpoints(aPoint, THE_TOL, SearchMode::Max)
        .unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1); // Only max
    assert!(!(aResult[0].is_minimum));
    EXPECT_NEAR((aResult[0].square_distance).sqrt(), 30.0, THE_TOL); // 20 + 10 = 30
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_SearchMode_Min() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&aGeomCircle.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let aPoint = Point3::new(20.0, 0.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1); // Only min
    assert!(aResult[0].is_minimum);
    EXPECT_NEAR((aResult[0].square_distance).sqrt(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_SearchMode_Max() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );
    let anAdaptor = GeomAdaptorCurve::with_curve(&aGeomCircle.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let aPoint = Point3::new(20.0, 0.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1); // Only max
    assert!(!(aResult[0].is_minimum));
    EXPECT_NEAR((aResult[0].square_distance).sqrt(), 30.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__Aggregator_BSpline_SearchMode_Min() {
    // Create a simple cubic BSpline
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(2.0, 2.0, 0.0);
    aPoles[4] = Point3::new(3.0, 0.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 4;
    aMults[2] = 4;

    let aBSpline = Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap());
    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    // Point near the curve
    let aPoint = Point3::new(1.5, 3.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    assert!(aResult.is_done());
    // In Min mode, should only return minimum extrema
    for i in 0..aResult.nb_ext() {
        assert!(aResult[i].is_minimum);
    }
}

#[test]
fn ExtremaPC_CurveTest__Result_MaxSquareDistance() {
    let mut aResult = ExtremaPC::ExtremaResult::default();
    aResult.status = Status::OK;

    // Add some extrema
    let mut anExt1 = ExtremaPC::ExtremumResult::default();
    anExt1.square_distance = 100.0;
    anExt1.is_minimum = false;
    aResult.extrema.push(anExt1);

    let mut anExt2 = ExtremaPC::ExtremumResult::default();
    anExt2.square_distance = 50.0;
    anExt2.is_minimum = true;
    aResult.extrema.push(anExt2);

    let mut anExt3 = ExtremaPC::ExtremumResult::default();
    anExt3.square_distance = 75.0;
    anExt3.is_minimum = false;
    aResult.extrema.push(anExt3);

    EXPECT_NEAR(aResult.min_square_distance(), 50.0, THE_TOL);
    assert_eq!(aResult.min_index(), 1);
    EXPECT_NEAR(aResult.max_square_distance(), 100.0, THE_TOL);
    assert_eq!(aResult.max_index(), 0);
}

//==================================================================================================
// GeomAdaptor_TransformedCurve tests - helpers
//==================================================================================================

// Helper: create a cubic BSpline for reuse across tests.
fn MakeTestBSpline() -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(2.0, 2.0, 0.0);
    aPoles[4] = Point3::new(3.0, 0.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 4;
    aMults[2] = 4;

    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap())
}

// Helper: create a cubic Bezier for reuse across tests.
fn MakeTestBezier() -> Handle<GeomBezierCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 3.0, 0.0);
    aPoles[3] = Point3::new(3.0, 3.0, 0.0);
    aPoles[4] = Point3::new(4.0, 0.0, 0.0);

    Handle::new(GeomBezierCurve::new(&aPoles).unwrap())
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - Identity transform
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Identity_LineMatchesNonTransformed() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));

    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomLine.cast(), -100.0, 100.0).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let anIdentity = Transform3::default();
    let aTransformed =
        GeomAdaptorTransformedCurve::with_range(&aGeomLine.cast(), -100.0, 100.0, anIdentity)
            .unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    assert!(anExtPC.is_initialized());
    assert!(anExtPCTr.is_initialized());

    let aPoint = Point3::new(5.0, 3.0, 4.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();
    let aResultTr = anExtPCTr.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());
    assert_eq!(aResult.nb_ext(), aResultTr.nb_ext());
    EXPECT_NEAR(
        aResult.min_square_distance(),
        aResultTr.min_square_distance(),
        THE_TOL,
    );
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Identity_BSplineMatchesNonTransformed() {
    let aBSpline = MakeTestBSpline();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let anIdentity = Transform3::default();
    let aTransformed =
        GeomAdaptorTransformedCurve::with_curve(&aBSpline.cast(), anIdentity).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aPoint = Point3::new(1.5, 3.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();
    let aResultTr = anExtPCTr.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());
    EXPECT_NEAR(
        aResult.min_square_distance(),
        aResultTr.min_square_distance(),
        THE_TOL,
    );
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - Translation
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Translation_Line() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(0.0, 10.0, 0.0));
    let aTransformed =
        GeomAdaptorTransformedCurve::with_range(&aGeomLine.cast(), -100.0, 100.0, aTrsf).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();
    assert!(anExtPC.is_initialized());

    // Point at (5, 13, 4): closest to translated line at (5, 10, 0), distance = sqrt(9+16) = 5
    let aPoint = Point3::new(5.0, 13.0, 4.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 25.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.y(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Translation_Circle() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(100.0, 0.0, 0.0));
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aGeomCircle.cast(), aTrsf).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    // Circle center at (100,0,0), closest at (110,0,0), dist = 10
    let aPoint = Point3::new(120.0, 0.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);
    EXPECT_NEAR((aResult.min_square_distance()).sqrt(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Translation_BSpline() {
    let aBSpline = MakeTestBSpline();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(10.0, 20.0, 30.0));
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aBSpline.cast(), aTrsf).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aOrigPoint = Point3::new(1.5, 3.0, 0.0);
    let aTransPoint = Point3::new(11.5, 23.0, 30.0);

    let aResult = anExtPC.perform(aOrigPoint, THE_TOL, MM).unwrap();
    let aResultTr = anExtPCTr.perform(aTransPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());
    assert_eq!(aResult.nb_ext(), aResultTr.nb_ext());
    EXPECT_NEAR(
        aResult.min_square_distance(),
        aResultTr.min_square_distance(),
        THE_TOL,
    );

    let aMinIdx: usize = aResult.min_index();
    let aMinIdxTr: usize = aResultTr.min_index();
    EXPECT_NEAR(
        aResult[aMinIdx].parameter,
        aResultTr[aMinIdxTr].parameter,
        THE_TOL,
    );
    EXPECT_NEAR(
        aResultTr[aMinIdxTr].point.x(),
        aResult[aMinIdx].point.x() + 10.0,
        THE_TOL,
    );
    EXPECT_NEAR(
        aResultTr[aMinIdxTr].point.y(),
        aResult[aMinIdx].point.y() + 20.0,
        THE_TOL,
    );
    EXPECT_NEAR(
        aResultTr[aMinIdxTr].point.z(),
        aResult[aMinIdx].point.z() + 30.0,
        THE_TOL,
    );
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - Rotation
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Rotation_Line() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));

    // Rotate 90 degrees around Z -> line becomes along Y
    let mut aTrsf = Transform3::default();
    aTrsf
        .set_rotation(
            Axis1::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)),
            PI / 2.0,
        )
        .unwrap();
    let aTransformed =
        GeomAdaptorTransformedCurve::with_range(&aGeomLine.cast(), 0.0, 100.0, aTrsf).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    // Point at (3, 5, 0): rotated line along Y, closest at (0, 5, 0), dist = 3
    let aPoint = Point3::new(3.0, 5.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR((aResult[aMinIdx].square_distance).sqrt(), 3.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.x(), 0.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.y(), 5.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Rotation_BSpline() {
    let aBSpline = MakeTestBSpline();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let mut aTrsf = Transform3::default();
    aTrsf
        .set_rotation(
            Axis1::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)),
            PI / 2.0,
        )
        .unwrap();
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aBSpline.cast(), aTrsf).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aOrigPoint = Point3::new(1.5, 3.0, 0.0);
    let aTransPoint = Point3::new(-3.0, 1.5, 0.0);

    let aResult = anExtPC.perform(aOrigPoint, THE_TOL, MM).unwrap();
    let aResultTr = anExtPCTr.perform(aTransPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());
    EXPECT_NEAR(
        aResult.min_square_distance(),
        aResultTr.min_square_distance(),
        THE_TOL,
    );

    let aMinIdx: usize = aResult.min_index();
    let aMinIdxTr: usize = aResultTr.min_index();
    EXPECT_NEAR(
        aResult[aMinIdx].parameter,
        aResultTr[aMinIdxTr].parameter,
        THE_TOL,
    );
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - Compound transform (rotation + translation)
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Compound_BSpline() {
    let aBSpline = MakeTestBSpline();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let mut aTrsf = Transform3::default();
    aTrsf
        .set_rotation(
            Axis1::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)),
            PI / 4.0,
        )
        .unwrap();
    let mut aTransl = Transform3::default();
    aTransl.set_translation(Vector3::new(5.0, 5.0, 0.0));
    aTrsf.multiply(aTransl);

    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aBSpline.cast(), aTrsf).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aOrigPoint = Point3::new(1.5, 3.0, 0.0);
    let aTransPoint = aOrigPoint.transformed(aTrsf);

    let aResult = anExtPC.perform(aOrigPoint, THE_TOL, MM).unwrap();
    let aResultTr = anExtPCTr.perform(aTransPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());
    EXPECT_NEAR(
        aResult.min_square_distance(),
        aResultTr.min_square_distance(),
        THE_TOL,
    );

    let aMinIdx: usize = aResult.min_index();
    let aMinIdxTr: usize = aResultTr.min_index();
    EXPECT_NEAR(
        aResult[aMinIdx].parameter,
        aResultTr[aMinIdxTr].parameter,
        THE_TOL,
    );

    let aExpectedPt = aResult[aMinIdx].point.transformed(aTrsf);
    EXPECT_NEAR(aResultTr[aMinIdxTr].point.x(), aExpectedPt.x(), THE_TOL);
    EXPECT_NEAR(aResultTr[aMinIdxTr].point.y(), aExpectedPt.y(), THE_TOL);
    EXPECT_NEAR(aResultTr[aMinIdxTr].point.z(), aExpectedPt.z(), THE_TOL);
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - Uniform scale
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Scale_BSpline() {
    let aBSpline = MakeTestBSpline();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let mut aTrsf = Transform3::default();
    aTrsf.set_scale(Point3::new(0.0, 0.0, 0.0), 2.0).unwrap();
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aBSpline.cast(), aTrsf).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aOrigPoint = Point3::new(1.5, 3.0, 0.0);
    let aScaledPoint = Point3::new(3.0, 6.0, 0.0);

    let aResult = anExtPC.perform(aOrigPoint, THE_TOL, MM).unwrap();
    let aResultTr = anExtPCTr.perform(aScaledPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());

    // Distances scale by s^2 = 4
    EXPECT_NEAR(
        aResultTr.min_square_distance(),
        aResult.min_square_distance() * 4.0,
        THE_TOL,
    );

    let aMinIdx: usize = aResult.min_index();
    let aMinIdxTr: usize = aResultTr.min_index();
    EXPECT_NEAR(
        aResult[aMinIdx].parameter,
        aResultTr[aMinIdxTr].parameter,
        THE_TOL,
    );
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Scale_Bezier() {
    let aBezier = MakeTestBezier();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBezier.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let mut aTrsf = Transform3::default();
    aTrsf.set_scale(Point3::new(0.0, 0.0, 0.0), 3.0).unwrap();
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aBezier.cast(), aTrsf).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aOrigPoint = Point3::new(2.0, 5.0, 0.0);
    let aScaledPoint = Point3::new(6.0, 15.0, 0.0);

    let aResult = anExtPC.perform(aOrigPoint, THE_TOL, MM).unwrap();
    let aResultTr = anExtPCTr.perform(aScaledPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());

    // Distances scale by s^2 = 9
    EXPECT_NEAR(
        aResultTr.min_square_distance(),
        aResult.min_square_distance() * 9.0,
        1.0e-4,
    );
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_CompoundScale_LinePreservesParameter() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));

    let mut aTrsf = Transform3::default();
    aTrsf.set_scale(Point3::new(0.0, 0.0, 0.0), 2.0).unwrap();
    let mut aRotation = Transform3::default();
    aRotation
        .set_rotation(
            Axis1::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)),
            PI / 2.0,
        )
        .unwrap();
    aTrsf.pre_multiply(aRotation);
    let mut aTranslation = Transform3::default();
    aTranslation.set_translation(Vector3::new(5.0, 7.0, 0.0));
    aTrsf.pre_multiply(aTranslation);

    let aTransformed =
        GeomAdaptorTransformedCurve::with_range(&aLine.cast(), 0.0, 10.0, aTrsf).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();
    let aQuery = Point3::new(4.0, 7.0, 0.0).transformed(aTrsf);
    let aResult = anExtPC.perform(aQuery, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 4.0, THE_TOL);
    EXPECT_NEAR(
        aResult[0]
            .point
            .distance(&aTransformed.value(aResult[0].parameter).unwrap()),
        0.0,
        THE_TOL,
    );
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_CompoundScale_ParabolaPreservesParameter() {
    let aParabola = Handle::new(
        GeomParabola::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            5.0,
        )
        .unwrap(),
    );

    let mut aTrsf = Transform3::default();
    aTrsf.set_scale(Point3::new(0.0, 0.0, 0.0), 3.0).unwrap();
    let mut aTranslation = Transform3::default();
    aTranslation.set_translation(Vector3::new(10.0, -4.0, 2.0));
    aTrsf.pre_multiply(aTranslation);

    let aTransformed =
        GeomAdaptorTransformedCurve::with_range(&aParabola.cast(), -10.0, 10.0, aTrsf).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();
    let aParameter: f64 = 3.0;
    let aQuery = aTransformed.value(aParameter).unwrap();
    let aResult = anExtPC.perform(aQuery, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);
    let aMinIndex: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIndex].parameter, aParameter, THE_TOL);
    EXPECT_NEAR(aResult[aMinIndex].point.distance(&aQuery), 0.0, THE_TOL);
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - Offset curve via Adaptor3d_Curve interface
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Translation_OffsetCurve() {
    let aCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(50.0, 0.0, 0.0));
    let aTransformed =
        GeomAdaptorTransformedCurve::with_curve(&anOffsetCurve.cast(), aTrsf).unwrap();

    let mut anExtPC = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();
    assert!(anExtPC.is_initialized());

    // Offset circle radius 15, center (50, 0, 0), point at (75, 0, 0): dist = 10
    let aPoint = Point3::new(75.0, 0.0, 0.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);
    EXPECT_NEAR((aResult.min_square_distance()).sqrt(), 10.0, THE_TOL);
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - Domain-limited
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_DomainLimited_Circle() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(0.0, 0.0, 50.0));
    let aTransformed =
        GeomAdaptorTransformedCurve::with_range(&aGeomCircle.cast(), 0.0, PI / 2.0, aTrsf).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aPoint = Point3::new(15.0, 15.0, 50.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    for i in 0..aResult.nb_ext() {
        EXPECT_GE(aResult[i].parameter, -THE_TOL);
        EXPECT_LE(aResult[i].parameter, PI / 2.0 + THE_TOL);
        EXPECT_NEAR(aResult[i].point.z(), 50.0, THE_TOL);
    }
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_DomainLimited_Constructor() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(0.0, 5.0, 0.0));
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aGeomLine.cast(), aTrsf).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_transformed_range(&aTransformed, 0.0, 10.0).unwrap();

    let aPoint = Point3::new(5.0, 8.0, 0.0);
    let aResult = anExtPC.perform_with_endpoints(aPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    EXPECT_GE(aResult.nb_ext(), 1);

    let aMinIdx: usize = aResult.min_index();
    EXPECT_NEAR(aResult[aMinIdx].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].square_distance, 9.0, THE_TOL);
    EXPECT_NEAR(aResult[aMinIdx].point.y(), 5.0, THE_TOL);
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - PerformWithEndpoints
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_PerformWithEndpoints_BSpline() {
    let aBSpline = MakeTestBSpline();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(0.0, 0.0, 10.0));
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aBSpline.cast(), aTrsf).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aOrigPoint = Point3::new(1.5, 3.0, 0.0);
    let aTransPoint = Point3::new(1.5, 3.0, 10.0);

    let aResult = anExtPC
        .perform_with_endpoints(aOrigPoint, THE_TOL, MM)
        .unwrap();
    let aResultTr = anExtPCTr
        .perform_with_endpoints(aTransPoint, THE_TOL, MM)
        .unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());
    assert_eq!(aResult.nb_ext(), aResultTr.nb_ext());
    EXPECT_NEAR(
        aResult.min_square_distance(),
        aResultTr.min_square_distance(),
        THE_TOL,
    );
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_PerformWithEndpoints_ScaledBSpline() {
    let aBSpline = MakeTestBSpline();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let mut aTrsf = Transform3::default();
    aTrsf.set_scale(Point3::new(0.0, 0.0, 0.0), 2.0).unwrap();
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aBSpline.cast(), aTrsf).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aOrigPoint = Point3::new(1.5, 3.0, 0.0);
    let aScaledPoint = Point3::new(3.0, 6.0, 0.0);

    let aResult = anExtPC
        .perform_with_endpoints(aOrigPoint, THE_TOL, MM)
        .unwrap();
    let aResultTr = anExtPCTr
        .perform_with_endpoints(aScaledPoint, THE_TOL, MM)
        .unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());
    EXPECT_NEAR(
        aResultTr.min_square_distance(),
        aResult.min_square_distance() * 4.0,
        THE_TOL,
    );
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - Bezier with transform
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Translation_Bezier() {
    let aBezier = MakeTestBezier();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBezier.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(10.0, 20.0, 30.0));
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aBezier.cast(), aTrsf).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aOrigPoint = Point3::new(2.0, 5.0, 0.0);
    let aTransPoint = Point3::new(12.0, 25.0, 30.0);

    let aResult = anExtPC.perform(aOrigPoint, THE_TOL, MM).unwrap();
    let aResultTr = anExtPCTr.perform(aTransPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());
    EXPECT_NEAR(
        aResult.min_square_distance(),
        aResultTr.min_square_distance(),
        THE_TOL,
    );

    let aMinIdx: usize = aResult.min_index();
    let aMinIdxTr: usize = aResultTr.min_index();
    let aExpectedPt = aResult[aMinIdx].point.transformed(aTrsf);
    EXPECT_NEAR(aResultTr[aMinIdxTr].point.x(), aExpectedPt.x(), THE_TOL);
    EXPECT_NEAR(aResultTr[aMinIdxTr].point.y(), aExpectedPt.y(), THE_TOL);
    EXPECT_NEAR(aResultTr[aMinIdxTr].point.z(), aExpectedPt.z(), THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_Compound_Bezier() {
    let aBezier = MakeTestBezier();

    let anAdaptor = GeomAdaptorCurve::with_curve(&aBezier.cast()).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();

    let mut aTrsf = Transform3::default();
    aTrsf
        .set_rotation(
            Axis1::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)),
            PI / 3.0,
        )
        .unwrap();
    let mut aTransl = Transform3::default();
    aTransl.set_translation(Vector3::new(10.0, 0.0, 0.0));
    aTrsf.multiply(aTransl);

    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aBezier.cast(), aTrsf).unwrap();
    let mut anExtPCTr = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aOrigPoint = Point3::new(2.0, 5.0, 0.0);
    let aTransPoint = aOrigPoint.transformed(aTrsf);

    let aResult = anExtPC.perform(aOrigPoint, THE_TOL, MM).unwrap();
    let aResultTr = anExtPCTr.perform(aTransPoint, THE_TOL, MM).unwrap();

    assert!(aResult.is_done());
    assert!(aResultTr.is_done());
    EXPECT_NEAR(
        aResult.min_square_distance(),
        aResultTr.min_square_distance(),
        THE_TOL,
    );
}

//==================================================================================================
// GeomAdaptor_TransformedCurve - SearchMode with transform
//==================================================================================================

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_SearchMode_Min() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(0.0, 0.0, 5.0));
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aGeomCircle.cast(), aTrsf).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aPoint = Point3::new(20.0, 0.0, 5.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    assert!(aResult[0].is_minimum);
    EXPECT_NEAR((aResult[0].square_distance).sqrt(), 10.0, THE_TOL);
}

#[test]
fn ExtremaPC_CurveTest__TransformedCurve_SearchMode_Max() {
    let aGeomCircle = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            10.0,
        )
        .unwrap(),
    );

    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(0.0, 0.0, 5.0));
    let aTransformed = GeomAdaptorTransformedCurve::with_curve(&aGeomCircle.cast(), aTrsf).unwrap();
    let mut anExtPC = ExtremaPC::Curve::from_transformed(&aTransformed).unwrap();

    let aPoint = Point3::new(20.0, 0.0, 5.0);
    let aResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    assert!(!(aResult[0].is_minimum));
    EXPECT_NEAR((aResult[0].square_distance).sqrt(), 30.0, THE_TOL);
}
