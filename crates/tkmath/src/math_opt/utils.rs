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

//! MathOpt_Utils.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
pub(super) fn valid_config(config: &Config) -> bool {
    config.is_valid()
        && config.tolerance > 0.0
        && config.x_tolerance > 0.0
        && config.f_tolerance > 0.0
}
pub(super) fn finite_vector(point: &Vector<'_>) -> bool {
    point.length() > 0 && point.values().iter().all(|v| v.is_finite())
}
pub(super) fn finite_matrix(matrix: &Matrix<'_>) -> bool {
    if matrix.row_number() == 0 || matrix.col_number() == 0 {
        return false;
    }
    for row in 0..matrix.row_number() as usize {
        for column in 0..matrix.col_number() as usize {
            if !matrix.at(row, column).unwrap().is_finite() {
                return false;
            }
        }
    }
    true
}
pub(super) fn value_status(
    f: &mut impl MultipleVarFunction,
    point: &Vector<'_>,
    value: &mut f64,
) -> Status {
    if !finite_vector(point) {
        return Status::NumericalError;
    }
    if !f.value(point, value) {
        return Status::CallbackError;
    }
    if value.is_finite() {
        Status::Ok
    } else {
        Status::NumericalError
    }
}
pub(super) fn gradient_status(
    f: &mut impl MultipleVarFunctionWithGradient,
    point: &Vector<'_>,
    gradient: &mut Vector<'_>,
) -> Status {
    if !finite_vector(point) {
        return Status::NumericalError;
    }
    if !f.gradient(point, gradient) {
        return Status::CallbackError;
    }
    if finite_vector(gradient) {
        Status::Ok
    } else {
        Status::NumericalError
    }
}
pub(super) fn norm(point: &Vector<'_>) -> f64 {
    let mut value: f64 = 0.0;
    for x in point.values() {
        value = value.hypot(*x);
    }
    value
}
pub(super) fn zero_based(point: &Vector<'_>) -> Vector<'static> {
    let mut result: Vector<'static> = Vector::with_size(point.length() as usize);
    result.initialized(point).unwrap();
    result
}
pub(super) fn identity(n: usize) -> Matrix<'static> {
    let mut matrix: Matrix<'static> =
        Matrix::filled(0, n as i32 - 1, 0, n as i32 - 1, 0.0).unwrap();
    for index in 0..n {
        *matrix.change_at(index, index).unwrap() = 1.0;
    }
    matrix
}
pub(super) fn max_difference(a: &Vector<'_>, b: &Vector<'_>) -> f64 {
    let mut maximum: f64 = 0.0;
    for (left, right) in a.values().iter().zip(b.values()) {
        let diff: f64 = (left - right).abs();
        if maximum < diff {
            maximum = diff;
        }
    }
    maximum
}
pub(super) fn evaluation_error(status: Status) -> bool {
    matches!(status, Status::CallbackError | Status::NumericalError)
}
pub(super) fn line_failure(first: Status, last: Status) -> Status {
    if evaluation_error(last) {
        return last;
    }
    if evaluation_error(first) {
        first
    } else {
        Status::NotConverged
    }
}
pub(super) struct Checked<'a, F> {
    pub(super) function: &'a mut F,
    pub(super) failure: Status,
}
impl<F: MultipleVarFunction> MultipleVarFunction for Checked<'_, F> {
    fn variables(&self) -> i32 {
        self.function.variables()
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let status: Status = value_status(self.function, point, value);
        if status != Status::Ok && self.failure == Status::Ok {
            self.failure = status;
        }
        status == Status::Ok
    }
}
pub(super) fn backtrack(
    f: &mut impl MultipleVarFunction,
    point: &Vector<'_>,
    direction: &Vector<'_>,
    gradient: &Vector<'_>,
    fx: f64,
    step_min: f64,
) -> LineSearchResult {
    backtrack_initial(f, point, direction, gradient, fx, 1.0, step_min)
}
pub(super) fn backtrack_initial(
    f: &mut impl MultipleVarFunction,
    point: &Vector<'_>,
    direction: &Vector<'_>,
    gradient: &Vector<'_>,
    fx: f64,
    initial_step: f64,
    step_min: f64,
) -> LineSearchResult {
    if !finite_vector(point)
        || !finite_vector(direction)
        || !finite_vector(gradient)
        || !fx.is_finite()
        || !initial_step.is_finite()
        || initial_step <= 0.0
    {
        return LineSearchResult::default();
    }
    let mut derivative: f64 = 0.0;
    for index in 0..point.length() as usize {
        derivative += gradient.at(index).unwrap() * direction.at(index).unwrap();
    }
    if !derivative.is_finite() || derivative >= 0.0 {
        return LineSearchResult::default();
    }
    let mut checked: Checked<'_, _> = Checked {
        function: f,
        failure: Status::Ok,
    };
    let mut result: LineSearchResult = armijo_backtrack(
        &mut checked,
        point,
        direction,
        gradient,
        fx,
        [initial_step, 1e-4, 0.5],
        50,
    );
    if checked.failure != Status::Ok {
        result.status = checked.failure;
    }
    if checked.failure != Status::Ok
        || !result.is_valid
        || !result.alpha.is_finite()
        || !result.f_new.is_finite()
        || result.alpha < step_min
        || result.f_new > fx + 1e-4 * result.alpha * derivative
    {
        result.is_valid = false;
    }
    result
}
