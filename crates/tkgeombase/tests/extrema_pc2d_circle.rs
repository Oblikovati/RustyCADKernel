//! Direct translation of ExtremaPC2d_Circle_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, dir2};
use rustycad_tkernel::precision;
use rustycad_tkgeombase::extrema_pc2d::{self as ExtremaPC2d, Domain1D, SearchMode, Status};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis22d, Circle2, Point2, Vector2};
use std::f64::consts::{FRAC_PI_2, PI};

const THE_TOL: f64 = 1.0e-9;
const MM: SearchMode = SearchMode::MinMax;

fn makeCircle(theRadius: f64) -> Circle2 {
    Circle2::new(
        Axis22d::new(Point2::default(), dir2(1.0, 0.0), true).unwrap(),
        theRadius,
    )
    .unwrap()
}

fn circle(radius: f64, domain: (f64, f64)) -> ExtremaPC2d::Circle {
    ExtremaPC2d::Circle::with_domain(makeCircle(radius), Domain1D::new(domain.0, domain.1))
}

#[test]
fn ExtremaPC2d_CircleTest__OutsideAndInsidePointsHaveOppositeExtrema() {
    let mut anEvaluator = ExtremaPC2d::Circle::new(makeCircle(5.0));
    let anOutside = anEvaluator.perform(Point2::new(13.0, 0.0), THE_TOL, MM);
    assert_eq!(anOutside.nb_ext(), 2);
    EXPECT_NEAR(anOutside[anOutside.min_index()].parameter, 0.0, THE_TOL);
    EXPECT_NEAR(anOutside.min_square_distance(), 64.0, THE_TOL);
    EXPECT_NEAR(anOutside.max_square_distance(), 324.0, THE_TOL);

    let anInside = anEvaluator.perform(Point2::new(2.0, 0.0), THE_TOL, MM);
    EXPECT_NEAR(anInside.min_square_distance(), 9.0, THE_TOL);
    EXPECT_NEAR(anInside.max_square_distance(), 49.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_CircleTest__SearchModesReturnExactSubsets() {
    let mut anEvaluator = ExtremaPC2d::Circle::new(makeCircle(5.0));
    let aMin = anEvaluator.perform(Point2::new(13.0, 0.0), THE_TOL, SearchMode::Min);
    assert_eq!(aMin.nb_ext(), 1);
    assert!(aMin[0].is_minimum);
    let aMax = anEvaluator.perform(Point2::new(13.0, 0.0), THE_TOL, SearchMode::Max);
    assert_eq!(aMax.nb_ext(), 1);
    assert!(aMax[0].is_maximum);
    EXPECT_NEAR(aMax[0].parameter, PI, THE_TOL);
}

#[test]
fn ExtremaPC2d_CircleTest__IndirectOrientationPreservesSense() {
    let aPosition =
        Axis22d::with_directions(Point2::new(2.0, -1.0), dir2(0.0, 1.0), dir2(1.0, 0.0)).unwrap();
    let aCircle = Circle2::new(aPosition, 4.0).unwrap();
    let anAngle: f64 = 0.7;
    let aQuery: Point2 = aCircle.location().translated(
        (Vector2::from(aPosition.x_direction()) * anAngle.cos()
            + Vector2::from(aPosition.y_direction()) * anAngle.sin())
            * 7.0,
    );
    let mut anEvaluator = ExtremaPC2d::Circle::new(aCircle);
    let aResult = anEvaluator.perform(aQuery, THE_TOL, MM);
    EXPECT_NEAR(aResult[aResult.min_index()].parameter, anAngle, THE_TOL);
    EXPECT_NEAR(aResult.min_square_distance(), 9.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_CircleTest__CenterAndZeroRadiusHaveInfiniteSolutions() {
    let mut aCircle = ExtremaPC2d::Circle::new(makeCircle(5.0));
    let aCenter = aCircle.perform(Point2::default(), THE_TOL, MM);
    assert!(aCenter.is_infinite());
    EXPECT_NEAR(aCenter.infinite_square_distance, 25.0, THE_TOL);

    let mut aPointCircle = ExtremaPC2d::Circle::new(makeCircle(0.0));
    let aPointResult = aPointCircle.perform(Point2::new(3.0, 4.0), THE_TOL, MM);
    assert!(aPointResult.is_infinite());
    EXPECT_NEAR(aPointResult.infinite_square_distance, 25.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_CircleTest__ShiftedFullPeriodHasUniqueRepresentatives() {
    let mut anEvaluator = circle(5.0, (7.0, 7.0 + 2.0 * PI));
    let aResult = anEvaluator.perform_with_endpoints(Point2::new(13.0, 0.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 2);
    EXPECT_NEAR(aResult[aResult.min_index()].parameter, 4.0 * PI, THE_TOL);
    EXPECT_NEAR(aResult[aResult.max_index()].parameter, 3.0 * PI, THE_TOL);
}

#[test]
fn ExtremaPC2d_CircleTest__MultiplePeriodsReturnAllParameterExtrema() {
    let mut anEvaluator = circle(5.0, (0.0, 4.0 * PI));
    let aResult = anEvaluator.perform(Point2::new(13.0, 0.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 4);
    EXPECT_NEAR(aResult[0].parameter, 0.0, THE_TOL);
    EXPECT_NEAR(aResult[1].parameter, 2.0 * PI, THE_TOL);
    EXPECT_NEAR(aResult[2].parameter, PI, THE_TOL);
    EXPECT_NEAR(aResult[3].parameter, 3.0 * PI, THE_TOL);
}

#[test]
fn ExtremaPC2d_CircleTest__HalfCircleClassifiesInteriorAndEndpoints() {
    let mut anEvaluator = circle(5.0, (FRAC_PI_2, 3.0 * FRAC_PI_2));
    let aResult = anEvaluator.perform_with_endpoints(Point2::new(10.0, 0.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 3);
    EXPECT_NEAR(aResult[aResult.max_index()].parameter, PI, THE_TOL);
    let aMinimumCount = aResult.extrema.iter().filter(|e| e.is_minimum).count();
    assert_eq!(aMinimumCount, 2);
}

#[test]
fn ExtremaPC2d_CircleTest__SingletonAndInvalidInput() {
    let mut aSingleton = circle(5.0, (0.4, 0.4));
    assert_eq!(
        aSingleton
            .perform(Point2::new(8.0, 0.0), THE_TOL, MM)
            .nb_ext(),
        0
    );
    assert_eq!(
        aSingleton.perform(Point2::default(), THE_TOL, MM).status,
        Status::NoSolution
    );
    assert_eq!(
        aSingleton
            .perform_with_endpoints(Point2::new(8.0, 0.0), THE_TOL, MM)
            .nb_ext(),
        1
    );
    let mut anInvalid = circle(5.0, (1.0, 0.0));
    assert_eq!(
        anInvalid.perform(Point2::default(), THE_TOL, MM).status,
        Status::InvalidInput
    );
    assert_eq!(
        anInvalid.perform(Point2::default(), 0.0, MM).status,
        Status::InvalidInput
    );
}

#[test]
fn ExtremaPC2d_CircleTest__LargeToleranceDoesNotExpandDomain() {
    let mut anEvaluator = circle(1.0, (1.0, 2.0));
    let aResult = anEvaluator.perform(Point2::new(2.0, 0.0), 2.0, MM);
    assert_eq!(aResult.status, Status::NoSolution);
    assert_eq!(aResult.nb_ext(), 0);
}

#[test]
fn ExtremaPC2d_CircleTest__NonFiniteQueryIsRejected() {
    let mut anEvaluator = ExtremaPC2d::Circle::new(makeCircle(5.0));
    let aQuery = Point2::new(precision::infinite(), 0.0);
    assert_eq!(
        anEvaluator.perform(aQuery, THE_TOL, MM).status,
        Status::InvalidInput
    );
}

#[test]
fn ExtremaPC2d_CircleTest__UnrepresentablePeriodicStepReportsNumericalError() {
    let aFirst: f64 = 1.0e17;
    let mut anEvaluator = circle(5.0, (aFirst, aFirst + 100.0));
    assert_eq!(
        anEvaluator
            .perform(Point2::new(12.0, 0.0), THE_TOL, MM)
            .status,
        Status::NumericalError
    );
}

#[test]
fn ExtremaPC2d_CircleTest__AccessorsAndValueMatchGeometry() {
    let aCircle = Circle2::new(
        Axis22d::new(Point2::new(2.0, 3.0), dir2(0.0, 1.0), false).unwrap(),
        4.0,
    )
    .unwrap();
    let anEvaluator = ExtremaPC2d::Circle::with_domain(aCircle, Domain1D::new(-1.0, 2.0));
    assert!(anEvaluator.is_bounded());
    EXPECT_NEAR(anEvaluator.domain().unwrap().min, -1.0, THE_TOL);
    let expected: Point2 = elclib::value(0.3, &aCircle);
    assert_eq!(anEvaluator.value(0.3).distance(expected), 0.0);
}
