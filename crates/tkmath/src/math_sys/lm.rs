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

//! MathSys_LevenbergMarquardt.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{
    SystemResult,
    linear::{safe_norm, zero_matrix},
    newton::{owned, same_bounds},
};
use crate::{
    math::{FunctionSetWithDerivatives, Matrix, Vector},
    math_lin::utils::{is_finite_matrix, is_finite_vector},
    math_poly::extended::LongDouble as E,
    math_utils::Status,
};
mod config;
mod damping;
mod finish;
pub use config::LMConfig;
pub use finish::finish_lm;
/// Damping retries tried in one LM iteration before giving up (`THE_LM_DAMPING_MAX`).
pub const LM_DAMPING_MAX: u32 = 24;
struct LMState {
    solution: Vector<'static>,
    residual: Vector<'static>,
    trial_residual: Vector<'static>,
    jacobian: Matrix<'static>,
    bounds: Option<(Vector<'static>, Vector<'static>)>,
    config: LMConfig,
    initial_norm: f64,
    merit: E,
    lambda: f64,
    result: SystemResult,
}
/// Minimizes squared residuals with native damping retries; `levenberg_marquardt(&mut f,&start,&LMConfig::default())`.
pub fn levenberg_marquardt(
    f: &mut impl FunctionSetWithDerivatives,
    start: &Vector<'_>,
    config: &LMConfig,
) -> SystemResult {
    lm_impl(f, start, config, None, None)
}
/// Projects LM steps onto box constraints; `levenberg_marquardt_bounded(&mut f,&start,&lower,&upper,&LMConfig::default())`.
pub fn levenberg_marquardt_bounded(
    f: &mut impl FunctionSetWithDerivatives,
    start: &Vector<'_>,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    config: &LMConfig,
) -> SystemResult {
    lm_impl(f, start, config, Some(lower), Some(upper))
}
/// Shared LM driver (`LMImpl`): validates the start and configuration, clamps into the box,
/// iterates damped scaled steps with gain-ratio control and finishes with [`finish_lm`].
/// Bounds apply only when both `lower` and `upper` are given, as with the native pointers;
/// `lm_impl(&mut f, &start, &LMConfig::default(), None, None)`.
pub fn lm_impl(
    f: &mut impl FunctionSetWithDerivatives,
    start: &Vector<'_>,
    config: &LMConfig,
    lower: Option<&Vector<'_>>,
    upper: Option<&Vector<'_>>,
) -> SystemResult {
    let bounds: Option<(&Vector<'_>, &Vector<'_>)> = lower.zip(upper);
    if f.variables() <= 0 || f.equations() <= 0 {
        return invalid();
    }
    let n: usize = f.variables() as usize;
    let m: usize = f.equations() as usize;
    if start.length() as usize != n
        || !is_finite_vector(start)
        || !config.valid()
        || !valid_bounds(n, bounds)
    {
        return invalid();
    }
    let mut state = LMState {
        solution: owned(start),
        residual: Vector::with_size(m),
        trial_residual: Vector::with_size(m),
        jacobian: zero_matrix(m, n),
        bounds: bounds.map(|(a, b)| (owned(a), owned(b))),
        config: *config,
        initial_norm: 0.0,
        merit: E::ZERO,
        lambda: config
            .lambda_init
            .clamp(config.lambda_min, config.lambda_max),
        result: SystemResult::default(),
    };
    for i in 0..n {
        *state.solution.change_at(i).unwrap() = state.clamp(i, *state.solution.at(i).unwrap());
    }
    if !f.value(&state.solution, &mut state.residual) {
        state.result.status = Status::CallbackError;
        return state.result;
    }
    if !is_finite_vector(&state.residual) {
        state.result.status = Status::NumericalError;
        return state.result;
    }
    state.initial_norm = safe_norm(&state.residual);
    state.merit = merit(state.initial_norm);
    state.iterate(f)
}
fn invalid() -> SystemResult {
    SystemResult {
        status: Status::InvalidInput,
        ..Default::default()
    }
}
fn valid_bounds(n: usize, bounds: Option<(&Vector<'_>, &Vector<'_>)>) -> bool {
    let Some((lower, upper)) = bounds else {
        return true;
    };
    lower.length() as usize == n
        && upper.length() as usize == n
        && is_finite_vector(lower)
        && is_finite_vector(upper)
        && (0..n).all(|i| lower.at(i).unwrap() <= upper.at(i).unwrap())
}
fn merit(norm: f64) -> E {
    E::new(0.5) * E::new(norm) * E::new(norm)
}
impl LMState {
    fn clamp(&self, i: usize, value: f64) -> f64 {
        if let Some((lower, upper)) = &self.bounds {
            value.clamp(*lower.at(i).unwrap(), *upper.at(i).unwrap())
        } else {
            value
        }
    }
    fn iterate(mut self, f: &mut impl FunctionSetWithDerivatives) -> SystemResult {
        for iteration in 0..self.config.solver.max_iterations {
            self.result.iterations = iteration;
            if self.iteration(f) {
                return self.finish(f, false);
            }
        }
        self.finish(f, true)
    }
    fn iteration(&mut self, f: &mut impl FunctionSetWithDerivatives) -> bool {
        if !f.derivatives(&self.solution, &mut self.jacobian) {
            self.result.status = Status::CallbackError;
            return true;
        }
        if !is_finite_matrix(&self.jacobian) {
            self.result.status = Status::NumericalError;
            return true;
        }
        let gradient: Vector<'static> = gradient(&self.jacobian, &self.residual);
        if !is_finite_vector(&gradient) {
            self.result.status = Status::NumericalError;
            return true;
        }
        let bounds = self.bounds.as_ref().map(|(lower, upper)| (lower, upper));
        if finish::projected_gradient_norm(&self.solution, &gradient, bounds)
            <= self.config.solver.tolerance
        {
            self.result.status = Status::Ok;
            self.result.is_stationary = true;
            return true;
        }
        let accepted: bool = self.damping(f, &gradient);
        self.result.iterations += 1;
        if matches!(
            self.result.status,
            Status::CallbackError | Status::NumericalError
        ) {
            return true;
        }
        if !accepted {
            self.result.status = Status::NotConverged;
            return true;
        }
        self.check_step()
    }
    fn check_step(&mut self) -> bool {
        let scale: f64 = self
            .solution
            .values()
            .iter()
            .fold(1.0_f64, |max, value| max.max(value.abs()));
        let tolerance: f64 =
            self.config.solver.step_min.max(
                self.config.solver.x_tolerance + self.config.solver.relative_tolerance * scale,
            );
        if self.result.step_norm > tolerance {
            return false;
        }
        self.result.status = if safe_norm(&self.residual) <= self.config.solver.f_tolerance {
            Status::Ok
        } else {
            Status::NotConverged
        };
        true
    }
}
fn gradient(jacobian: &Matrix<'_>, residual: &Vector<'_>) -> Vector<'static> {
    let mut gradient: Vector<'static> = Vector::with_size(jacobian.col_number() as usize);
    for row in 0..jacobian.row_number() as usize {
        for col in 0..jacobian.col_number() as usize {
            *gradient.change_at(col).unwrap() +=
                jacobian.at(row, col).unwrap() * residual.at(row).unwrap();
        }
    }
    gradient
}
