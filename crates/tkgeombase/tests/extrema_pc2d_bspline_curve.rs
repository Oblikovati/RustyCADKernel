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

//! Direct translation of ExtremaPC2d_BSplineCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_GT, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkgeombase::extrema_pc2d::{self as ExtremaPC2d, Domain1D, SearchMode, Status};
use rustycad_tkmath::gp::Point2;

const THE_TOL: f64 = 1.0e-8;
const MM: SearchMode = SearchMode::MinMax;

fn bspline(
    points: &[(f64, f64)],
    knots: &[f64],
    mults: &[i32],
    degree: i32,
) -> Handle<Geom2dBSplineCurve> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, (x, y)) in points.iter().enumerate() {
        aPoles[i as i32 + 1] = Point2::new(*x, *y);
    }
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, knots.len() as i32).unwrap();
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, mults.len() as i32).unwrap();
    for i in 0..knots.len() {
        aKnots[i as i32 + 1] = knots[i];
        aMults[i as i32 + 1] = mults[i];
    }
    Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, degree, false).unwrap())
}

fn makeLinear() -> Handle<Geom2dBSplineCurve> {
    bspline(&[(0.0, 0.0), (10.0, 0.0)], &[0.0, 1.0], &[2, 2], 1)
}

fn makeC1Curve() -> Handle<Geom2dBSplineCurve> {
    let aPoles = [(0.0, 0.0), (1.0, 1.0), (2.0, 1.0), (3.0, 0.0)];
    bspline(&aPoles, &[0.0, 0.5, 1.0], &[3, 1, 3], 2)
}

#[test]
fn ExtremaPC2d_BSplineCurveTest__LinearProjectionIsExact() {
    let mut anEvaluator = ExtremaPC2d::BSplineCurve::new(makeLinear());
    let aResult = anEvaluator
        .perform(Point2::new(6.0, 4.0), THE_TOL, MM)
        .unwrap();
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.6, THE_TOL);
    EXPECT_NEAR(aResult[0].square_distance, 16.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_BSplineCurveTest__PartialDomainUsesEndpointWhenRequested() {
    let mut anEvaluator =
        ExtremaPC2d::BSplineCurve::with_domain(makeLinear(), Domain1D::new(0.0, 0.5));
    assert_eq!(
        anEvaluator
            .perform(Point2::new(8.0, 3.0), THE_TOL, MM)
            .unwrap()
            .nb_ext(),
        0
    );
    let aResult = anEvaluator
        .perform_with_endpoints(Point2::new(8.0, 3.0), THE_TOL, SearchMode::Min)
        .unwrap();
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.5, THE_TOL);
}

#[test]
fn ExtremaPC2d_BSplineCurveTest__C1JunctionPointIsFound() {
    let aCurve = makeC1Curve();
    let mut anEvaluator = ExtremaPC2d::BSplineCurve::new(aCurve.clone());
    let aResult = anEvaluator
        .perform(aCurve.value(0.5).unwrap(), THE_TOL, SearchMode::Min)
        .unwrap();
    assert!(aResult.nb_ext() >= 1);
    EXPECT_NEAR(aResult.min_square_distance(), 0.0, THE_TOL);
    EXPECT_NEAR(aResult[aResult.min_index()].parameter, 0.5, THE_TOL);
}

#[test]
fn ExtremaPC2d_BSplineCurveTest__DegenerateCurveHasInfiniteSolutions() {
    let aCurve = makeLinear();
    aCurve.set_pole(1, Point2::new(2.0, -1.0)).unwrap();
    aCurve.set_pole(2, Point2::new(2.0, -1.0)).unwrap();
    let mut anEvaluator = ExtremaPC2d::BSplineCurve::new(aCurve);
    let aResult = anEvaluator
        .perform(Point2::new(5.0, 3.0), THE_TOL, MM)
        .unwrap();
    assert!(aResult.is_infinite());
    EXPECT_NEAR(aResult.infinite_square_distance, 25.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_BSplineCurveTest__PoleMutationIsObserved() {
    let aCurve = makeC1Curve();
    let mut anEvaluator = ExtremaPC2d::BSplineCurve::new(aCurve.clone());
    let aBefore: f64 = anEvaluator
        .perform(Point2::new(1.5, 2.0), THE_TOL, MM)
        .unwrap()
        .min_square_distance();
    aCurve.set_pole(2, Point2::new(1.0, -3.0)).unwrap();
    let anAfter = anEvaluator
        .perform(Point2::new(1.5, 2.0), THE_TOL, MM)
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
fn ExtremaPC2d_BSplineCurveTest__SearchModesFilterExtrema() {
    let mut anEvaluator = ExtremaPC2d::BSplineCurve::new(makeC1Curve());
    let aMin = anEvaluator
        .perform(Point2::new(1.5, 2.0), THE_TOL, SearchMode::Min)
        .unwrap();
    for anExtremum in &aMin.extrema {
        assert!(anExtremum.is_minimum);
    }
    let aMax = anEvaluator
        .perform(Point2::new(1.5, 2.0), THE_TOL, SearchMode::Max)
        .unwrap();
    for anExtremum in &aMax.extrema {
        assert!(anExtremum.is_maximum);
    }
}

#[test]
fn ExtremaPC2d_BSplineCurveTest__SingletonInvalidNullAndOwnership() {
    let mut aSingleton =
        ExtremaPC2d::BSplineCurve::with_domain(makeLinear(), Domain1D::new(0.4, 0.4));
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
        ExtremaPC2d::BSplineCurve::with_domain(makeLinear(), Domain1D::new(1.0, 0.0));
    assert_eq!(
        anInvalid
            .perform(Point2::default(), THE_TOL, MM)
            .unwrap()
            .status,
        Status::InvalidInput
    );
    let aNull: Handle<Geom2dBSplineCurve> = Handle::default();
    let mut aNullEvaluator = ExtremaPC2d::BSplineCurve::new(aNull);
    assert_eq!(
        aNullEvaluator
            .perform(Point2::default(), THE_TOL, MM)
            .unwrap()
            .status,
        Status::NotDone
    );
    let mut aCurve = makeLinear();
    let anOwned = ExtremaPC2d::BSplineCurve::new(aCurve.clone());
    aCurve.nullify();
    assert!(!anOwned.curve().is_null());
}
