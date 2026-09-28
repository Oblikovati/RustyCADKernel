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

//! Direct math_Matrix_Test.cxx translation at pinned OCCT 3d097a0.
// Keep the original hand-written multiplication expectations verbatim.
#![allow(non_snake_case, unused_mut, clippy::identity_op, clippy::erasing_op)]
use rustycad_tkmath::math::{Matrix, Vector};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
fn check_matrices_equal(left: &Matrix<'_>, right: &Matrix<'_>) {
    assert_eq!(
        (
            left.row_number(),
            left.col_number(),
            left.lower_row(),
            left.lower_col()
        ),
        (
            right.row_number(),
            right.col_number(),
            right.lower_row(),
            right.lower_col()
        )
    );
    for row in left.lower_row()..=left.upper_row() {
        for col in left.lower_col()..=left.upper_col() {
            assert_near(left[(row, col)], right[(row, col)], 1e-7);
        }
    }
}
include!("math_matrix/suites_0.rs");
include!("math_matrix/suites_1.rs");
include!("math_matrix/suites_2.rs");
include!("math_matrix/suites_3.rs");
include!("math_matrix/suites_4.rs");
include!("math_matrix/suites_5.rs");
