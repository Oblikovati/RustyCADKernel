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

//! MathSys native behavior at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{
    SystemResult,
    linear::{RANK_TOLERANCE, safe_norm, scaled_rank, zero_matrix},
};
use crate::{
    math::{FunctionSetWithDerivatives, Matrix, Vector},
    math_lin::utils::{is_finite_matrix, is_finite_vector},
    math_utils::Status,
};
mod direction;
mod search;
struct NewtonState {
    solution: Vector<'static>,
    residual: Vector<'static>,
    trial_residual: Vector<'static>,
    jacobian: Matrix<'static>,
    tolerance: Vector<'static>,
    f_tolerance: f64,
    limit: u32,
    bounds: Option<(Vector<'static>, Vector<'static>)>,
    taken: bool,
    within_x: bool,
    result: SystemResult,
}
/// Solves square or rectangular systems with component tolerances; `newton(&mut f,&start,&tol_x,1e-10,100)`.
pub fn newton(
    f: &mut impl FunctionSetWithDerivatives,
    start: &Vector<'_>,
    tolerance: &Vector<'_>,
    f_tolerance: f64,
    limit: u32,
) -> SystemResult {
    newton_impl(f, start, tolerance, f_tolerance, limit, None, None)
}
/// Projects trial steps onto box bounds; `newton_bounded(&mut f,&start,&lower,&upper,&tol_x,1e-10,100)`.
pub fn newton_bounded(
    f: &mut impl FunctionSetWithDerivatives,
    start: &Vector<'_>,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    tolerance: &Vector<'_>,
    f_tolerance: f64,
    limit: u32,
) -> SystemResult {
    newton_impl(
        f,
        start,
        tolerance,
        f_tolerance,
        limit,
        Some(lower),
        Some(upper),
    )
}
/// Uniform variable-tolerance overload; `newton_uniform(&mut f,&start,1e-10,1e-10,100)`.
pub fn newton_uniform(
    f: &mut impl FunctionSetWithDerivatives,
    start: &Vector<'_>,
    x_tolerance: f64,
    f_tolerance: f64,
    limit: u32,
) -> SystemResult {
    let mut tolerance: Vector<'static> = Vector::with_size(start.length() as usize);
    tolerance.init(x_tolerance);
    newton(f, start, &tolerance, f_tolerance, limit)
}
/// Copy that keeps the source's lower bound, as `std::optional<math_Vector> = theSolution` does.
pub(super) fn same_bounds(vector: &Vector<'_>) -> Vector<'static> {
    let upper: i32 = vector.lower() + vector.length() - 1;
    let mut result: Vector<'static> = Vector::with_bounds(vector.lower(), upper).unwrap();
    result.initialized(vector).unwrap();
    result
}
pub(super) fn owned(vector: &Vector<'_>) -> Vector<'static> {
    let mut result: Vector<'static> = Vector::with_size(vector.length() as usize);
    result.initialized(vector).unwrap();
    result
}
/// Shared Newton driver (`NewtonImpl`): validates inputs, clamps the start into the box,
/// takes rank-aware linearized steps with a projected Armijo backtracking of at most 24
/// halvings and finishes with [`finalize_system_result`]. Bounds apply only when both
/// `lower` and `upper` are given, as with the native pointers;
/// `newton_impl(&mut f, &start, &tol_x, 1e-10, 100, None, None)`.
pub fn newton_impl(
    f: &mut impl FunctionSetWithDerivatives,
    start: &Vector<'_>,
    tolerance: &Vector<'_>,
    f_tolerance: f64,
    limit: u32,
    lower: Option<&Vector<'_>>,
    upper: Option<&Vector<'_>>,
) -> SystemResult {
    let bounds: Option<(&Vector<'_>, &Vector<'_>)> = lower.zip(upper);
    if f.variables() <= 0 || f.equations() <= 0 {
        return invalid();
    }
    let n: usize = f.variables() as usize;
    let m: usize = f.equations() as usize;
    if !validate(n, start, tolerance, f_tolerance, limit, bounds) {
        return invalid();
    }
    let mut state = NewtonState {
        solution: owned(start),
        residual: Vector::with_size(m),
        trial_residual: Vector::with_size(m),
        jacobian: zero_matrix(m, n),
        tolerance: owned(tolerance),
        f_tolerance,
        limit,
        bounds: bounds.map(|(a, b)| (owned(a), owned(b))),
        taken: false,
        within_x: false,
        result: SystemResult::default(),
    };
    state.clamp_start();
    state.iterate(f)
}
fn invalid() -> SystemResult {
    SystemResult {
        status: Status::InvalidInput,
        ..Default::default()
    }
}
fn validate(
    n: usize,
    start: &Vector<'_>,
    tolerance: &Vector<'_>,
    f_tolerance: f64,
    limit: u32,
    bounds: Option<(&Vector<'_>, &Vector<'_>)>,
) -> bool {
    if start.length() as usize != n
        || tolerance.length() as usize != n
        || f_tolerance <= 0.0
        || !f_tolerance.is_finite()
        || limit == 0
        || !is_finite_vector(start)
        || !is_finite_vector(tolerance)
    {
        return false;
    }
    if tolerance.values().iter().any(|value| *value <= 0.0) {
        return false;
    }
    let Some((lower, upper)) = bounds else {
        return true;
    };
    lower.length() as usize == n
        && upper.length() as usize == n
        && is_finite_vector(lower)
        && is_finite_vector(upper)
        && (0..n).all(|i| lower.at(i).unwrap() <= upper.at(i).unwrap())
}
impl NewtonState {
    fn clamp_start(&mut self) {
        if let Some((lower, upper)) = &self.bounds {
            for i in 0..self.solution.length() as usize {
                *self.solution.change_at(i).unwrap() = self
                    .solution
                    .at(i)
                    .unwrap()
                    .clamp(*lower.at(i).unwrap(), *upper.at(i).unwrap());
            }
        }
    }
    fn iterate(mut self, f: &mut impl FunctionSetWithDerivatives) -> SystemResult {
        for iteration in 0..self.limit {
            self.result.iterations = iteration;
            match self.iteration(f) {
                Err(status) => {
                    self.result.status = status;
                    self.result.solution = Some(self.solution);
                    return self.result;
                }
                Ok(true) => break,
                Ok(false) => {}
            }
        }
        self.finish(f)
    }
    fn evaluate(&mut self, f: &mut impl FunctionSetWithDerivatives) -> Result<(), Status> {
        if !f.values(&self.solution, &mut self.residual, &mut self.jacobian) {
            return Err(Status::CallbackError);
        }
        if !is_finite_vector(&self.residual) || !is_finite_matrix(&self.jacobian) {
            return Err(Status::NumericalError);
        }
        Ok(())
    }
    fn iteration(&mut self, f: &mut impl FunctionSetWithDerivatives) -> Result<bool, Status> {
        self.evaluate(f)?;
        let norm: f64 = safe_norm(&self.residual);
        if norm <= self.f_tolerance && (!self.taken || self.within_x) {
            self.result.status = Status::Ok;
            return Ok(true);
        }
        let Some((mut direction, gradient, search_gradient)) = self.direction() else {
            return Ok(true);
        };
        if self.direction_converged(&direction) {
            self.within_x = true;
            self.result.status = Status::Ok;
            return Ok(true);
        }
        if !self.ensure_descent(&mut direction, &search_gradient, norm) {
            return Ok(true);
        }
        let accepted: bool = self.line_search(f, &direction, &gradient, norm)?;
        self.result.iterations += 1;
        if !accepted {
            self.result.status = Status::NotConverged;
            return Ok(true);
        }
        if self.within_x {
            self.result.status = if safe_norm(&self.trial_residual) <= self.f_tolerance {
                Status::Ok
            } else {
                Status::NotConverged
            };
            return Ok(true);
        }
        Ok(false)
    }
    fn direction_converged(&self, direction: &Vector<'_>) -> bool {
        direction
            .values()
            .iter()
            .zip(self.tolerance.values())
            .all(|(step, tolerance)| step.abs() <= *tolerance)
            && self
                .residual
                .values()
                .iter()
                .all(|value| value.abs() <= self.f_tolerance)
    }
    fn finish(mut self, f: &mut impl FunctionSetWithDerivatives) -> SystemResult {
        if self.result.status == Status::NotConverged && self.result.iterations == self.limit {
            self.result.status = Status::MaxIterations;
        }
        if !finalize_system_result(
            f,
            &self.solution,
            self.residual.length() as usize,
            self.f_tolerance,
            &mut self.result,
        ) {
            return self.result;
        }
        if self.result.is_root && (!self.taken || self.within_x) {
            self.result.status = Status::Ok;
        } else if self.result.status == Status::Ok {
            self.result.status = Status::NotConverged;
        }
        self.result
    }
}
/// Re-evaluates the final point and fills the result (`FinalizeSystemResult`): solution,
/// residual norm, squared value when finite, Jacobian, scaled rank and `is_root` against
/// `tolerance`. Returns false with `CallbackError`/`NumericalError` set on failure;
/// `finalize_system_result(&mut f, &x, 2, 1e-10, &mut result)`.
pub fn finalize_system_result(
    f: &mut impl FunctionSetWithDerivatives,
    solution: &Vector<'_>,
    equations: usize,
    tolerance: f64,
    result: &mut SystemResult,
) -> bool {
    let mut residual: Vector<'static> = Vector::with_size(equations);
    let mut jacobian: Matrix<'static> = zero_matrix(equations, solution.length() as usize);
    result.solution = Some(same_bounds(solution));
    if !f.values(solution, &mut residual, &mut jacobian) {
        result.status = Status::CallbackError;
        return false;
    }
    if !is_finite_vector(&residual) || !is_finite_matrix(&jacobian) {
        result.status = Status::NumericalError;
        return false;
    }
    let Some(rank) = scaled_rank(&jacobian, RANK_TOLERANCE) else {
        result.status = Status::NumericalError;
        return false;
    };
    result.residual_norm = safe_norm(&residual);
    let square: f64 = result.residual_norm * result.residual_norm;
    result.value = square.is_finite().then_some(square);
    result.jacobian = Some(jacobian);
    result.rank = rank;
    result.is_root = result.residual_norm <= tolerance;
    true
}
