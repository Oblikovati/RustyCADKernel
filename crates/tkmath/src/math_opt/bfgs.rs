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

//! Basic BFGS and numerical-gradient adapter from MathOpt_BFGS.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::*;
use super::*;
struct BfgsState {
    point: Vector<'static>,
    gradient: Vector<'static>,
    next: Vector<'static>,
    next_gradient: Vector<'static>,
    step: Vector<'static>,
    difference: Vector<'static>,
    direction: Vector<'static>,
    inverse: Matrix<'static>,
    fx: f64,
    initial_norm: f64,
    iterations: u32,
}
impl BfgsState {
    fn initialize(
        f: &mut impl MultipleVarFunctionWithGradient,
        start: &Vector<'_>,
    ) -> Result<Self, Status> {
        let point: Vector<'static> = zero_based(start);
        let n: usize = point.length() as usize;
        let mut fx: f64 = 0.0;
        let status: Status = value_status(f, &point, &mut fx);
        if status != Status::Ok {
            return Err(status);
        }
        let mut gradient: Vector<'static> = Vector::with_size(n);
        let status: Status = gradient_status(f, &point, &mut gradient);
        if status != Status::Ok {
            return Err(status);
        }
        let initial_norm: f64 = norm(&gradient);
        Ok(Self {
            point,
            gradient,
            fx,
            initial_norm,
            iterations: 0,
            inverse: identity(n),
            next: Vector::with_size(n),
            next_gradient: Vector::with_size(n),
            step: Vector::with_size(n),
            difference: Vector::with_size(n),
            direction: Vector::with_size(n),
        })
    }
    fn finish(self, status: Status) -> VectorResult {
        VectorResult {
            status,
            iterations: self.iterations,
            solution: Some(self.point),
            value: Some(self.fx),
            gradient: Some(self.gradient),
            ..Default::default()
        }
    }
    fn search_direction(&mut self) {
        for row in 0..self.point.length() {
            let mut sum: f64 = 0.0;
            for column in 0..self.point.length() {
                sum += self.inverse[(row, column)] * self.gradient[column];
            }
            self.direction[row] = -sum;
        }
    }
    fn line(
        &mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        config: &Config,
    ) -> Result<LineSearchResult, Status> {
        self.search_direction();
        let first: LineSearchResult = backtrack(
            f,
            &self.point,
            &self.direction,
            &self.gradient,
            self.fx,
            config.step_min,
        );
        if first.is_valid {
            return Ok(first);
        }
        for index in 0..self.point.length() {
            self.direction[index] = -self.gradient[index];
        }
        let fallback: LineSearchResult = backtrack(
            f,
            &self.point,
            &self.direction,
            &self.gradient,
            self.fx,
            config.step_min,
        );
        if !fallback.is_valid {
            return Err(line_failure(first.status, fallback.status));
        }
        self.inverse = identity(self.point.length() as usize);
        Ok(fallback)
    }
    fn prepare_next(&mut self, alpha: f64) {
        for index in 0..self.point.length() {
            self.next[index] = self.point[index] + alpha * self.direction[index];
        }
        for index in 0..self.point.length() {
            self.step[index] = self.next[index] - self.point[index];
        }
    }
    fn accept(&mut self, value: f64) {
        self.point.initialized(&self.next).unwrap();
        self.gradient.initialized(&self.next_gradient).unwrap();
        self.fx = value;
    }
    fn update_inverse(&mut self) {
        let mut curvature: f64 = 0.0;
        for index in 0..self.point.length() {
            self.difference[index] = self.next_gradient[index] - self.gradient[index];
        }
        for index in 0..self.point.length() {
            curvature += self.step[index] * self.difference[index];
        }
        if curvature <= ZERO_TOL || curvature.is_nan() {
            return;
        }
        let rho: f64 = 1.0 / curvature;
        let mut hy: Vector<'_> = Vector::with_size(self.point.length() as usize);
        for row in 0..self.point.length() {
            for column in 0..self.point.length() {
                hy[row] += self.inverse[(row, column)] * self.difference[column];
            }
        }
        let mut yhy: f64 = 0.0;
        for index in 0..self.point.length() {
            yhy += self.difference[index] * hy[index];
        }
        self.update_entries(&hy, rho, 1.0 + rho * yhy);
    }
    fn update_entries(&mut self, hy: &Vector<'_>, rho: f64, factor: f64) {
        for row in 0..self.point.length() {
            for column in 0..self.point.length() {
                self.inverse[(row, column)] = self.inverse[(row, column)]
                    - rho * (hy[row] * self.step[column] + self.step[row] * hy[column])
                    + factor * rho * self.step[row] * self.step[column];
            }
        }
    }
    fn iteration(
        &mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        config: &Config,
    ) -> Result<Option<Status>, Status> {
        let line: LineSearchResult = self.line(f, config)?;
        self.prepare_next(line.alpha);
        let status: Status = gradient_status(f, &self.next, &mut self.next_gradient);
        if status != Status::Ok {
            return Err(status);
        }
        let gradient_norm: f64 = norm(&self.next_gradient);
        if gradient_norm <= config.f_tolerance {
            self.accept(line.f_new);
            return Ok(Some(Status::Ok));
        }
        if max_difference(&self.next, &self.point) <= config.x_tolerance {
            let tolerance: f64 = config.f_tolerance + config.relative_tolerance * self.initial_norm;
            let status: Status = if gradient_norm <= tolerance {
                Status::Ok
            } else {
                Status::NotConverged
            };
            self.accept(line.f_new);
            return Ok(Some(status));
        }
        self.update_inverse();
        self.accept(line.f_new);
        Ok(None)
    }
}
/// Minimizes using native BFGS updates and fallback line searches; `bfgs(&mut f,&start,&Config::default())`.
pub fn bfgs(
    f: &mut impl MultipleVarFunctionWithGradient,
    start: &Vector<'_>,
    config: &Config,
) -> VectorResult {
    if !valid_config(config) || !finite_vector(start) {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut state: BfgsState = match BfgsState::initialize(f, start) {
        Ok(state) => state,
        Err(status) => {
            return VectorResult {
                status,
                ..Default::default()
            };
        }
    };
    if state.initial_norm <= config.f_tolerance {
        return state.finish(Status::Ok);
    }
    for iteration in 0..config.max_iterations {
        state.iterations = iteration + 1;
        match state.iteration(f, config) {
            Ok(Some(status)) | Err(status) => return state.finish(status),
            Ok(None) => {}
        }
    }
    state.finish(Status::MaxIterations)
}
pub(super) struct Numerical<'a, F> {
    pub(super) function: &'a mut F,
    pub(super) step: f64,
}
impl<F: MultipleVarFunction> MultipleVarFunction for Numerical<'_, F> {
    fn variables(&self) -> i32 {
        self.function.variables()
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        self.function.value(point, value)
    }
}
impl<F: MultipleVarFunction> MultipleVarFunctionWithGradient for Numerical<'_, F> {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let mut copy: Vector<'static> = zero_based(point);
        crate::math_utils::numerical_gradient_adaptive(
            self.function,
            &mut copy,
            gradient,
            self.step,
        )
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
/// Uses native adaptive central differences when derivatives are absent; `bfgs_numerical(&mut f,&start,1e-8,&config)`.
pub fn bfgs_numerical(
    f: &mut impl MultipleVarFunction,
    start: &Vector<'_>,
    step: f64,
    config: &Config,
) -> VectorResult {
    if !step.is_finite() || step <= 0.0 {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    bfgs(&mut Numerical { function: f, step }, start, config)
}

mod bounded;
pub use bounded::bfgs_bounded;
