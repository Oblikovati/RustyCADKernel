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

//! GoogleTest assertion semantics shared by the translated TKG2d suites.
#![allow(non_snake_case, dead_code)]

/// GoogleTest `EXPECT_DOUBLE_EQ`: at most four ULPs apart, never NaN.
pub fn EXPECT_DOUBLE_EQ(a: f64, b: f64) {
    assert!(
        !a.is_nan() && !b.is_nan(),
        "EXPECT_DOUBLE_EQ on NaN: {a} vs {b}"
    );
    assert!(
        ordered_bits(a).abs_diff(ordered_bits(b)) <= 4,
        "EXPECT_DOUBLE_EQ: {a} != {b}"
    );
}

/// GoogleTest `EXPECT_NEAR`: `|a - b| <= tolerance`.
pub fn EXPECT_NEAR(a: f64, b: f64, tolerance: f64) {
    assert!(
        (a - b).abs() <= tolerance,
        "EXPECT_NEAR: |{a} - {b}| > {tolerance}"
    );
}

// Maps doubles to integers whose order matches the numeric order (GoogleTest's
// sign-and-magnitude to biased conversion).
fn ordered_bits(value: f64) -> u64 {
    let bits: u64 = value.to_bits();
    if bits >> 63 != 0 {
        (!bits).wrapping_add(1)
    } else {
        bits | (1 << 63)
    }
}

/// GoogleTest `EXPECT_THROW(statement, Type)`: the statement fails with the native
/// exception `exception`.
pub fn EXPECT_THROW<T>(result: Result<T, rustycad_tkernel::standard::Failure>, exception: &str) {
    match result {
        Ok(_) => panic!("EXPECT_THROW: no exception; expected {exception}"),
        Err(error) => assert_eq!(
            error.exception_type(),
            exception,
            "EXPECT_THROW: wrong exception; expected {exception}"
        ),
    }
}
