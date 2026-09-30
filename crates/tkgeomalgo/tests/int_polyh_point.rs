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

//! Direct translation of IntPolyh_Point_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkgeomalgo::int_polyh::IntPolyhPoint;

/// `EXPECT_DOUBLE_EQ`: equal within 4 units in the last place.
fn expect_double_eq(a: f64, b: f64) {
    let ulps = (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs();
    assert!(a == b || ulps <= 4, "{a} != {b}");
}

#[test]
fn IntPolyh_Point__DefaultConstructor_AllZero() {
    let a_pt = IntPolyhPoint::default();
    expect_double_eq(a_pt.x(), 0.0);
    expect_double_eq(a_pt.y(), 0.0);
    expect_double_eq(a_pt.z(), 0.0);
    expect_double_eq(a_pt.u(), 0.0);
    expect_double_eq(a_pt.v(), 0.0);
}

#[test]
fn IntPolyh_Point__Divide_NormalDivisor_CorrectResult() {
    let a_pt = IntPolyhPoint::new(10.0, 20.0, 30.0, 0.5, 0.8);
    let a_res = a_pt.divide(2.0);

    expect_double_eq(a_res.x(), 5.0);
    expect_double_eq(a_res.y(), 10.0);
    expect_double_eq(a_res.z(), 15.0);
    expect_double_eq(a_res.u(), 0.25);
    expect_double_eq(a_res.v(), 0.4);
}

// Regression test for bug #18: Divide by zero must not crash or print to stdout;
// returns default-initialized (zero) point instead.
#[test]
fn IntPolyh_Point__Divide_ZeroDivisor_ReturnsDefaultPoint() {
    let a_pt = IntPolyhPoint::new(10.0, 20.0, 30.0, 0.5, 0.8);
    let a_res = a_pt.divide(0.0);

    expect_double_eq(a_res.x(), 0.0);
    expect_double_eq(a_res.y(), 0.0);
    expect_double_eq(a_res.z(), 0.0);
    expect_double_eq(a_res.u(), 0.0);
    expect_double_eq(a_res.v(), 0.0);
}

// Near-zero divisor (below Precision::Computational) returns default point.
#[test]
fn IntPolyh_Point__Divide_NearZeroDivisor_ReturnsDefaultPoint() {
    let a_pt = IntPolyhPoint::new(1.0, 2.0, 3.0, 0.1, 0.2);
    let a_res = a_pt.divide(1.0e-20);

    expect_double_eq(a_res.x(), 0.0);
    expect_double_eq(a_res.y(), 0.0);
    expect_double_eq(a_res.z(), 0.0);
}

// Negative divisor works correctly.
#[test]
fn IntPolyh_Point__Divide_NegativeDivisor_CorrectResult() {
    let a_pt = IntPolyhPoint::new(6.0, -9.0, 12.0, 0.3, 0.6);
    let a_res = a_pt.divide(-3.0);

    expect_double_eq(a_res.x(), -2.0);
    expect_double_eq(a_res.y(), 3.0);
    expect_double_eq(a_res.z(), -4.0);
}

#[test]
fn IntPolyh_Point__Add_CorrectResult() {
    let a_p1 = IntPolyhPoint::new(1.0, 2.0, 3.0, 0.1, 0.2);
    let a_p2 = IntPolyhPoint::new(4.0, 5.0, 6.0, 0.3, 0.4);
    let a_res = a_p1.add(&a_p2);

    expect_double_eq(a_res.x(), 5.0);
    expect_double_eq(a_res.y(), 7.0);
    expect_double_eq(a_res.z(), 9.0);
}

#[test]
fn IntPolyh_Point__Sub_CorrectResult() {
    let a_p1 = IntPolyhPoint::new(5.0, 7.0, 9.0, 0.5, 0.8);
    let a_p2 = IntPolyhPoint::new(1.0, 2.0, 3.0, 0.1, 0.2);
    let a_res = a_p1.sub(&a_p2);

    expect_double_eq(a_res.x(), 4.0);
    expect_double_eq(a_res.y(), 5.0);
    expect_double_eq(a_res.z(), 6.0);
}

#[test]
fn IntPolyh_Point__SquareModulus_CorrectResult() {
    let a_pt = IntPolyhPoint::new(1.0, 2.0, 3.0, 0.0, 0.0);
    expect_double_eq(a_pt.square_modulus(), 14.0);
}

#[test]
fn IntPolyh_Point__SquareDistance_SamePoint_Zero() {
    let a_pt = IntPolyhPoint::new(3.0, 4.0, 5.0, 0.0, 0.0);
    expect_double_eq(a_pt.square_distance(&a_pt), 0.0);
}

#[test]
fn IntPolyh_Point__Dot_OrthogonalVectors_Zero() {
    let a_p1 = IntPolyhPoint::new(1.0, 0.0, 0.0, 0.0, 0.0);
    let a_p2 = IntPolyhPoint::new(0.0, 1.0, 0.0, 0.0, 0.0);
    expect_double_eq(a_p1.dot(&a_p2), 0.0);
}
