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

//! Direct translation of ExtremaPC2d_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::extrema_pc2d::{
    self as ExtremaPC2d, CurveEvaluator, Domain1D, ExtremaResult, ExtremumResult, SearchMode,
    Status,
};
use rustycad_tkmath::gp::Point2;
use std::convert::Infallible;
use std::f64::consts::PI;
use std::marker::PhantomData;

struct LinearEvaluator2d;
impl CurveEvaluator<Point2> for LinearEvaluator2d {
    type Error = Infallible;
    fn evaluate(&self, theU: f64) -> Result<Point2, Infallible> {
        Ok(Point2::new(theU, 0.0))
    }
}

struct RapidEndpointEvaluator2d;
impl CurveEvaluator<Point2> for RapidEndpointEvaluator2d {
    type Error = Infallible;
    fn evaluate(&self, theU: f64) -> Result<Point2, Infallible> {
        Ok(Point2::new(1.0 + theU - 200.0 * theU * theU, 0.0))
    }
}

// The std::is_copy_constructible probe: the inherent method exists only for Clone types.
struct TraitProbe<T>(PhantomData<T>);
trait NotCloneFallback {
    fn is_clone(&self) -> bool {
        false
    }
}
impl<T> NotCloneFallback for TraitProbe<T> {}
#[allow(dead_code)] // Reached only for a Clone type, which the result is not.
impl<T: Clone> TraitProbe<T> {
    fn is_clone(&self) -> bool {
        true
    }
}

fn add(
    aResult: &mut ExtremaResult,
    theQuery: Point2,
    theDomain: Domain1D,
    theEvaluator: &impl CurveEvaluator<Point2, Error = Infallible>,
    theMode: SearchMode,
    theIsClosed: bool,
) {
    let Ok(()) = ExtremaPC2d::add_endpoint_extrema(
        aResult,
        &theQuery,
        &theDomain,
        theEvaluator,
        theMode,
        theIsClosed,
    );
}

#[test]
fn ExtremaPC2dTest__Result_DefaultClearAndTraits() {
    // static_assert(!std::is_copy_constructible_v<ExtremaPC2d::Result>) and
    // !std::is_copy_assignable_v: the Rust result does not implement Clone; moves are
    // the language default.
    assert!(!TraitProbe::<ExtremaResult>(PhantomData).is_clone());

    let mut aResult = ExtremaResult::default();
    assert_eq!(aResult.status, Status::NotDone);
    assert_eq!(aResult.nb_ext(), 0);
    assert_eq!(aResult.min_index(), ExtremaResult::INVALID_INDEX);
    assert_eq!(aResult.max_index(), ExtremaResult::INVALID_INDEX);
    assert!(aResult.min_square_distance().is_infinite());
    assert_eq!(aResult.max_square_distance(), 0.0);

    aResult.status = Status::OK;
    aResult.extrema.push(ExtremumResult::new(
        0.0,
        Point2::default(),
        9.0,
        true,
        false,
    ));
    aResult.extrema.push(ExtremumResult::new(
        1.0,
        Point2::default(),
        4.0,
        true,
        false,
    ));
    aResult.extrema.push(ExtremumResult::new(
        2.0,
        Point2::default(),
        9.0,
        false,
        true,
    ));
    assert_eq!(aResult.min_index(), 1);
    assert_eq!(aResult.max_index(), 0);
    aResult.clear();
    assert_eq!(aResult.status, Status::NotDone);
    assert_eq!(aResult.nb_ext(), 0);
}

#[test]
fn ExtremaPC2dTest__ToleranceAndFiniteParameterValidation() {
    assert!(ExtremaPC2d::is_valid_tolerance(1.0e-9));
    assert!(!ExtremaPC2d::is_valid_tolerance(0.0));
    assert!(!ExtremaPC2d::is_valid_tolerance(-1.0));
    assert!(!ExtremaPC2d::is_valid_tolerance(f64::INFINITY));
    assert!(!ExtremaPC2d::is_valid_tolerance(f64::NAN));
    assert!(ExtremaPC2d::is_finite_parameter(1.0));
    assert!(!ExtremaPC2d::is_finite_parameter(precision::infinite()));
    assert!(!ExtremaPC2d::is_finite_parameter(f64::INFINITY));
}

#[test]
fn ExtremaPC2dTest__ClosedPeriodicDomainValidation() {
    let aTol: f64 = 1.0e-10;
    let closed = |a: f64, b: f64, p: f64, t: f64| {
        ExtremaPC2d::is_closed_periodic_domain(&Domain1D::new(a, b), p, t)
    };
    assert!(closed(0.0, 2.0 * PI, 2.0 * PI, aTol));
    assert!(closed(7.0, 7.0 + 4.0 * PI, 2.0 * PI, aTol));
    assert!(!closed(0.0, PI, 2.0 * PI, aTol));
    assert!(!closed(1.0, 0.0, 2.0 * PI, aTol));
    assert!(!closed(0.0, 2.0 * PI, 0.0, aTol));
    assert!(!closed(0.0, 2.0 * PI, 2.0 * PI, -1.0));
}

#[test]
fn ExtremaPC2dTest__EndpointClassificationAndModes() {
    let anEvaluator = LinearEvaluator2d;
    let aQuery = Point2::new(-2.0, 3.0);
    let mut aResult = ExtremaResult::default();
    add(
        &mut aResult,
        aQuery,
        Domain1D::new(0.0, 10.0),
        &anEvaluator,
        SearchMode::MinMax,
        false,
    );
    assert_eq!(aResult.nb_ext(), 2);
    EXPECT_NEAR(aResult[0].parameter, 0.0, 1.0e-12);
    assert!(aResult[0].is_minimum);
    EXPECT_NEAR(aResult[1].parameter, 10.0, 1.0e-12);
    assert!(aResult[1].is_maximum);

    aResult.clear();
    add(
        &mut aResult,
        aQuery,
        Domain1D::new(0.0, 10.0),
        &anEvaluator,
        SearchMode::Min,
        false,
    );
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.0, 1.0e-12);
}

#[test]
fn ExtremaPC2dTest__SingletonAndClosedEndpointBehavior() {
    let anEvaluator = LinearEvaluator2d;
    let mut aResult = ExtremaResult::default();
    add(
        &mut aResult,
        Point2::new(5.0, 3.0),
        Domain1D::new(2.0, 2.0),
        &anEvaluator,
        SearchMode::MinMax,
        false,
    );
    assert_eq!(aResult.nb_ext(), 1);
    assert!(aResult[0].is_minimum);
    assert!(aResult[0].is_maximum);

    aResult.clear();
    add(
        &mut aResult,
        Point2::default(),
        Domain1D::new(0.0, 1.0),
        &anEvaluator,
        SearchMode::MinMax,
        true,
    );
    assert_eq!(aResult.nb_ext(), 0);
}

#[test]
fn ExtremaPC2dTest__EndpointClassificationUsesLocalBehavior() {
    let anEvaluator = RapidEndpointEvaluator2d;
    let mut aResult = ExtremaResult::default();
    add(
        &mut aResult,
        Point2::default(),
        Domain1D::new(0.0, 1.0),
        &anEvaluator,
        SearchMode::Min,
        false,
    );
    assert_eq!(aResult.nb_ext(), 1);
    EXPECT_NEAR(aResult[0].parameter, 0.0, precision::p_confusion());
    assert!(aResult[0].is_minimum);
}
