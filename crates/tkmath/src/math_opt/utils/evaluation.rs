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

//! Callback evaluation with native status classification (`MathOpt::Utils::*Status`).
use super::{is_finite_matrix, is_finite_vector};
use crate::math::{Function, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
use crate::math_opt::HessianFunction;
use crate::math_utils::Status;

/// Maps a finite result to `Ok` and anything else to `NumericalError`.
fn finite_status(finite: bool) -> Status {
    if finite {
        Status::Ok
    } else {
        Status::NumericalError
    }
}

/// Evaluates a scalar callback: a non-finite argument or result is `NumericalError`, a
/// callback refusal is `CallbackError`; `MathOpt::Utils::ValueStatus(Function&,double,double&)`.
///
/// ```
/// use rustycad_tkmath::{math_opt::utils::scalar_value_status, math_utils::{scalar::make_scalar, Status}};
/// let mut f = make_scalar(|x: f64, y: &mut f64| { *y = x * x; true });
/// let mut value = 0.0;
/// assert_eq!(scalar_value_status(&mut f, 3.0, &mut value), Status::Ok);
/// assert_eq!(scalar_value_status(&mut f, f64::NAN, &mut value), Status::NumericalError);
/// ```
pub fn scalar_value_status(f: &mut impl Function, x: f64, value: &mut f64) -> Status {
    if !x.is_finite() {
        return Status::NumericalError;
    }
    if !f.value(x, value) {
        return Status::CallbackError;
    }
    finite_status(value.is_finite())
}

/// Evaluates a multivariate callback with the native status classification;
/// `MathOpt::Utils::ValueStatus(Function&,const math_Vector&,double&)`.
///
/// ```
/// use rustycad_tkmath::{math::Vector, math_opt::utils::value_status, math_utils::{vector::Sphere, Status}};
/// let mut value = 0.0;
/// let status = value_status(&mut Sphere::new(2), &Vector::with_size(2), &mut value);
/// assert_eq!(status, Status::Ok);
/// ```
pub fn value_status(
    f: &mut impl MultipleVarFunction,
    point: &Vector<'_>,
    value: &mut f64,
) -> Status {
    if !is_finite_vector(point) {
        return Status::NumericalError;
    }
    if !f.value(point, value) {
        return Status::CallbackError;
    }
    finite_status(value.is_finite())
}

/// True when [`value_status`] is `Ok`; `MathOpt::Utils::Value`.
///
/// ```
/// use rustycad_tkmath::{math::Vector, math_opt::utils::checked_value, math_utils::vector::Sphere};
/// let mut value = 0.0;
/// assert!(checked_value(&mut Sphere::new(2), &Vector::with_size(2), &mut value));
/// ```
pub fn checked_value(
    f: &mut impl MultipleVarFunction,
    point: &Vector<'_>,
    value: &mut f64,
) -> bool {
    value_status(f, point, value) == Status::Ok
}

/// Evaluates a gradient callback, requiring a finite point and a finite, non-empty gradient;
/// `MathOpt::Utils::GradientStatus`.
///
/// ```
/// use rustycad_tkmath::{math::Vector, math_opt::utils::gradient_status, math_utils::{vector::Sphere, Status}};
/// let mut gradient = Vector::with_size(2);
/// let status = gradient_status(&mut Sphere::new(2), &Vector::with_size(2), &mut gradient);
/// assert_eq!(status, Status::Ok);
/// ```
pub fn gradient_status(
    f: &mut impl MultipleVarFunctionWithGradient,
    point: &Vector<'_>,
    gradient: &mut Vector<'_>,
) -> Status {
    if !is_finite_vector(point) {
        return Status::NumericalError;
    }
    if !f.gradient(point, gradient) {
        return Status::CallbackError;
    }
    finite_status(is_finite_vector(gradient))
}

/// True when [`gradient_status`] is `Ok`; `MathOpt::Utils::Gradient`.
///
/// ```
/// use rustycad_tkmath::{math::Vector, math_opt::utils::checked_gradient, math_utils::vector::Sphere};
/// let mut gradient = Vector::with_size(2);
/// assert!(checked_gradient(&mut Sphere::new(2), &Vector::with_size(2), &mut gradient));
/// ```
pub fn checked_gradient(
    f: &mut impl MultipleVarFunctionWithGradient,
    point: &Vector<'_>,
    gradient: &mut Vector<'_>,
) -> bool {
    gradient_status(f, point, gradient) == Status::Ok
}

/// Evaluates a Hessian callback, requiring a finite point and a finite, non-empty matrix;
/// `MathOpt::Utils::HessianStatus`.
///
/// ```
/// use rustycad_tkmath::{math::{Matrix, Vector}, math_opt::utils::hessian_status};
/// # use rustycad_tkmath::math_utils::Status;
/// # fn check(f: &mut impl rustycad_tkmath::math_opt::HessianFunction) -> Status {
/// let mut h = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
/// hessian_status(f, &Vector::with_size(2), &mut h)
/// # }
/// ```
pub fn hessian_status(
    f: &mut impl HessianFunction,
    point: &Vector<'_>,
    hessian: &mut Matrix<'_>,
) -> Status {
    if !is_finite_vector(point) {
        return Status::NumericalError;
    }
    if !f.hessian(point, hessian) {
        return Status::CallbackError;
    }
    finite_status(is_finite_matrix(hessian))
}

/// True when [`hessian_status`] is `Ok`; `MathOpt::Utils::Hessian`.
///
/// ```
/// use rustycad_tkmath::{math::{Matrix, Vector}, math_opt::utils::checked_hessian};
/// # fn check(f: &mut impl rustycad_tkmath::math_opt::HessianFunction) -> bool {
/// let mut h = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
/// checked_hessian(f, &Vector::with_size(2), &mut h)
/// # }
/// ```
pub fn checked_hessian(
    f: &mut impl HessianFunction,
    point: &Vector<'_>,
    hessian: &mut Matrix<'_>,
) -> bool {
    hessian_status(f, point, hessian) == Status::Ok
}

/// Objective adapter that records the first failed evaluation and whether any evaluation
/// succeeded; `MathOpt::Utils::CheckedFunction<Function>`.
///
/// ```
/// use rustycad_tkmath::{math::{MultipleVarFunction, Vector}, math_opt::utils::CheckedFunction};
/// use rustycad_tkmath::math_utils::{vector::Sphere, Status};
/// let mut sphere = Sphere::new(1);
/// let mut checked = CheckedFunction::new(&mut sphere);
/// let mut value = 0.0;
/// assert!(checked.value(&Vector::with_size(1), &mut value));
/// assert!(checked.has_valid_evaluation && !checked.had_failure);
/// assert_eq!(checked.failure_status, Status::Ok);
/// ```
pub struct CheckedFunction<'a, F> {
    function: &'a mut F,
    /// Whether any evaluation failed; `HadFailure`.
    pub had_failure: bool,
    /// Whether any evaluation succeeded; `HasValidEvaluation`.
    pub has_valid_evaluation: bool,
    /// Status of the first failed evaluation, `Ok` until one fails; `FailureStatus`.
    pub failure_status: Status,
}

impl<'a, F: MultipleVarFunction> CheckedFunction<'a, F> {
    /// Wraps a callback with a clean failure record; `CheckedFunction(Function&)`.
    pub fn new(function: &'a mut F) -> Self {
        Self {
            function,
            had_failure: false,
            has_valid_evaluation: false,
            failure_status: Status::Ok,
        }
    }

    /// Returns the wrapped callback; `CheckedFunction::Original()`.
    pub fn original(&mut self) -> &mut F {
        self.function
    }
}

impl<F: MultipleVarFunction> MultipleVarFunction for CheckedFunction<'_, F> {
    fn variables(&self) -> i32 {
        self.function.variables()
    }

    /// Evaluates through [`value_status`] and updates the failure record;
    /// `CheckedFunction::Value`.
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let status: Status = value_status(self.function, point, value);
        let is_valid: bool = status == Status::Ok;
        self.had_failure = self.had_failure || !is_valid;
        self.has_valid_evaluation = self.has_valid_evaluation || is_valid;
        if !is_valid && self.failure_status == Status::Ok {
            self.failure_status = status;
        }
        is_valid
    }
}
