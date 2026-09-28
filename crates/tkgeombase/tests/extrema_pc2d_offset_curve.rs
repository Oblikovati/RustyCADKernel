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

//! Direct translation of ExtremaPC2d_OffsetCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, dir2};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dOffsetCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::extrema_pc2d::{self as ExtremaPC2d, Domain1D, SearchMode, Status};
use rustycad_tkmath::gp::{Axis22d, Point2, Vector2};
use std::f64::consts::{FRAC_PI_2, PI};

const THE_TOL: f64 = 1.0e-8;
const MM: SearchMode = SearchMode::MinMax;

fn makeOffsetCircle(theOffset: f64) -> Handle<dyn Geom2dCurve> {
    let aFrame = Axis22d::new(Point2::default(), dir2(1.0, 0.0), true).unwrap();
    let aCircle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::new(aFrame, 3.0).unwrap()).cast();
    Handle::new(Geom2dOffsetCurve::new(&aCircle, theOffset, false).unwrap()).cast()
}

fn full_circle(anAdaptor: &Geom2dAdaptorCurve) -> ExtremaPC2d::OffsetCurve<'_> {
    ExtremaPC2d::OffsetCurve::with_domain(anAdaptor, Domain1D::new(0.0, 2.0 * PI))
}

#[test]
fn ExtremaPC2d_OffsetCurveTest__OffsetCircleFindsExactMinAndMax() {
    let aCurve = makeOffsetCircle(1.0);
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut anEvaluator = full_circle(&anAdaptor);
    let aResult = anEvaluator
        .perform(Point2::new(8.0, 0.0), THE_TOL, MM)
        .unwrap();
    assert_eq!(aResult.nb_ext(), 2);
    EXPECT_NEAR(aResult.min_square_distance(), 16.0, THE_TOL);
    EXPECT_NEAR(aResult.max_square_distance(), 144.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_OffsetCurveTest__CenterHasConstantDistance() {
    let aCurve = makeOffsetCircle(1.0);
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut anEvaluator = full_circle(&anAdaptor);
    let aResult = anEvaluator.perform(Point2::default(), THE_TOL, MM).unwrap();
    assert!(aResult.is_infinite());
    EXPECT_NEAR(aResult.infinite_square_distance, 16.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_OffsetCurveTest__NegativeOffsetUsesCorrectSide() {
    let aCurve = makeOffsetCircle(-1.0);
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut anEvaluator = full_circle(&anAdaptor);
    let aResult = anEvaluator
        .perform(Point2::new(8.0, 0.0), THE_TOL, MM)
        .unwrap();
    EXPECT_NEAR(aResult.min_square_distance(), 36.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_OffsetCurveTest__OffsetLineProjectionIsExact() {
    let aLine: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::default(), dir2(1.0, 0.0))).cast();
    let aCurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dOffsetCurve::new(&aLine, 2.0, false).unwrap()).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_range(&aCurve, 0.0, 4.0).unwrap();
    let mut anEvaluator =
        ExtremaPC2d::OffsetCurve::with_domain(&anAdaptor, Domain1D::new(0.0, 4.0));
    let aQuery: Point2 = aCurve
        .value(2.0)
        .unwrap()
        .translated(Vector2::new(0.0, 3.0));
    let aResult = anEvaluator.perform(aQuery, THE_TOL, MM).unwrap();
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 2.0, THE_TOL);
    EXPECT_NEAR(aResult[0].square_distance, 9.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_OffsetCurveTest__PartialArcAndSearchModes() {
    let aCurve = makeOffsetCircle(1.0);
    let anAdaptor = Geom2dAdaptorCurve::with_range(&aCurve, 0.0, FRAC_PI_2).unwrap();
    let mut anEvaluator =
        ExtremaPC2d::OffsetCurve::with_domain(&anAdaptor, Domain1D::new(0.0, FRAC_PI_2));
    let aMin = anEvaluator
        .perform_with_endpoints(Point2::new(8.0, -1.0), THE_TOL, SearchMode::Min)
        .unwrap();
    assert_eq!(aMin.nb_ext(), 1);
    EXPECT_NEAR(aMin[0].parameter, 0.0, THE_TOL);
    let aMax = anEvaluator
        .perform_with_endpoints(Point2::new(8.0, -1.0), THE_TOL, SearchMode::Max)
        .unwrap();
    assert_eq!(aMax.nb_ext(), 1);
    EXPECT_NEAR(aMax[0].parameter, FRAC_PI_2, THE_TOL);
}

#[test]
fn ExtremaPC2d_OffsetCurveTest__InvalidDomainAndTolerance() {
    let aCurve = makeOffsetCircle(1.0);
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut anInvalid = ExtremaPC2d::OffsetCurve::with_domain(&anAdaptor, Domain1D::new(1.0, 0.0));
    assert_eq!(
        anInvalid
            .perform(Point2::default(), THE_TOL, MM)
            .unwrap()
            .status,
        Status::InvalidInput
    );
    let mut anEvaluator = full_circle(&anAdaptor);
    assert_eq!(
        anEvaluator
            .perform(Point2::default(), 0.0, MM)
            .unwrap()
            .status,
        Status::InvalidInput
    );
}

#[test]
fn ExtremaPC2d_OffsetCurveTest__EndpointPathRefreshesPartitionAfterMutation() {
    let aNbPoles: usize = 10;
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, aNbPoles as i32).unwrap();
    for anIndex in 0..aNbPoles {
        let aParameter: f64 = anIndex as f64 / (aNbPoles - 1) as f64;
        aPoles[anIndex as i32 + 1] = Point2::new(aParameter, 0.0);
    }
    let aBasis = Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap());
    let aBasisCurve: Handle<dyn Geom2dCurve> = aBasis.cast();
    let aCurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dOffsetCurve::new(&aBasisCurve, 0.1, false).unwrap()).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let mut aReusedEvaluator =
        ExtremaPC2d::OffsetCurve::with_domain(&anAdaptor, Domain1D::new(0.0, 1.0));

    for anIndex in 1..aNbPoles - 1 {
        let aPoleNumber: i32 = anIndex as i32 + 1;
        let aParameter: f64 = anIndex as f64 / (aNbPoles - 1) as f64;
        let aY: f64 = if aPoleNumber % 2 == 0 { 2.0 } else { -2.0 };
        aBasis
            .set_pole(aPoleNumber, Point2::new(aParameter, aY))
            .unwrap();
    }

    let aQuery = Point2::new(0.5, 0.25);
    let aReusedResult = aReusedEvaluator
        .perform_with_endpoints(aQuery, THE_TOL, MM)
        .unwrap();
    let mut aFreshEvaluator =
        ExtremaPC2d::OffsetCurve::with_domain(&anAdaptor, Domain1D::new(0.0, 1.0));
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
