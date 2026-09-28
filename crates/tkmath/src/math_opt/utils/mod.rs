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

//! `MathOpt::Utils` helpers from MathOpt_Utils.hxx, MathOpt_Newton.hxx and MathOpt_FRPR.hxx
//! at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//!
//! The C++ namespace is public, header-only API; it maps to this public module. Overloads
//! that differ by argument type get distinct names (`is_finite_vector`/`is_finite_matrix`,
//! `scalar_value_status`/`value_status`).
use crate::math::{Matrix, Vector};

mod bounded;
mod checks;
mod evaluation;
mod hessian;
mod line;
mod stationary;
pub use bounded::BoundedFunction;
pub use checks::{is_finite_matrix, is_finite_vector, is_valid_bounds, is_valid_config, norm};
pub use evaluation::{
    CheckedFunction, checked_gradient, checked_hessian, checked_value, gradient_status,
    hessian_status, scalar_value_status, value_status,
};
pub use hessian::regularize_hessian;
pub(crate) use line::failure_after_fallback;
pub use line::{
    backtrack, is_line_search_evaluation_error, line_search_failure_status,
    line_search_fallback_failure_status,
};
pub use stationary::is_stationary;
pub(crate) use stationary::stationary_status;

/// Wolfe curvature constant of the FRPR line search; `MathOpt::Utils::THE_FRPR_WOLFE_CURVATURE`.
///
/// ```
/// assert_eq!(rustycad_tkmath::math_opt::utils::FRPR_WOLFE_CURVATURE, 0.1);
/// ```
pub const FRPR_WOLFE_CURVATURE: f64 = 0.1;

/// Copies a vector into zero-based storage, as native `math_Vector` copies keep only offsets.
pub(crate) fn zero_based(point: &Vector<'_>) -> Vector<'static> {
    let mut result: Vector<'static> = Vector::with_size(point.length() as usize);
    result.initialized(point).unwrap();
    result
}

/// Zero-based identity matrix of order `n`, the initial BFGS/Powell matrix.
pub(crate) fn identity(n: usize) -> Matrix<'static> {
    let mut matrix: Matrix<'static> =
        Matrix::filled(0, n as i32 - 1, 0, n as i32 - 1, 0.0).unwrap();
    for index in 0..n {
        *matrix.change_at(index, index).unwrap() = 1.0;
    }
    matrix
}

/// Largest absolute component difference, NaN-ignoring like the native `<` update.
pub(crate) fn max_difference(a: &Vector<'_>, b: &Vector<'_>) -> f64 {
    let mut maximum: f64 = 0.0;
    for (left, right) in a.values().iter().zip(b.values()) {
        let diff: f64 = (left - right).abs();
        if maximum < diff {
            maximum = diff;
        }
    }
    maximum
}
