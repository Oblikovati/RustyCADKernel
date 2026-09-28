// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Direct translation of ExtremaPC2d_Curve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_GE, EXPECT_GT, EXPECT_LE, EXPECT_NEAR, dir2};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg2d::geom2d_eval::Geom2dEvalSineWaveCurve;
use rustycad_tkgeombase::extrema_pc2d::{
    self as ExtremaPC2d, Domain1D, ExtremaResult, SearchMode, Status,
};
use rustycad_tkmath::gp::{Axis2d, Axis22d, Point2};
use std::f64::consts::PI;

const THE_TOL: f64 = 1.0e-8;
const MM: SearchMode = SearchMode::MinMax;

fn compareResults(theFirst: &ExtremaResult, theSecond: &ExtremaResult) {
    assert_eq!(theFirst.status, theSecond.status);
    assert_eq!(theFirst.nb_ext(), theSecond.nb_ext());
    for anIndex in 0..theFirst.nb_ext() {
        EXPECT_NEAR(
            theFirst[anIndex].parameter,
            theSecond[anIndex].parameter,
            THE_TOL,
        );
        EXPECT_NEAR(
            theFirst[anIndex].square_distance,
            theSecond[anIndex].square_distance,
            THE_TOL,
        );
        assert_eq!(theFirst[anIndex].is_minimum, theSecond[anIndex].is_minimum);
        assert_eq!(theFirst[anIndex].is_maximum, theSecond[anIndex].is_maximum);
    }
}

fn poles(points: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut aPoles = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, (x, y)) in points.iter().enumerate() {
        aPoles[i as i32 + 1] = Point2::new(*x, *y);
    }
    aPoles
}

fn makeBezier() -> Handle<Geom2dBezierCurve> {
    let aPoles = poles(&[(0.0, 0.0), (2.0, 3.0), (4.0, 0.0)]);
    Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap())
}

fn makeBSpline() -> Handle<Geom2dBSplineCurve> {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 2.0), (3.0, -1.0), (4.0, 0.0)]);
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 4;
    aMults[2] = 4;
    Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap())
}

fn frame() -> Axis22d {
    Axis22d::new(Point2::default(), dir2(1.0, 0.0), true).unwrap()
}

fn circle(radius: f64) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dCircle::new(frame(), radius).unwrap()).cast()
}

#[test]
fn ExtremaPC2d_CurveTest__NullCurveIsUninitializedAndNotDone() {
    let aNull: Handle<dyn Geom2dCurve> = Handle::default();
    let mut anEvaluator = ExtremaPC2d::Curve::from_geom(&aNull).unwrap();
    assert!(!anEvaluator.is_initialized());
    assert_eq!(
        anEvaluator
            .perform(Point2::default(), THE_TOL, MM)
            .unwrap()
            .status,
        Status::NotDone
    );
    assert_eq!(
        anEvaluator
            .perform_with_endpoints(Point2::default(), THE_TOL, MM)
            .unwrap()
            .status,
        Status::NotDone
    );
}

