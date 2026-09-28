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

//! Direct math_SVD_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::math::{Matrix, Svd, Vector};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
fn create_well_conditioned_matrix() -> Matrix<'static> {
    let mut matrix: Matrix<'static> = Matrix::filled(1, 3, 1, 3, 0.0).unwrap();
    matrix.set_diag(2.0).unwrap();
    matrix[(1, 2)] = 1.0;
    matrix[(2, 1)] = 1.0;
    matrix[(2, 3)] = 1.0;
    matrix[(3, 2)] = 1.0;
    matrix
}
fn check_solution(matrix: &Matrix<'_>, solution: &Vector<'_>, rhs: &Vector<'_>, tolerance: f64) {
    assert_eq!(matrix.col_number(), solution.length());
    assert_eq!(matrix.row_number(), rhs.length());
    let mut result: Vector<'static> = Vector::with_bounds(rhs.lower(), rhs.upper()).unwrap();
    for row in matrix.lower_row()..=matrix.upper_row() {
        let mut sum: f64 = 0.0;
        for col in matrix.lower_col()..=matrix.upper_col() {
            sum += matrix[(row, col)] * solution[col - matrix.lower_col() + solution.lower()];
        }
        result[row - matrix.lower_row() + rhs.lower()] = sum;
    }
    for index in rhs.lower()..=rhs.upper() {
        assert_near(result[index], rhs[index], tolerance);
    }
}
include!("math_svd/suites_0.rs");
include!("math_svd/suites_1.rs");
include!("math_svd/suites_2.rs");
include!("math_svd/suites_3.rs");
include!("math_svd/suites_4.rs");
