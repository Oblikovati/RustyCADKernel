//! Direct translation of ExtremaPC2d_BezierCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_GT, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkgeombase::extrema_pc2d::{self as ExtremaPC2d, Domain1D, SearchMode, Status};
use rustycad_tkmath::gp::Point2;
use std::f64::consts::FRAC_1_SQRT_2;

const THE_TOL: f64 = 1.0e-8;
const MM: SearchMode = SearchMode::MinMax;

fn poles(points: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut aPoles = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, (x, y)) in points.iter().enumerate() {
        aPoles[i as i32 + 1] = Point2::new(*x, *y);
    }
    aPoles
}

fn makeLinear() -> Handle<Geom2dBezierCurve> {
    Handle::new(Geom2dBezierCurve::new(&poles(&[(0.0, 0.0), (10.0, 0.0)])).unwrap())
}

fn makeQuadratic() -> Handle<Geom2dBezierCurve> {
    let aPoles = poles(&[(-2.0, 0.0), (0.0, 2.0), (2.0, 0.0)]);
    Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap())
}

#[test]
fn ExtremaPC2d_BezierCurveTest__LinearProjectionIsExact() {
    let mut anEvaluator = ExtremaPC2d::BezierCurve::new(makeLinear());
    let aResult = anEvaluator
        .perform(Point2::new(4.0, 3.0), THE_TOL, MM)
        .unwrap();
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.4, THE_TOL);
    EXPECT_NEAR(aResult[0].square_distance, 9.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_BezierCurveTest__SearchModesAndEndpoints() {
    let mut anEvaluator = ExtremaPC2d::BezierCurve::new(makeLinear());
    let aMaxOnly = anEvaluator.perform(Point2::new(4.0, 3.0), THE_TOL, SearchMode::Max);
    assert_eq!(aMaxOnly.unwrap().nb_ext(), 0);
    assert_eq!(
        anEvaluator
            .perform(Point2::new(-2.0, 0.0), THE_TOL, MM)
            .unwrap()
            .nb_ext(),
        0
    );
    let aMin = anEvaluator
        .perform_with_endpoints(Point2::new(-2.0, 0.0), THE_TOL, SearchMode::Min)
        .unwrap();
    assert_eq!(aMin.nb_ext(), 1);
    EXPECT_NEAR(aMin[0].parameter, 0.0, THE_TOL);
    let aMax = anEvaluator
        .perform_with_endpoints(Point2::new(-2.0, 0.0), THE_TOL, SearchMode::Max)
        .unwrap();
    assert_eq!(aMax.nb_ext(), 1);
    EXPECT_NEAR(aMax[0].parameter, 1.0, THE_TOL);
    EXPECT_NEAR(aMax[0].square_distance, 144.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_BezierCurveTest__SymmetricQuadraticHasCentralMinimum() {
    let mut anEvaluator = ExtremaPC2d::BezierCurve::new(makeQuadratic());
    let aResult = anEvaluator
        .perform(Point2::default(), THE_TOL, SearchMode::Min)
        .unwrap();
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.5, THE_TOL);
    EXPECT_NEAR(aResult[0].square_distance, 1.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_BezierCurveTest__RationalArcHasConstantDistance() {
    let aPoles = poles(&[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    aWeights[1] = 1.0;
    aWeights[2] = FRAC_1_SQRT_2;
    aWeights[3] = 1.0;
    let aCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let mut anEvaluator = ExtremaPC2d::BezierCurve::new(aCurve);
    let aResult = anEvaluator.perform(Point2::default(), THE_TOL, MM).unwrap();
    assert!(aResult.is_infinite());
    EXPECT_NEAR(aResult.infinite_square_distance, 1.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_BezierCurveTest__DegenerateCurveHasInfiniteSolutions() {
    let aPoles = poles(&[(2.0, -1.0), (2.0, -1.0), (2.0, -1.0)]);
    let mut anEvaluator =
        ExtremaPC2d::BezierCurve::new(Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap()));
    let aResult = anEvaluator
        .perform(Point2::new(5.0, 3.0), THE_TOL, MM)
        .unwrap();
    assert!(aResult.is_infinite());
    EXPECT_NEAR(aResult.infinite_square_distance, 25.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_BezierCurveTest__MaxDegreeRationalWorkspaceHandlesConstantCurve() {
    let aNbPoles: i32 = Geom2dBezierCurve::max_degree() + 1;
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, aNbPoles).unwrap();
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, aNbPoles).unwrap();
    for aPoleIndex in 0..aNbPoles {
        aPoles[aPoleIndex + 1] = Point2::new(2.0, -1.0);
        aWeights[aPoleIndex + 1] = 1.0 + 0.01 * f64::from(aPoleIndex + 1);
    }

    let aCurve = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());
    let mut anExtrema = ExtremaPC2d::BezierCurve::new(aCurve);
    let aResult = anExtrema
        .perform(Point2::new(5.0, 3.0), THE_TOL, MM)
        .unwrap();
    assert!(aResult.is_infinite());
    EXPECT_NEAR(aResult.infinite_square_distance, 25.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_BezierCurveTest__MutationIsObserved() {
    let aCurve = makeQuadratic();
    let mut anEvaluator = ExtremaPC2d::BezierCurve::new(aCurve.clone());
    let aBefore: f64 = anEvaluator
        .perform(Point2::new(0.0, 1.5), THE_TOL, MM)
        .unwrap()
        .min_square_distance();
    aCurve.set_pole(2, Point2::new(3.0, -4.0)).unwrap();
    let anAfter = anEvaluator
        .perform(Point2::new(0.0, 1.5), THE_TOL, MM)
        .unwrap();
    assert!(anAfter.is_done());
    EXPECT_GT((anAfter.min_square_distance() - aBefore).abs(), 1.0e-4);
    let aMin = &anAfter[anAfter.min_index()];
    EXPECT_NEAR(
        aMin.point.distance(aCurve.value(aMin.parameter).unwrap()),
        0.0,
        THE_TOL,
    );
}

#[test]
fn ExtremaPC2d_BezierCurveTest__SingletonInvalidNullAndOwnership() {
    let mut aSingleton =
        ExtremaPC2d::BezierCurve::with_domain(makeLinear(), Domain1D::new(0.4, 0.4));
    assert_eq!(
        aSingleton
            .perform(Point2::default(), THE_TOL, MM)
            .unwrap()
            .nb_ext(),
        0
    );
    let anEnds = aSingleton.perform_with_endpoints(Point2::default(), THE_TOL, MM);
    assert_eq!(anEnds.unwrap().nb_ext(), 1);
    let mut anInvalid =
        ExtremaPC2d::BezierCurve::with_domain(makeLinear(), Domain1D::new(1.0, 0.0));
    assert_eq!(
        anInvalid
            .perform(Point2::default(), THE_TOL, MM)
            .unwrap()
            .status,
        Status::InvalidInput
    );
    let aNull: Handle<Geom2dBezierCurve> = Handle::default();
    let mut aNullEvaluator = ExtremaPC2d::BezierCurve::new(aNull);
    assert_eq!(
        aNullEvaluator
            .perform(Point2::default(), THE_TOL, MM)
            .unwrap()
            .status,
        Status::NotDone
    );

    let mut aCurve = makeLinear();
    let mut anOwned = ExtremaPC2d::BezierCurve::new(aCurve.clone());
    aCurve.nullify();
    assert!(!anOwned.curve().is_null());
    assert!(
        anOwned
            .perform(Point2::new(4.0, 3.0), THE_TOL, MM)
            .unwrap()
            .is_done()
    );
}