#[test]
fn ExtremaPC2d_CurveTest__AnalyticalAdaptorDispatchMatchesSpecialists() {
    let aQuery = Point2::new(8.0, 3.0);
    let aLine = Handle::new(Geom2dLine::from_point(Point2::default(), dir2(1.0, 0.0)));
    let aLineAdaptor = Geom2dAdaptorCurve::with_range(&aLine.cast(), -10.0, 10.0).unwrap();
    let mut aLineAggregate = ExtremaPC2d::Curve::from_adaptor(&aLineAdaptor).unwrap();
    let mut aLineDirect = ExtremaPC2d::Line::with_domain(aLine.lin2d(), Domain1D::new(-10.0, 10.0));
    compareResults(
        aLineAggregate.perform(aQuery, THE_TOL, MM).unwrap(),
        aLineDirect.perform(aQuery, THE_TOL, MM),
    );

    let aCircle = Handle::new(Geom2dCircle::new(frame(), 4.0).unwrap());
    let mut aCircleAggregate = ExtremaPC2d::Curve::from_geom(&aCircle.cast()).unwrap();
    let mut aCircleDirect = ExtremaPC2d::Circle::new(aCircle.circ2d().unwrap());
    compareResults(
        aCircleAggregate.perform(aQuery, THE_TOL, MM).unwrap(),
        aCircleDirect.perform(aQuery, THE_TOL, MM),
    );

    let anEllipse = Handle::new(Geom2dEllipse::new(frame(), 5.0, 2.0).unwrap());
    let mut anEllipseAggregate = ExtremaPC2d::Curve::from_geom(&anEllipse.cast()).unwrap();
    let mut anEllipseDirect = ExtremaPC2d::Ellipse::new(anEllipse.elips2d().unwrap());
    compareResults(
        anEllipseAggregate.perform(aQuery, THE_TOL, MM).unwrap(),
        anEllipseDirect.perform(aQuery, THE_TOL, MM),
    );

    let aHyperbola = Handle::new(Geom2dHyperbola::new(frame(), 3.0, 2.0).unwrap());
    let aHyperbolaAdaptor = Geom2dAdaptorCurve::with_range(&aHyperbola.cast(), -2.0, 2.0).unwrap();
    let mut aHyperbolaAggregate = ExtremaPC2d::Curve::from_adaptor(&aHyperbolaAdaptor).unwrap();
    let mut aHyperbolaDirect =
        ExtremaPC2d::Hyperbola::with_domain(aHyperbola.hypr2d().unwrap(), Domain1D::new(-2.0, 2.0));
    compareResults(
        aHyperbolaAggregate.perform(aQuery, THE_TOL, MM).unwrap(),
        aHyperbolaDirect.perform(aQuery, THE_TOL, MM),
    );

    let aParabola = Handle::new(Geom2dParabola::new(frame(), 2.0).unwrap());
    let aParabolaAdaptor = Geom2dAdaptorCurve::with_range(&aParabola.cast(), -2.0, 2.0).unwrap();
    let mut aParabolaAggregate = ExtremaPC2d::Curve::from_adaptor(&aParabolaAdaptor).unwrap();
    let mut aParabolaDirect =
        ExtremaPC2d::Parabola::with_domain(aParabola.parab2d().unwrap(), Domain1D::new(-2.0, 2.0));
    compareResults(
        aParabolaAggregate.perform(aQuery, THE_TOL, MM).unwrap(),
        aParabolaDirect.perform(aQuery, THE_TOL, MM),
    );
}

#[test]
fn ExtremaPC2d_CurveTest__NumericalHandleDispatchMatchesSpecialists() {
    let aQuery = Point2::new(2.0, 1.5);
    let aBezier = makeBezier();
    let mut aBezierAggregate = ExtremaPC2d::Curve::from_geom(&aBezier.cast()).unwrap();
    let mut aBezierDirect = ExtremaPC2d::BezierCurve::new(aBezier);
    compareResults(
        aBezierAggregate.perform(aQuery, THE_TOL, MM).unwrap(),
        aBezierDirect.perform(aQuery, THE_TOL, MM).unwrap(),
    );

    let aBSpline = makeBSpline();
    let mut aBSplineAggregate = ExtremaPC2d::Curve::from_geom(&aBSpline.cast()).unwrap();
    let mut aBSplineDirect = ExtremaPC2d::BSplineCurve::new(aBSpline);
    compareResults(
        aBSplineAggregate.perform(aQuery, THE_TOL, MM).unwrap(),
        aBSplineDirect.perform(aQuery, THE_TOL, MM).unwrap(),
    );
}

#[test]
fn ExtremaPC2d_CurveTest__OffsetDispatchAndSearchModes() {
    let anOffset: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dOffsetCurve::new(&circle(3.0), 1.0, false).unwrap()).cast();
    let mut anEvaluator = ExtremaPC2d::Curve::from_geom_range(&anOffset, 0.0, 2.0 * PI).unwrap();
    let aMin = anEvaluator
        .perform(Point2::new(8.0, 0.0), THE_TOL, SearchMode::Min)
        .unwrap();
    assert_eq!(aMin.nb_ext(), 1);
    assert!(aMin[0].is_minimum);
    let aMax = anEvaluator
        .perform(Point2::new(8.0, 0.0), THE_TOL, SearchMode::Max)
        .unwrap();
    assert_eq!(aMax.nb_ext(), 1);
    assert!(aMax[0].is_maximum);
}

