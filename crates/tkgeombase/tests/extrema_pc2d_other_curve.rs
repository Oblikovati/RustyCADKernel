//! Direct translation of ExtremaPC2d_OtherCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, dir2};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg2d::geom2d_eval::Geom2dEvalSineWaveCurve;
use rustycad_tkgeombase::extrema_pc2d::{self as ExtremaPC2d, Domain1D, SearchMode, Status};
use rustycad_tkmath::gp::{Axis2d, Point2, Vector2};
use std::f64::consts::PI;

const THE_TOL: f64 = 1.0e-8;
const MM: SearchMode = SearchMode::MinMax;

fn sineWave(omega: f64) -> Handle<dyn Geom2dCurve> {
    let anAxis = Axis2d::new(Point2::default(), dir2(1.0, 0.0));
    Handle::new(Geom2dEvalSineWaveCurve::new(anAxis, 1.0, omega, 0.0).unwrap()).cast()
}

#[test]
fn ExtremaPC2d_OtherCurveTest__SineWaveFindsOriginMinimum() {
    let aCurve = sineWave(1.0);
    let anAdaptor = Geom2dAdaptorCurve::with_range(&aCurve, -PI, PI).unwrap();
    let mut anEvaluator = ExtremaPC2d::OtherCurve::with_domain(&anAdaptor, Domain1D::new(-PI, PI));
    let aResult = anEvaluator
        .perform(Point2::new(-1.0, 1.0), THE_TOL, SearchMode::Min)
        .unwrap();
    assert!(aResult.nb_ext() >= 1);
    let mut hasOrigin = false;
    for anExtremum in &aResult.extrema {
        hasOrigin = hasOrigin || anExtremum.parameter.abs() <= THE_TOL;
        let aCurvePoint = aCurve.value(anExtremum.parameter).unwrap();
        EXPECT_NEAR(anExtremum.point.distance(aCurvePoint), 0.0, THE_TOL);
    }
    assert!(hasOrigin);
}

#[test]
fn ExtremaPC2d_OtherCurveTest__MultipleWavesReturnMultipleExtrema() {
    let aCurve = sineWave(3.0);
    let anAdaptor = Geom2dAdaptorCurve::with_range(&aCurve, -2.0, 2.0).unwrap();
    let mut anEvaluator =
        ExtremaPC2d::OtherCurve::with_domain(&anAdaptor, Domain1D::new(-2.0, 2.0));
    let aResult = anEvaluator.perform(Point2::default(), THE_TOL, MM).unwrap();
    assert!(aResult.is_done());
    assert!(aResult.nb_ext() >= 5);
    for anExtremum in &aResult.extrema {
        let (aPoint, aDerivative) = aCurve.d1(anExtremum.parameter).unwrap();
        EXPECT_NEAR(
            Vector2::between(Point2::default(), aPoint).dot(aDerivative),
            0.0,
            1.0e-6,
        );
    }
}

#[test]
fn ExtremaPC2d_OtherCurveTest__EndpointsAndModesAreForwarded() {
    let aCurve = sineWave(1.0);
    let anAdaptor = Geom2dAdaptorCurve::with_range(&aCurve, -1.0, 1.0).unwrap();
    let mut anEvaluator =
        ExtremaPC2d::OtherCurve::with_domain(&anAdaptor, Domain1D::new(-1.0, 1.0));
    let aMin = anEvaluator
        .perform_with_endpoints(Point2::new(3.0, 0.0), THE_TOL, SearchMode::Min)
        .unwrap();
    for anExtremum in &aMin.extrema {
        assert!(anExtremum.is_minimum);
    }
    let aMax = anEvaluator
        .perform_with_endpoints(Point2::new(3.0, 0.0), THE_TOL, SearchMode::Max)
        .unwrap();
    for anExtremum in &aMax.extrema {
        assert!(anExtremum.is_maximum);
    }
}

#[test]
fn ExtremaPC2d_OtherCurveTest__NaturalAndInvalidDomains() {
    let aCurve = sineWave(1.0);
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut aNatural = ExtremaPC2d::OtherCurve::new(&anAdaptor).unwrap();
    assert_eq!(
        aNatural
            .perform(Point2::default(), THE_TOL, MM)
            .unwrap()
            .status,
        Status::InvalidInput
    );
    let mut anInvalid = ExtremaPC2d::OtherCurve::with_domain(&anAdaptor, Domain1D::new(1.0, 0.0));
    assert_eq!(
        anInvalid
            .perform(Point2::default(), THE_TOL, MM)
            .unwrap()
            .status,
        Status::InvalidInput
    );
}

#[test]
fn ExtremaPC2d_OtherCurveTest__EndpointPathRefreshesPartitionAfterMutation() {
    let aNbPoles: usize = 26;
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, aNbPoles as i32).unwrap();
    for anIndex in 0..aNbPoles {
        let aParameter: f64 = anIndex as f64 / (aNbPoles - 1) as f64;
        aPoles[anIndex as i32 + 1] = Point2::new(aParameter, 0.0);
    }
    let aBezier = Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap());
    let aCurve: Handle<dyn Geom2dCurve> = aBezier.cast();
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut aReusedEvaluator =
        ExtremaPC2d::OtherCurve::with_domain(&anAdaptor, Domain1D::new(0.0, 1.0));

    for anIndex in 1..aNbPoles - 1 {
        let aPoleNumber: i32 = anIndex as i32 + 1;
        let aParameter: f64 = anIndex as f64 / (aNbPoles - 1) as f64;
        let aY: f64 = if aPoleNumber % 2 == 0 { 4.0 } else { -4.0 };
        aBezier
            .set_pole(aPoleNumber, Point2::new(aParameter, aY))
            .unwrap();
    }

    let aQuery = Point2::new(0.5, 0.25);
    let aReusedResult = aReusedEvaluator
        .perform_with_endpoints(aQuery, THE_TOL, MM)
        .unwrap();
    let mut aFreshEvaluator =
        ExtremaPC2d::OtherCurve::with_domain(&anAdaptor, Domain1D::new(0.0, 1.0));
    let aFreshResult = aFreshEvaluator
        .perform_with_endpoints(aQuery, THE_TOL, MM)
        .unwrap();

    assert_eq!(aReusedResult.status, aFreshResult.status);
    assert_eq!(aReusedResult.nb_ext(), aFreshResult.nb_ext());
    for anIndex in 0..aFreshResult.nb_ext() {
        EXPECT_NEAR(
            aReusedResult[anIndex].parameter,
            aFreshResult[anIndex].parameter,
            1.0e-7,
        );
        EXPECT_NEAR(
            aReusedResult[anIndex].square_distance,
            aFreshResult[anIndex].square_distance,
            1.0e-7,
        );
    }
}
