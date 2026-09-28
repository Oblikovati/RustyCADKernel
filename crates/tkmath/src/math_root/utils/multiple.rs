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

//! Validation, bookkeeping and evaluation helpers of MathRoot_MultipleUtils.hxx.
use super::is_valid_bounds;
use crate::bvh::{std_max, std_min};
use crate::math::{Function, FunctionWithDerivative};
use crate::math_root::{MultipleConfig, MultipleResult};
use crate::math_utils::Status;

/// Newton steps allowed to polish a root already isolated by Brent's method
/// (`THE_MAX_NEWTON_POLISH_ITERATIONS`).
pub const MAX_NEWTON_POLISH_ITERATIONS: u32 = 5;

/// Relative interval scale enforcing the established minimum X tolerance
/// (`THE_MULTIPLE_RELATIVE_INTERVAL_TOLERANCE`).
pub const MULTIPLE_RELATIVE_INTERVAL_TOLERANCE: f64 = 1.0e-10;

/// Reports whether a multiple-root configuration can be sampled safely
/// (`IsValidMultipleInput`); `is_valid_multiple_input(0.0, 1.0, &MultipleConfig::default())`.
pub fn is_valid_multiple_input(lower: f64, upper: f64, config: &MultipleConfig) -> bool {
    is_valid_bounds(lower, upper)
        && config.nb_samples >= 2
        && config.max_iterations > 0
        && config.x_tolerance.is_finite()
        && config.x_tolerance > 0.0
        && config.f_tolerance.is_finite()
        && config.f_tolerance >= 0.0
        && config.null_tolerance.is_finite()
        && config.null_tolerance >= 0.0
        && config.offset.is_finite()
}

/// Stable in-place insertion sort of roots and their values by ascending root
/// (`SortRoots`); `sort_roots(&mut result)`.
pub fn sort_roots(result: &mut MultipleResult) {
    for i in 1..result.roots.len() {
        let key_root: f64 = result.roots[i];
        let key_value: f64 = result.values[i];
        let mut j: usize = i;
        while j > 0 && result.roots[j - 1] > key_root {
            result.roots[j] = result.roots[j - 1];
            result.values[j] = result.values[j - 1];
            j -= 1;
        }
        result.roots[j] = key_root;
        result.values[j] = key_value;
    }
}

/// Appends a root unless one closer than `eps_x` is already stored (`AddRoot`);
/// `add_root(&mut result, 1e-10, 0.5, 0.0)`.
pub fn add_root(result: &mut MultipleResult, eps_x: f64, root: f64, value: f64) {
    // A NaN distance never compares below eps_x, so NaN roots are always appended as natively.
    if result.roots.iter().any(|old| (root - old).abs() < eps_x) {
        return;
    }
    result.roots.push(root);
    result.values.push(value);
}

/// Smallest X tolerance compatible with the interval magnitude (`EffectiveXTolerance`);
/// `effective_x_tolerance(0.0, 1.0, 1e-12)` is `1e-10`.
pub fn effective_x_tolerance(lower: f64, upper: f64, x_tolerance: f64) -> f64 {
    // std::max/std::min keep the first operand unless the comparison holds, so NaN operands
    // propagate exactly as natively (f64::max would drop them).
    let max_magnitude: f64 = std_max(lower.abs(), upper.abs());
    let min_magnitude: f64 = std_min(lower.abs(), upper.abs());
    if max_magnitude == 0.0 {
        return x_tolerance;
    }
    let min_eps_x: f64 = MULTIPLE_RELATIVE_INTERVAL_TOLERANCE
        * max_magnitude
        * (1.0 + min_magnitude / max_magnitude);
    std_max(x_tolerance, min_eps_x)
}

/// Evaluates the original function value (`EvaluateValue`); returns `Ok`, `CallbackError`
/// or `NumericalError`; `evaluate_value(&mut f, 0.5, &mut value)`.
pub fn evaluate_value(function: &mut impl Function, x: f64, value: &mut f64) -> Status {
    if !function.value(x, value) {
        return Status::CallbackError;
    }
    finite_status(*value)
}

/// Evaluates `f(x) - offset`, requiring both the raw and shifted values to be finite
/// (`EvaluateShiftedValue`); `evaluate_shifted_value(&mut f, 0.5, 1.0, &mut value)`.
pub fn evaluate_shifted_value(
    function: &mut impl Function,
    x: f64,
    offset: f64,
    value: &mut f64,
) -> Status {
    if !function.value(x, value) {
        return Status::CallbackError;
    }
    if !value.is_finite() {
        return Status::NumericalError;
    }
    *value -= offset;
    finite_status(*value)
}

/// Evaluates `f(x) - offset` and `f'(x)` (`EvaluateShiftedValues`);
/// `evaluate_shifted_values(&mut f, 0.5, 0.0, &mut value, &mut derivative)`.
pub fn evaluate_shifted_values(
    function: &mut impl FunctionWithDerivative,
    x: f64,
    offset: f64,
    value: &mut f64,
    derivative: &mut f64,
) -> Status {
    if !function.values(x, value, derivative) {
        return Status::CallbackError;
    }
    if !value.is_finite() || !derivative.is_finite() {
        return Status::NumericalError;
    }
    *value -= offset;
    finite_status(*value)
}

fn finite_status(value: f64) -> Status {
    if value.is_finite() {
        Status::Ok
    } else {
        Status::NumericalError
    }
}
