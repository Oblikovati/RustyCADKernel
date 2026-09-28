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

//! Direct translation of ExtremaPC2d_Hyperbola_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, dir2};
use rustycad_tkgeombase::extrema_pc2d::{self as ExtremaPC2d, Domain1D, SearchMode, Status};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis22d, Hyperbola2, Point2};

const THE_TOL: f64 = 1.0e-8;
const MM: SearchMode = SearchMode::MinMax;

fn makeHyperbola(theMajor: f64, theMinor: f64) -> Hyperbola2 {
    let aFrame = Axis22d::new(Point2::default(), dir2(1.0, 0.0), true).unwrap();
    Hyperbola2::new(aFrame, theMajor, theMinor).unwrap()
}

fn bounded(domain: (f64, f64)) -> ExtremaPC2d::Hyperbola {
    ExtremaPC2d::Hyperbola::with_domain(makeHyperbola(2.0, 1.0), Domain1D::new(domain.0, domain.1))
}

#[test]
fn ExtremaPC2d_HyperbolaTest__CenterHasVertexMinimum() {
    let mut anEvaluator = ExtremaPC2d::Hyperbola::new(makeHyperbola(2.0, 1.0));
    let aResult = anEvaluator.perform(Point2::default(), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.0, THE_TOL);
    EXPECT_NEAR(aResult[0].square_distance, 4.0, THE_TOL);
    assert!(aResult[0].is_minimum);
}

#[test]
fn ExtremaPC2d_HyperbolaTest__MajorAxisPointHasThreeExtrema() {
    let aRoot: f64 = 2.0_f64.acosh();
    let mut anEvaluator = ExtremaPC2d::Hyperbola::new(makeHyperbola(2.0, 1.0));
    let aResult = anEvaluator.perform(Point2::new(5.0, 0.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 3);
    let aMinima = aResult.extrema.iter().filter(|e| e.is_minimum).count();
    let aMaxima = aResult.extrema.iter().filter(|e| e.is_maximum).count();
    let hasNegative = aResult
        .extrema
        .iter()
        .any(|e| (e.parameter + aRoot).abs() <= THE_TOL);
    let hasPositive = aResult
        .extrema
        .iter()
        .any(|e| (e.parameter - aRoot).abs() <= THE_TOL);
    assert_eq!(aMinima, 2);
    assert_eq!(aMaxima, 1);
    assert!(hasNegative);
    assert!(hasPositive);
    EXPECT_NEAR(aResult.min_square_distance(), 4.0, THE_TOL);
    EXPECT_NEAR(aResult.max_square_distance(), 9.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_HyperbolaTest__RootWithinParameterToleranceIsClampedToBoundary() {
    let aRoot: f64 = 2.0_f64.acosh();
    let anUpper: f64 = aRoot - 0.5 * ExtremaPC2d::THE_PARAM_TOLERANCE;
    let mut anEvaluator = bounded((0.5, anUpper));
    let aResult = anEvaluator.perform(Point2::new(5.0, 0.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 1);
    assert_eq!(aResult[0].parameter, anUpper);
}

#[test]
fn ExtremaPC2d_HyperbolaTest__SearchModesReturnExactClasses() {
    let mut anEvaluator = ExtremaPC2d::Hyperbola::new(makeHyperbola(2.0, 1.0));
    let aMin = anEvaluator.perform(Point2::new(5.0, 0.0), THE_TOL, SearchMode::Min);
    assert_eq!(aMin.nb_ext(), 2);
    assert!(aMin[0].is_minimum && aMin[1].is_minimum);
    let aMax = anEvaluator.perform(Point2::new(5.0, 0.0), THE_TOL, SearchMode::Max);
    assert_eq!(aMax.nb_ext(), 1);
    EXPECT_NEAR(aMax[0].parameter, 0.0, THE_TOL);
    assert!(aMax[0].is_maximum);
}

#[test]
fn ExtremaPC2d_HyperbolaTest__PointOnCurveHasZeroDistanceMinimum() {
    let aCurve = makeHyperbola(2.0, 1.0);
    let aU: f64 = 0.7;
    let mut anEvaluator = ExtremaPC2d::Hyperbola::new(aCurve);
    let aResult = anEvaluator.perform(elclib::value(aU, &aCurve), THE_TOL, MM);
    EXPECT_NEAR(aResult.min_square_distance(), 0.0, THE_TOL);
    EXPECT_NEAR(aResult[aResult.min_index()].parameter, aU, THE_TOL);
}

#[test]
fn ExtremaPC2d_HyperbolaTest__IndirectOrientationPreservesSignedParameters() {
    let aPosition =
        Axis22d::with_directions(Point2::new(1.0, 2.0), dir2(0.0, 1.0), dir2(1.0, 0.0)).unwrap();
    let aCurve = Hyperbola2::new(aPosition, 2.0, 1.0).unwrap();
    let mut anEvaluator = ExtremaPC2d::Hyperbola::new(aCurve);
    let aResult = anEvaluator.perform(Point2::new(1.0, 7.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 3);
    EXPECT_NEAR(aResult.min_square_distance(), 4.0, THE_TOL);
    EXPECT_NEAR(aResult.max_square_distance(), 9.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_HyperbolaTest__BoundedArcAddsEndpointMinima() {
    let mut anEvaluator = bounded((-0.5, 0.5));
    let aResult = anEvaluator.perform_with_endpoints(Point2::new(5.0, 0.0), THE_TOL, MM);
    assert_eq!(aResult.nb_ext(), 3);
    EXPECT_NEAR(aResult[aResult.max_index()].parameter, 0.0, THE_TOL);
    let aMinima = aResult.extrema.iter().filter(|e| e.is_minimum).count();
    assert_eq!(aMinima, 2);
}

#[test]
fn ExtremaPC2d_HyperbolaTest__ZeroAxesDegeneracies() {
    let mut aSinhLine = ExtremaPC2d::Hyperbola::new(makeHyperbola(0.0, 1.0));
    let aLineResult = aSinhLine.perform(Point2::new(3.0, 2.0), THE_TOL, MM);
    assert!(aLineResult.nb_ext() >= 1);
    EXPECT_NEAR(aLineResult.min_square_distance(), 9.0, THE_TOL);

    let mut aPoint = ExtremaPC2d::Hyperbola::new(makeHyperbola(0.0, 0.0));
    let aPointResult = aPoint.perform(Point2::new(3.0, 4.0), THE_TOL, MM);
    assert!(aPointResult.is_infinite());
    EXPECT_NEAR(aPointResult.infinite_square_distance, 25.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_HyperbolaTest__SingletonInvalidAndValueAccessor() {
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
    let expected: Point2 = elclib::value(-0.7, &makeHyperbola(2.0, 1.0));
    assert_eq!(aSingleton.value(-0.7).distance(expected), 0.0);
}
