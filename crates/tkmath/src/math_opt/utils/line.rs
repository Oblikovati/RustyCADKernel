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

//! Line-search status mapping and the checked Armijo backtrack of `MathOpt::Utils`.
use super::{CheckedFunction, is_finite_vector};
use crate::math::{MultipleVarFunction, Vector};
use crate::math_utils::{LineSearchResult, Status, armijo_backtrack};

fn is_evaluation_error(status: Status) -> bool {
    matches!(status, Status::CallbackError | Status::NumericalError)
}

/// True when a line search stopped on a callback or numerical failure;
/// `MathOpt::Utils::IsLineSearchEvaluationError`.
///
/// ```
/// use rustycad_tkmath::{math_opt::utils::is_line_search_evaluation_error, math_utils::*};
/// let mut result = LineSearchResult::default();
/// result.status = Status::CallbackError;
/// assert!(is_line_search_evaluation_error(&result));
/// ```
pub fn is_line_search_evaluation_error(result: &LineSearchResult) -> bool {
    is_evaluation_error(result.status)
}

/// Solver status for a failed line search: its evaluation error, else `NotConverged`;
/// `MathOpt::Utils::LineSearchFailureStatus(const LineSearchResult&)`.
///
/// ```
/// use rustycad_tkmath::{math_opt::utils::line_search_failure_status, math_utils::*};
/// let result = LineSearchResult::default();
/// assert_eq!(line_search_failure_status(&result), Status::NotConverged);
/// ```
pub fn line_search_failure_status(result: &LineSearchResult) -> Status {
    if is_evaluation_error(result.status) {
        result.status
    } else {
        Status::NotConverged
    }
}

/// Solver status after a failed initial search and a failed fallback: the fallback's
/// evaluation error wins, then the initial one's, else `NotConverged`;
/// `MathOpt::Utils::LineSearchFailureStatus(const LineSearchResult&, const LineSearchResult&)`.
///
/// ```
/// use rustycad_tkmath::{math_opt::utils::line_search_fallback_failure_status, math_utils::*};
/// let mut initial = LineSearchResult::default();
/// initial.status = Status::NumericalError;
/// let fallback = LineSearchResult::default();
/// assert_eq!(line_search_fallback_failure_status(&initial, &fallback), Status::NumericalError);
/// ```
pub fn line_search_fallback_failure_status(
    initial: &LineSearchResult,
    fallback: &LineSearchResult,
) -> Status {
    failure_after_fallback(initial.status, fallback.status)
}

/// Status form of [`line_search_fallback_failure_status`] for solvers that keep only the
/// initial search's status.
pub(crate) fn failure_after_fallback(initial: Status, fallback: Status) -> Status {
    if is_evaluation_error(fallback) {
        return fallback;
    }
    if is_evaluation_error(initial) {
        initial
    } else {
        Status::NotConverged
    }
}

/// Armijo backtracking (c1 = 1e-4, rho = 0.5, 50 steps) on a checked callback that also
/// demands a descent direction, a step of at least `step_min` and sufficient decrease;
/// `MathOpt::Utils::Backtrack`.
///
/// ```
/// use rustycad_tkmath::{math::Vector, math_opt::utils::backtrack, math_utils::vector::Sphere};
/// let mut x = Vector::with_size(1);
/// x[0] = 1.0;
/// let mut d = Vector::with_size(1);
/// d[0] = -1.0;
/// let mut g = Vector::with_size(1);
/// g[0] = 2.0;
/// let result = backtrack(&mut Sphere::new(1), &x, &d, &g, 1.0, 1.0, f64::EPSILON);
/// assert!(result.is_valid && result.alpha == 1.0);
/// ```
pub fn backtrack(
    f: &mut impl MultipleVarFunction,
    point: &Vector<'_>,
    direction: &Vector<'_>,
    gradient: &Vector<'_>,
    fx: f64,
    initial_step: f64,
    step_min: f64,
) -> LineSearchResult {
    if !is_finite_vector(point)
        || !is_finite_vector(direction)
        || !is_finite_vector(gradient)
        || !fx.is_finite()
        || !initial_step.is_finite()
        || initial_step <= 0.0
    {
        return LineSearchResult::default();
    }
    let derivative: f64 = directional_derivative(point, direction, gradient);
    if !derivative.is_finite() || derivative >= 0.0 {
        return LineSearchResult::default();
    }
    let mut checked = CheckedFunction::new(f);
    let mut result: LineSearchResult = armijo_backtrack(
        &mut checked,
        point,
        direction,
        gradient,
        fx,
        [initial_step, 1.0e-4, 0.5],
        50,
    );
    accept_backtrack(&checked, &mut result, fx, derivative, step_min);
    result
}

/// `g . d` accumulated over the point's size in index order.
fn directional_derivative(
    point: &Vector<'_>,
    direction: &Vector<'_>,
    gradient: &Vector<'_>,
) -> f64 {
    let mut derivative: f64 = 0.0;
    for index in 0..point.length() as usize {
        derivative += gradient.at(index).unwrap() * direction.at(index).unwrap();
    }
    derivative
}

/// Applies the checked callback's failure and the native acceptance tests to a result.
fn accept_backtrack<F>(
    checked: &CheckedFunction<'_, F>,
    result: &mut LineSearchResult,
    fx: f64,
    derivative: f64,
    step_min: f64,
) {
    if checked.failure_status != Status::Ok {
        result.status = checked.failure_status;
    }
    if checked.had_failure
        || !result.is_valid
        || !result.alpha.is_finite()
        || !result.f_new.is_finite()
        || result.alpha < step_min
        || result.f_new > fx + 1.0e-4 * result.alpha * derivative
    {
        result.is_valid = false;
    }
}
