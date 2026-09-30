//! Direct translation of ExtremaPC2d_GridEvaluator_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, dir2};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCircle, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::extrema_pc2d::{Domain1D, GridEvaluator, SearchMode, Status};
use rustycad_tkmath::gp::{Axis22d, Point2};
use rustycad_tkmath::math::Vector;
use std::f64::consts::PI;

fn bezier(points: &[(f64, f64)]) -> Handle<dyn Geom2dCurve> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, (x, y)) in points.iter().enumerate() {
        aPoles[i as i32 + 1] = Point2::new(*x, *y);
    }
    Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap()).cast()
}

#[test]
fn ExtremaPC2d_GridEvaluatorTest__UniformParamsCoverDomainExactly() {
    let aParams: Vector<'static> = GridEvaluator::build_uniform_params(-3.0, 5.0, 7).unwrap();
    assert_eq!(aParams.size(), 7);
    assert_eq!(*aParams.at(0).unwrap(), -3.0);
    assert_eq!(*aParams.at(6).unwrap(), 5.0);
    for anIndex in 1..aParams.size() {
        assert!(*aParams.at(anIndex).unwrap() > *aParams.at(anIndex - 1).unwrap());
    }
}

#[test]
fn ExtremaPC2d_GridEvaluatorTest__UniformParamsRejectInvalidSampleCount() {
    // The pinned build raises Standard_RangeError (No_Exception is not defined).
    let aFailure = GridEvaluator::build_uniform_params(0.0, 1.0, 1).unwrap_err();
    assert_eq!(aFailure.exception_type(), "Standard_RangeError");
}

#[test]
fn ExtremaPC2d_GridEvaluatorTest__InvalidPartitionAndToleranceAreRejected() {
    let aCurve = bezier(&[(0.0, 0.0), (1.0, 0.0)]);
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut anEvaluator = GridEvaluator::default();
    let mm = SearchMode::MinMax;
    let aStatus = anEvaluator
        .perform(
            &anAdaptor,
            &Point2::default(),
            &Domain1D::new(0.0, 1.0),
            1.0e-8,
            mm,
        )
        .status;
    assert_eq!(aStatus, Status::InvalidInput);

    let mut aParams: Vector<'static> = Vector::with_size(3);
    *aParams.change_at(0).unwrap() = 0.0;
    *aParams.change_at(1).unwrap() = 0.5;
    *aParams.change_at(2).unwrap() = 1.0;
    anEvaluator.set_params(&aParams);
    let aStatus = anEvaluator
        .perform(
            &anAdaptor,
            &Point2::default(),
            &Domain1D::new(1.0, 0.0),
            1.0e-8,
            mm,
        )
        .status;
    assert_eq!(aStatus, Status::InvalidInput);
    let aStatus = anEvaluator
        .perform(
            &anAdaptor,
            &Point2::default(),
            &Domain1D::new(0.0, 1.0),
            0.0,
            mm,
        )
        .status;
    assert_eq!(aStatus, Status::InvalidInput);
}

#[test]
fn ExtremaPC2d_GridEvaluatorTest__ConstantCurveHasInfiniteSolutions() {
    let aCurve = bezier(&[(2.0, -1.0), (2.0, -1.0), (2.0, -1.0)]);
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut anEvaluator = GridEvaluator::default();
    anEvaluator.set_params(&GridEvaluator::build_uniform_params(0.0, 1.0, 8).unwrap());
    let aResult = anEvaluator.perform(
        &anAdaptor,
        &Point2::new(5.0, 3.0),
        &Domain1D::new(0.0, 1.0),
        1.0e-8,
        SearchMode::MinMax,
    );
    assert!(aResult.is_infinite());
    EXPECT_NEAR(aResult.infinite_square_distance, 25.0, 1.0e-12);
}

#[test]
fn ExtremaPC2d_GridEvaluatorTest__ConstantCircleDistanceHasInfiniteSolutions() {
    let aFrame = Axis22d::new(Point2::default(), dir2(1.0, 0.0), true).unwrap();
    let aCircle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::new(aFrame, 3.0).unwrap()).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCircle).unwrap();
    let mut anEvaluator = GridEvaluator::default();
    anEvaluator.set_params(&GridEvaluator::build_uniform_params(0.0, 2.0 * PI, 32).unwrap());
    let aResult = anEvaluator.perform(
        &anAdaptor,
        &Point2::default(),
        &Domain1D::new(0.0, 2.0 * PI),
        1.0e-8,
        SearchMode::MinMax,
    );
    assert!(aResult.is_infinite());
    EXPECT_NEAR(aResult.infinite_square_distance, 9.0, 1.0e-10);
}

#[test]
fn ExtremaPC2d_GridEvaluatorTest__QuadraticCurveFindsCentralMinimum() {
    let aCurve = bezier(&[(-2.0, 0.0), (0.0, 2.0), (2.0, 0.0)]);
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut anEvaluator = GridEvaluator::default();
    anEvaluator.set_params(&GridEvaluator::build_uniform_params(0.0, 1.0, 24).unwrap());
    let aResult = anEvaluator.perform(
        &anAdaptor,
        &Point2::default(),
        &Domain1D::new(0.0, 1.0),
        1.0e-9,
        SearchMode::Min,
    );
    assert!(aResult.is_done());
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.5, 1.0e-8);
    EXPECT_NEAR(aResult[0].square_distance, 1.0, 1.0e-8);
    assert!(aResult[0].is_minimum);
}

#[test]
fn ExtremaPC2d_GridEvaluatorTest__ShortOpenCurveIsNotClosed() {
    let aCurve = bezier(&[(0.0, 0.0), (0.5 * precision::confusion(), 0.0)]);
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    assert!(!GridEvaluator::is_closed_domain(
        &anAdaptor,
        &Domain1D::new(0.0, 1.0)
    ));
}
