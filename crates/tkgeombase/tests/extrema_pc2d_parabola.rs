//! Direct translation of ExtremaPC2d_Parabola_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, dir2};
use rustycad_tkgeombase::extrema_pc2d::{self as ExtremaPC2d, Domain1D, SearchMode, Status};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis22d, Parabola2, Point2};

const THE_TOL: f64 = 1.0e-8;
const MM: SearchMode = SearchMode::MinMax;

fn makeParabola(theFocal: f64) -> Parabola2 {
    let aFrame = Axis22d::new(Point2::default(), dir2(1.0, 0.0), true).unwrap();
    Parabola2::new(aFrame, theFocal).unwrap()
}

fn bounded(domain: (f64, f64)) -> ExtremaPC2d::Parabola {
    ExtremaPC2d::Parabola::with_domain(makeParabola(1.0), Domain1D::new(domain.0, domain.1))
}

#[test]
fn ExtremaPC2d_ParabolaTest__VertexAndBehindVertexHaveUniqueMinimum() {
    let mut anEvaluator = ExtremaPC2d::Parabola::new(makeParabola(1.0));
    let aVertex = anEvaluator.perform(Point2::default(), THE_TOL, MM);
    assert_eq!(aVertex.nb_ext(), 1);
    EXPECT_NEAR(aVertex[0].parameter, 0.0, THE_TOL);
    EXPECT_NEAR(aVertex[0].square_distance, 0.0, THE_TOL);
    let aBehind = anEvaluator.perform(Point2::new(-2.0, 0.0), THE_TOL, MM);
    assert_eq!(aBehind.nb_ext(), 1);
    EXPECT_NEAR(aBehind[0].square_distance, 4.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_ParabolaTest__AxisPointAheadHasThreeExtrema() {
    let mut anEvaluator = ExtremaPC2d::Parabola::new(makeParabola(1.0));
    let aResult = anEvaluator.perform(Point2::new(3.0, 0.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 3);
    EXPECT_NEAR(aResult.min_square_distance(), 8.0, THE_TOL);
    EXPECT_NEAR(aResult.max_square_distance(), 9.0, THE_TOL);
    let aMinima = aResult.extrema.iter().filter(|e| e.is_minimum).count();
    assert_eq!(aMinima, 2);
}

#[test]
fn ExtremaPC2d_ParabolaTest__SearchModesReturnExactClasses() {
    let mut anEvaluator = ExtremaPC2d::Parabola::new(makeParabola(1.0));
    assert_eq!(
        anEvaluator
            .perform(Point2::new(3.0, 0.0), THE_TOL, SearchMode::Min)
            .nb_ext(),
        2
    );
    let aMax = anEvaluator.perform(Point2::new(3.0, 0.0), THE_TOL, SearchMode::Max);
    assert_eq!(aMax.nb_ext(), 1);
    EXPECT_NEAR(aMax[0].parameter, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_ParabolaTest__KnownCubicHasThreeRoots() {
    let mut anEvaluator = ExtremaPC2d::Parabola::new(makeParabola(1.0));
    let aResult = anEvaluator.perform(Point2::new(3.75, -0.75), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 3);
    let has = |u: f64| {
        aResult
            .extrema
            .iter()
            .any(|e| (e.parameter - u).abs() <= THE_TOL)
    };
    assert!(has(-3.0));
    assert!(has(1.0));
    assert!(has(2.0));
}

#[test]
fn ExtremaPC2d_ParabolaTest__RootWithinParameterToleranceIsClampedToBoundary() {
    let anUpper: f64 = 1.0 - 0.5 * ExtremaPC2d::THE_PARAM_TOLERANCE;
    let mut anEvaluator = bounded((0.5, anUpper));
    let aResult = anEvaluator.perform(Point2::new(3.75, -0.75), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 1);
    assert_eq!(aResult[0].parameter, anUpper);
}

#[test]
fn ExtremaPC2d_ParabolaTest__PointOnCurveHasZeroDistanceMinimum() {
    let aCurve = makeParabola(1.0);
    let mut anEvaluator = ExtremaPC2d::Parabola::new(aCurve);
    let aResult = anEvaluator.perform(elclib::value(2.0, &aCurve), THE_TOL, MM);
    EXPECT_NEAR(aResult.min_square_distance(), 0.0, THE_TOL);
    EXPECT_NEAR(aResult[aResult.min_index()].parameter, 2.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_ParabolaTest__IndirectOrientationPreservesParameterSense() {
    let aPosition =
        Axis22d::with_directions(Point2::new(1.0, 2.0), dir2(0.0, 1.0), dir2(1.0, 0.0)).unwrap();
    let mut anEvaluator = ExtremaPC2d::Parabola::new(Parabola2::new(aPosition, 1.0).unwrap());
    let aResult = anEvaluator.perform(Point2::new(1.0, 5.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 3);
    EXPECT_NEAR(aResult.min_square_distance(), 8.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_ParabolaTest__BoundedArcAddsEndpointMinima() {
    let mut anEvaluator = bounded((-1.0, 1.0));
    let aResult = anEvaluator.perform_with_endpoints(Point2::new(3.0, 0.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 3);
    EXPECT_NEAR(aResult[aResult.max_index()].parameter, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_ParabolaTest__ZeroFocalMatchesMirrorAxisLine() {
    let aPosition = Axis22d::new(Point2::new(1.0, 2.0), dir2(0.0, 1.0), true).unwrap();
    let mut anEvaluator = ExtremaPC2d::Parabola::new(Parabola2::new(aPosition, 0.0).unwrap());
    let aResult = anEvaluator.perform(Point2::new(4.0, 7.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 5.0, THE_TOL);
    EXPECT_NEAR(aResult[0].square_distance, 9.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_ParabolaTest__SingletonInvalidAndValueAccessor() {
    let mut aSingleton = bounded((0.4, 0.4));
    assert_eq!(
        aSingleton.perform(Point2::default(), THE_TOL, MM).nb_ext(),
        0
    );
    assert_eq!(
        aSingleton
            .perform_with_endpoints(Point2::default(), THE_TOL, MM)
            .nb_ext(),
        1
    );
    let mut anInvalid = bounded((1.0, 0.0));
    assert_eq!(
        anInvalid.perform(Point2::default(), THE_TOL, MM).status,
        Status::InvalidInput
    );
    let expected: Point2 = elclib::value(-0.7, &makeParabola(1.0));
    assert_eq!(aSingleton.value(-0.7).distance(expected), 0.0);
}
