// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Direct translation of PLib_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, unused_mut, clippy::assertions_on_constants)]
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::plib;

// Helper function for comparing points with tolerance
fn checkPointsEqual(thePnt1: &Point3, thePnt2: &Point3, theTolerance: f64) {
    assert!(
        (thePnt1.x() - thePnt2.x()).abs() <= theTolerance,
        "X coordinates differ"
    );
    assert!(
        (thePnt1.y() - thePnt2.y()).abs() <= theTolerance,
        "Y coordinates differ"
    );
    assert!(
        (thePnt1.z() - thePnt2.z()).abs() <= theTolerance,
        "Z coordinates differ"
    );
}

// Helper function for comparing 2D points with tolerance
fn checkPoint2dEqual(thePnt1: &Point2, thePnt2: &Point2, theTolerance: f64) {
    assert!(
        (thePnt1.x() - thePnt2.x()).abs() <= theTolerance,
        "X coordinates differ"
    );
    assert!(
        (thePnt1.y() - thePnt2.y()).abs() <= theTolerance,
        "Y coordinates differ"
    );
}

fn reals(lower: i32, upper: i32) -> Array1<'static, f64> {
    Array1::with_bounds(lower, upper).unwrap()
}

// Tests for basic utility functions
#[test]
fn PLibTest__NoWeightsPointers() {
    // Test that NoWeights() returns NULL as expected
    assert!(plib::no_weights().is_none());
    assert!(plib::no_weights2().is_none());
}

// Tests for 3D pole conversion functions
#[test]
fn PLibTest__SetGetPoles3D() {
    // Create test data
    let mut aPoles: Array1<Point3> = Array1::with_bounds(1, 3).unwrap();
    aPoles[1] = Point3::new(1.0, 2.0, 3.0);
    aPoles[2] = Point3::new(4.0, 5.0, 6.0);
    aPoles[3] = Point3::new(7.0, 8.0, 9.0);

    // Test SetPoles and GetPoles without weights
    let mut aFP = reals(1, 9); // 3 poles * 3 coordinates
    plib::set_poles(&aPoles, &mut aFP).unwrap();

    let mut aResultPoles: Array1<Point3> = Array1::with_bounds(1, 3).unwrap();
    plib::get_poles(&aFP, &mut aResultPoles).unwrap();

    // Verify results
    for i in 1..=3 {
        checkPointsEqual(&aPoles[i], &aResultPoles[i], precision::confusion());
    }
}

#[test]
fn PLibTest__SetGetPoles3DWithWeights() {
    // Create test data
    let mut aPoles: Array1<Point3> = Array1::with_bounds(1, 2).unwrap();
    aPoles[1] = Point3::new(1.0, 2.0, 3.0);
    aPoles[2] = Point3::new(4.0, 5.0, 6.0);

    let mut aWeights = reals(1, 2);
    aWeights[1] = 0.5;
    aWeights[2] = 2.0;

    // Test SetPoles and GetPoles with weights
    let mut aFP = reals(1, 8); // 2 poles * (3 coords + 1 weight)
    plib::set_rational_poles(&aPoles, &aWeights, &mut aFP).unwrap();

    let mut aResultPoles: Array1<Point3> = Array1::with_bounds(1, 2).unwrap();
    let mut aResultWeights = reals(1, 2);
    plib::get_rational_poles(&aFP, &mut aResultPoles, &mut aResultWeights).unwrap();

    // Verify results
    for i in 1..=2 {
        checkPointsEqual(&aPoles[i], &aResultPoles[i], precision::confusion());
        assert!((aWeights[i] - aResultWeights[i]).abs() <= precision::confusion());
    }
}

// Tests for 2D pole conversion functions
#[test]
fn PLibTest__SetGetPoles2D() {
    // Create test data
    let mut aPoles: Array1<Point2> = Array1::with_bounds(1, 3).unwrap();
    aPoles[1] = Point2::new(1.0, 2.0);
    aPoles[2] = Point2::new(3.0, 4.0);
    aPoles[3] = Point2::new(5.0, 6.0);

    // Test SetPoles and GetPoles without weights
    let mut aFP = reals(1, 6); // 3 poles * 2 coordinates
    plib::set_poles(&aPoles, &mut aFP).unwrap();

    let mut aResultPoles: Array1<Point2> = Array1::with_bounds(1, 3).unwrap();
    plib::get_poles(&aFP, &mut aResultPoles).unwrap();

    // Verify results
    for i in 1..=3 {
        checkPoint2dEqual(&aPoles[i], &aResultPoles[i], precision::confusion());
    }
}

#[test]
fn PLibTest__SetGetPoles2DWithWeights() {
    // Create test data
    let mut aPoles: Array1<Point2> = Array1::with_bounds(1, 2).unwrap();
    aPoles[1] = Point2::new(1.0, 2.0);
    aPoles[2] = Point2::new(3.0, 4.0);

    let mut aWeights = reals(1, 2);
    aWeights[1] = 0.8;
    aWeights[2] = 1.5;

    // Test SetPoles and GetPoles with weights
    let mut aFP = reals(1, 6); // 2 poles * (2 coords + 1 weight)
    plib::set_rational_poles(&aPoles, &aWeights, &mut aFP).unwrap();

    let mut aResultPoles: Array1<Point2> = Array1::with_bounds(1, 2).unwrap();
    let mut aResultWeights = reals(1, 2);
    plib::get_rational_poles(&aFP, &mut aResultPoles, &mut aResultWeights).unwrap();

    // Verify results
    for i in 1..=2 {
        checkPoint2dEqual(&aPoles[i], &aResultPoles[i], precision::confusion());
        assert!((aWeights[i] - aResultWeights[i]).abs() <= precision::confusion());
    }
}

// Test for zero weights handling (safety test)
#[test]
fn PLibTest__ZeroWeightsHandling() {
    // Create test data with zero weight - this should not crash
    let mut aPoles: Array1<Point2> = Array1::with_bounds(1, 2).unwrap();
    aPoles[1] = Point2::new(1.0, 2.0);
    aPoles[2] = Point2::new(3.0, 4.0);

    let mut aWeights = reals(1, 2);
    aWeights[1] = 1.0;
    aWeights[2] = 0.0; // Zero weight - potential division by zero

    let mut aFP = reals(1, 6);
    plib::set_rational_poles(&aPoles, &aWeights, &mut aFP).unwrap();

    // This test should complete without crashing
    // The behavior with zero weights may vary, but it shouldn't crash
    assert!(true); // We mainly test that no crash occurs
}

include!("plib/suites_0.rs");
include!("plib/suites_1.rs");
