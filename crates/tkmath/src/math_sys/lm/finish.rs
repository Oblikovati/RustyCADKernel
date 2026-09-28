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

//! `MathSys::Utils::FinishLM` of MathSys_LevenbergMarquardt.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{LMConfig, LMState, gradient, same_bounds};
use crate::{
    math::{FunctionSetWithDerivatives, Matrix, Vector},
    math_lin::utils::{is_finite_matrix, is_finite_vector},
    math_sys::{
        SystemResult,
        linear::{safe_norm, scaled_rank, zero_matrix},
    },
    math_utils::Status,
};

/// Re-evaluates the final point and fills the result diagnostics (`FinishLM`): solution,
/// gradient `J^T F`, residual norm, squared value when finite, Jacobian, scaled rank,
/// `is_root` against `f_tolerance + relative_tolerance * initial_residual_norm` and
/// `is_stationary` from the bound-projected gradient. Bounds apply only when both are given.
/// Returns false with `CallbackError`/`NumericalError` set on failure;
/// `finish_lm(&mut f, &x, 2, &LMConfig::default(), 1.0, None, None, &mut result)`.
#[allow(clippy::too_many_arguments)] // Keeps the upstream eight-argument contract.
pub fn finish_lm(
    f: &mut impl FunctionSetWithDerivatives,
    solution: &Vector<'_>,
    equations: usize,
    config: &LMConfig,
    initial_residual_norm: f64,
    lower: Option<&Vector<'_>>,
    upper: Option<&Vector<'_>>,
    result: &mut SystemResult,
) -> bool {
    result.solution = Some(same_bounds(solution));
    let (residual, jacobian, rank) = match evaluate(f, solution, equations, config) {
        Ok(values) => values,
        Err(status) => {
            result.status = status;
            return false;
        }
    };
    let bounds = lower.zip(upper);
    fill_fields(result, residual, jacobian, rank);
    if !is_finite_vector(result.gradient.as_ref().unwrap()) {
        result.status = Status::NumericalError;
        return false;
    }
    let norm: f64 = projected_gradient_norm(solution, result.gradient.as_ref().unwrap(), bounds);
    result.is_root = result.residual_norm
        <= config.solver.f_tolerance + config.solver.relative_tolerance * initial_residual_norm;
    result.is_stationary = norm <= config.solver.tolerance;
    true
}
type Evaluation = (Vector<'static>, Matrix<'static>, usize);
fn evaluate(
    f: &mut impl FunctionSetWithDerivatives,
    solution: &Vector<'_>,
    equations: usize,
    config: &LMConfig,
) -> Result<Evaluation, Status> {
    let mut residual: Vector<'static> = Vector::with_size(equations);
    let mut jacobian: Matrix<'static> = zero_matrix(equations, solution.length() as usize);
    if !f.value(solution, &mut residual) {
        return Err(Status::CallbackError);
    }
    if !is_finite_vector(&residual) {
        return Err(Status::NumericalError);
    }
    if !f.derivatives(solution, &mut jacobian) {
        return Err(Status::CallbackError);
    }
    if !is_finite_matrix(&jacobian) {
        return Err(Status::NumericalError);
    }
    let rank: usize =
        scaled_rank(&jacobian, config.rank_tolerance).ok_or(Status::NumericalError)?;
    Ok((residual, jacobian, rank))
}
/// Stores gradient, residual norm and value first: upstream keeps them even when the
/// gradient then proves non-finite, while Jacobian and rank are stored only on success.
fn fill_fields(
    result: &mut SystemResult,
    residual: Vector<'static>,
    jacobian: Matrix<'static>,
    rank: usize,
) {
    let gradient: Vector<'static> = gradient(&jacobian, &residual);
    let finite: bool = is_finite_vector(&gradient);
    result.gradient = Some(gradient);
    result.residual_norm = safe_norm(&residual);
    let square: f64 = result.residual_norm * result.residual_norm;
    if square.is_finite() {
        result.value = Some(square);
    }
    if finite {
        result.jacobian = Some(jacobian);
        result.rank = rank;
    }
}
/// Hypot norm of the gradient with components pushing out of an active bound zeroed;
/// `projected_gradient_norm(&x, &g, Some((&lower, &upper)))`.
pub(super) fn projected_gradient_norm(
    solution: &Vector<'_>,
    gradient: &Vector<'_>,
    bounds: Option<(&Vector<'_>, &Vector<'_>)>,
) -> f64 {
    let mut norm: f64 = 0.0;
    for i in 0..solution.length() as usize {
        let x: f64 = *solution.at(i).unwrap();
        let mut component: f64 = *gradient.at(i).unwrap();
        if let Some((lower, upper)) = bounds
            && ((x <= *lower.at(i).unwrap() && component > 0.0)
                || (x >= *upper.at(i).unwrap() && component < 0.0))
        {
            component = 0.0;
        }
        norm = norm.hypot(component);
    }
    norm
}
impl LMState {
    pub(super) fn finish(
        mut self,
        f: &mut impl FunctionSetWithDerivatives,
        exhausted: bool,
    ) -> SystemResult {
        if self.result.status == Status::NotConverged && exhausted {
            self.result.status = Status::MaxIterations;
        }
        if matches!(
            self.result.status,
            Status::CallbackError | Status::NumericalError
        ) {
            self.result.solution = Some(self.solution);
            return self.result;
        }
        let equations: usize = self.residual.length() as usize;
        let finished: bool = finish_lm(
            f,
            &self.solution,
            equations,
            &self.config,
            self.initial_norm,
            self.bounds.as_ref().map(|b| &b.0),
            self.bounds.as_ref().map(|b| &b.1),
            &mut self.result,
        );
        if finished && (self.result.is_root || self.result.is_stationary) {
            self.result.status = Status::Ok;
        } else if self.result.status == Status::Ok {
            self.result.status = Status::NotConverged;
        }
        self.result
    }
}