#[test]
fn ExtremaPC2d_CurveTest__GeneralCurveDispatchMatchesSpecialist() {
    let anAxis = Axis2d::new(Point2::default(), dir2(1.0, 0.0));
    let aCurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dEvalSineWaveCurve::new(anAxis, 1.0, 2.0, 0.0).unwrap()).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_range(&aCurve, -2.0, 2.0).unwrap();
    let mut anAggregate = ExtremaPC2d::Curve::from_adaptor(&anAdaptor).unwrap();
    let mut aDirect = ExtremaPC2d::OtherCurve::with_domain(&anAdaptor, Domain1D::new(-2.0, 2.0));
    compareResults(
        anAggregate
            .perform(Point2::new(0.5, 1.0), THE_TOL, MM)
            .unwrap(),
        aDirect.perform(Point2::new(0.5, 1.0), THE_TOL, MM).unwrap(),
    );
}

#[test]
fn ExtremaPC2d_CurveTest__TrimmedCurveUsesTrimBounds() {
    let aTrimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&circle(3.0), 0.2, 1.0, true, true).unwrap()).cast();
    let mut anEvaluator = ExtremaPC2d::Curve::from_geom(&aTrimmed).unwrap();
    let aResult = anEvaluator
        .perform_with_endpoints(Point2::new(6.0, 0.0), THE_TOL, MM)
        .unwrap();
    assert!(aResult.is_done());
    for anExtremum in &aResult.extrema {
        EXPECT_GE(anExtremum.parameter, 0.2 - THE_TOL);
        EXPECT_LE(anExtremum.parameter, 1.0 + THE_TOL);
    }
}

#[test]
fn ExtremaPC2d_CurveTest__ExplicitBoundsIntersectTrimmedBounds() {
    let aTrimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&circle(3.0), 0.2, 1.0, true, true).unwrap()).cast();
    let mut anEvaluator = ExtremaPC2d::Curve::from_geom_range(&aTrimmed, 0.5, 2.0).unwrap();
    let aResult = anEvaluator
        .perform_with_endpoints(Point2::new(6.0, 0.0), THE_TOL, MM)
        .unwrap();
    for anExtremum in &aResult.extrema {
        EXPECT_GE(anExtremum.parameter, 0.5 - THE_TOL);
        EXPECT_LE(anExtremum.parameter, 1.0 + THE_TOL);
    }
}

#[test]
fn ExtremaPC2d_CurveTest__BezierMutationFlowsThroughAggregator() {
    let aCurve = makeBezier();
    let mut anEvaluator = ExtremaPC2d::Curve::from_geom(&aCurve.cast()).unwrap();
    let aBefore: f64 = anEvaluator
        .perform(Point2::new(2.0, 2.0), THE_TOL, MM)
        .unwrap()
        .min_square_distance();
    aCurve.set_pole(2, Point2::new(3.5, -4.0)).unwrap();
    let anAfter = anEvaluator
        .perform(Point2::new(2.0, 2.0), THE_TOL, MM)
        .unwrap();
    EXPECT_GT((anAfter.min_square_distance() - aBefore).abs(), 1.0e-4);
    let aMin = &anAfter[anAfter.min_index()];
    EXPECT_NEAR(
        aMin.point.distance(aCurve.value(aMin.parameter).unwrap()),
        0.0,
        THE_TOL,
    );
}

#[test]
fn ExtremaPC2d_CurveTest__AnalyticalDispatchCopiesPrimitiveGeometry() {
    let aLine: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::default(), dir2(1.0, 0.0))).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_range(&aLine, -10.0, 10.0).unwrap();
    let mut anEvaluator = ExtremaPC2d::Curve::from_adaptor(&anAdaptor).unwrap();
    let aVertical: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::default(), dir2(0.0, 1.0))).cast();
    anAdaptor.load_range(&aVertical, -10.0, 10.0).unwrap();
    let aResult = anEvaluator
        .perform(Point2::new(4.0, 3.0), THE_TOL, MM)
        .unwrap();
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 4.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_CurveTest__ResultStorageIsEvaluatorLocal() {
    let mut aFirst = ExtremaPC2d::Curve::from_geom(&makeBezier().cast()).unwrap();
    let mut aSecond = ExtremaPC2d::Curve::from_geom(&makeBSpline().cast()).unwrap();
    let aFirstResult: *const ExtremaResult =
        aFirst.perform(Point2::new(1.0, 2.0), THE_TOL, MM).unwrap();
    let aSecondResult: *const ExtremaResult =
        aSecond.perform(Point2::new(1.0, 2.0), THE_TOL, MM).unwrap();
    assert_ne!(aFirstResult, aSecondResult);
    let aRepeated: *const ExtremaResult =
        aFirst.perform(Point2::new(3.0, 2.0), THE_TOL, MM).unwrap();
    assert_eq!(aFirstResult, aRepeated);
}
