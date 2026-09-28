// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! math_Recipes: the Numerical-Recipes kernels shared by math_Gauss, math_SVD,
//! math_Jacobi, math_GaussLeastSquare and math_NewtonFunctionSetRoot.
//!
//! The kernels return the native `math_Status_*` integer codes. `math_Recipes.cxx`
//! disables range checking, so natively an array that does not cover the indices a
//! kernel walks (1-based matrices, `indx` entries into a skyline) is undefined
//! behaviour; here such input is refused with a `Standard_OutOfRange` or
//! `Standard_DimensionError` failure before or at the first invalid access.
use super::{IntegerVector, Matrix, Vector, VectorBase, VectorItem};
use rustycad_tkernel::standard::Failure;
mod dactcl;
mod jacobi;
mod jacobi_rotation;
mod lu;
mod svd;
mod svd_accumulate;
mod svd_qr;
mod svd_reduce;
pub use dactcl::{dactcl_decompose, dactcl_solve};
pub use jacobi::jacobi;
pub use lu::{
    lu_decompose, lu_decompose_with_progress, lu_decompose_with_scales,
    lu_decompose_with_scales_and_progress, lu_invert, lu_solve,
};
pub use svd::{svd_decompose, svd_decompose_with_work, svd_solve};

/// `math_Status_UserAborted`: a progress scope stopped the decomposition.
pub const STATUS_USER_ABORTED: i32 = -1;
/// `math_Status_OK`.
pub const STATUS_OK: i32 = 0;
/// `math_Status_SingularMatrix`: a pivot is not above the threshold.
pub const STATUS_SINGULAR_MATRIX: i32 = 1;
/// `math_Status_ArgumentError`.
pub const STATUS_ARGUMENT_ERROR: i32 = 2;
/// `math_Status_NoConvergence`: an iteration limit was reached.
pub const STATUS_NO_CONVERGENCE: i32 = 3;

/// Refuses a matrix whose bounds do not contain rows `1..=rows` and columns `1..=cols`.
fn require_matrix(matrix: &Matrix<'_>, rows: i32, cols: i32, kernel: &str) -> Result<(), Failure> {
    let covers = matrix.lower_row() <= 1.min(rows)
        && matrix.upper_row() >= rows
        && matrix.lower_col() <= 1.min(cols)
        && matrix.upper_col() >= cols;
    if covers {
        return Ok(());
    }
    Err(out_of_range(&format!(
        "{kernel}: matrix rows {}..{} and columns {}..{} must contain 1..{rows} x 1..{cols}",
        matrix.lower_row(),
        matrix.upper_row(),
        matrix.lower_col(),
        matrix.upper_col()
    )))
}
/// Refuses a vector whose bounds do not contain the indices `1..=length`.
fn require_vector<T: VectorItem>(
    vector: &VectorBase<'_, T>,
    length: i32,
    kernel: &str,
) -> Result<(), Failure> {
    if vector.lower() <= 1.min(length) && vector.upper() >= length {
        return Ok(());
    }
    Err(out_of_range(&format!(
        "{kernel}: vector bounds {}..{} must contain 1..{length}",
        vector.lower(),
        vector.upper()
    )))
}
fn out_of_range(message: &str) -> Failure {
    Failure::with_type("Standard_OutOfRange", message, "")
}
/// Shared shape of the square kernels: `a` is `n x n` from 1 and `indx` covers `1..n`.
fn require_square(a: &Matrix<'_>, indx: &IntegerVector<'_>, kernel: &str) -> Result<i32, Failure> {
    let n = a.row_number();
    require_matrix(a, n, n, kernel)?;
    require_vector(indx, n, kernel)?;
    Ok(n)
}
/// Reads a declared vector index, raising `Standard_OutOfRange` with the index.
fn checked<T: VectorItem>(vector: &VectorBase<'_, T>, index: i32) -> Result<T, Failure> {
    vector.value(index).copied().map_err(|_| {
        out_of_range(&format!(
            "index {index} outside {}..{}",
            vector.lower(),
            vector.upper()
        ))
    })
}
/// Mutates a declared vector index, raising `Standard_OutOfRange` with the index.
fn checked_mut<'v, T: VectorItem>(
    vector: &'v mut VectorBase<'_, T>,
    index: i32,
) -> Result<&'v mut T, Failure> {
    let (lower, upper) = (vector.lower(), vector.upper());
    vector
        .change_value(index)
        .map_err(|_| out_of_range(&format!("index {index} outside {lower}..{upper}")))
}
