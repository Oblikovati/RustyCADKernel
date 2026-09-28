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

//! Direct translation of ExtremaPC2d_Line_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, dir2};
use rustycad_tkgeombase::extrema_pc2d::{self as ExtremaPC2d, Domain1D, SearchMode, Status};
use rustycad_tkmath::gp::{Line2, Point2};

const THE_TOL: f64 = 1.0e-10;

#[test]
fn ExtremaPC2d_LineTest__UnboundedProjectionIsExact() {
    let mut anEvaluator = ExtremaPC2d::Line::new(Line2::new(Point2::new(1.0, 2.0), dir2(1.0, 0.0)));
    let aResult = anEvaluator.perform(Point2::new(4.0, 6.0), THE_TOL, SearchMode::MinMax);
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 3.0, THE_TOL);
    EXPECT_NEAR(aResult[0].square_distance, 16.0, THE_TOL);
    assert!(aResult[0].is_minimum);
}

#[test]
fn ExtremaPC2d_LineTest__ReversedAndDiagonalDirectionsPreserveParameter() {
    let mut aReversed = ExtremaPC2d::Line::new(Line2::new(Point2::new(1.0, 2.0), dir2(-1.0, 0.0)));
    EXPECT_NEAR(
        aReversed.perform(Point2::new(-2.0, 5.0), THE_TOL, SearchMode::MinMax)[0].parameter,
        3.0,
        THE_TOL,
    );

    let mut aDiagonal = ExtremaPC2d::Line::new(Line2::new(Point2::default(), dir2(1.0, 1.0)));
    let aResult = aDiagonal.perform(Point2::new(5.0, 5.0), THE_TOL, SearchMode::MinMax);
    EXPECT_NEAR(aResult[0].parameter, 5.0 * 2.0_f64.sqrt(), THE_TOL);
    EXPECT_NEAR(aResult[0].square_distance, 0.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_LineTest__SearchModesFilterInteriorProjection() {
    let mut anEvaluator = ExtremaPC2d::Line::new(Line2::new(Point2::default(), dir2(1.0, 0.0)));
    assert_eq!(
        anEvaluator
            .perform(Point2::new(4.0, 3.0), THE_TOL, SearchMode::Min)
            .nb_ext(),
        1
    );
    let aMax = anEvaluator.perform(Point2::new(4.0, 3.0), THE_TOL, SearchMode::Max);
    assert_eq!(aMax.status, Status::NoSolution);
    assert_eq!(aMax.nb_ext(), 0);
}

#[test]
fn ExtremaPC2d_LineTest__BoundedProjectionAndEndpointsAreClassified() {
    let mut anEvaluator = ExtremaPC2d::Line::with_domain(
        Line2::new(Point2::default(), dir2(1.0, 0.0)),
        Domain1D::new(0.0, 10.0),
    );
    let aResult =
        anEvaluator.perform_with_endpoints(Point2::new(4.0, 3.0), THE_TOL, SearchMode::MinMax);
    assert_eq!(aResult.nb_ext(), 3);
    EXPECT_NEAR(aResult[aResult.min_index()].parameter, 4.0, THE_TOL);
    EXPECT_NEAR(aResult[aResult.min_index()].square_distance, 9.0, THE_TOL);
    EXPECT_NEAR(aResult[aResult.max_index()].parameter, 10.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_LineTest__ProjectionOutsideDomainUsesEndpointsOnlyWhenRequested() {
    let mut anEvaluator = ExtremaPC2d::Line::with_domain(
        Line2::new(Point2::default(), dir2(1.0, 0.0)),
        Domain1D::new(0.0, 10.0),
    );
    assert_eq!(
        anEvaluator
            .perform(Point2::new(-2.0, 3.0), THE_TOL, SearchMode::MinMax)
            .nb_ext(),
        0
    );
    let aResult =
        anEvaluator.perform_with_endpoints(Point2::new(-2.0, 3.0), THE_TOL, SearchMode::MinMax);
    assert_eq!(aResult.nb_ext(), 2);
    EXPECT_NEAR(aResult[aResult.min_index()].parameter, 0.0, THE_TOL);
    EXPECT_NEAR(aResult[aResult.min_index()].square_distance, 13.0, THE_TOL);
    EXPECT_NEAR(aResult[aResult.max_index()].parameter, 10.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_LineTest__BoundaryProjectionIsNotDuplicated() {
    let mut anEvaluator = ExtremaPC2d::Line::with_domain(
        Line2::new(Point2::default(), dir2(1.0, 0.0)),
        Domain1D::new(0.0, 10.0),
    );
    let aResult =
        anEvaluator.perform_with_endpoints(Point2::new(0.0, 3.0), THE_TOL, SearchMode::MinMax);
    assert_eq!(aResult.nb_ext(), 2);
    EXPECT_NEAR(aResult[aResult.min_index()].parameter, 0.0, THE_TOL);
    EXPECT_NEAR(aResult[aResult.max_index()].parameter, 10.0, THE_TOL);
}

#[test]
fn ExtremaPC2d_LineTest__SingletonDomainHonorsModes() {
    let mut anEvaluator = ExtremaPC2d::Line::with_domain(
        Line2::new(Point2::default(), dir2(1.0, 0.0)),
        Domain1D::new(2.0, 2.0),
    );
    let aBoth =
        anEvaluator.perform_with_endpoints(Point2::new(5.0, 3.0), THE_TOL, SearchMode::MinMax);
    assert_eq!(aBoth.nb_ext(), 1);
    assert!(aBoth[0].is_minimum);
    assert!(aBoth[0].is_maximum);
    let aMax = anEvaluator.perform_with_endpoints(Point2::new(5.0, 3.0), THE_TOL, SearchMode::Max);
    assert_eq!(aMax.nb_ext(), 1);
    assert!(!aMax[0].is_minimum);
    assert!(aMax[0].is_maximum);
}

#[test]
fn ExtremaPC2d_LineTest__InvalidInputAndAccessors() {
    let aLine = Line2::new(Point2::new(1.0, 2.0), dir2(0.0, 1.0));
    let mut anEvaluator = ExtremaPC2d::Line::with_domain(aLine, Domain1D::new(3.0, 1.0));
    assert_eq!(
        anEvaluator
            .perform(Point2::default(), THE_TOL, SearchMode::MinMax)
            .status,
        Status::InvalidInput
    );
    assert_eq!(
        anEvaluator
            .perform(Point2::default(), 0.0, SearchMode::MinMax)
            .status,
        Status::InvalidInput
    );
    assert_eq!(
        anEvaluator.line().location().distance(aLine.location()),
        0.0
    );
    assert_eq!(anEvaluator.value(2.0).distance(Point2::new(1.0, 4.0)), 0.0);
}
