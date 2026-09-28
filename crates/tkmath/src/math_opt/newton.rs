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

//! MathOpt_Newton.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::*;
use super::*;
use crate::math_utils::LinearResult;
/// Separate Hessian callback used by the modern API; `f.hessian(&x,&mut h)`.
pub trait HessianFunction: MultipleVarFunctionWithGradient {
    /// Computes only the Hessian, preserving callback ordering; `f.hessian(&x,&mut h)`.
    fn hessian(&mut self, point: &Vector<'_>, hessian: &mut Matrix<'_>) -> bool;
}
/// Native Newton options; `NewtonConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct NewtonConfig {
    /// Shared tolerances; `config.base.f_tolerance`.
    pub base: Config,
    /// Minimum eigenvalue used to shift the Hessian; `config.regularization`.
    pub regularization: f64,
    /// Enables native backtracking; `config.use_line_search`.
    pub use_line_search: bool,
}
impl Default for NewtonConfig {
    fn default() -> Self {
        Self {
            base: Config::default(),
            regularization: 1e-8,
            use_line_search: true,
        }
    }
}
impl NewtonConfig {
    /// Initializes the native shared tolerances; `NewtonConfig::new(1e-8,100)`.
    pub fn new(tolerance: f64, limit: u32) -> Self {
        Self {
            base: Config::new(tolerance, limit),
            ..Default::default()
        }
    }
    fn valid(&self, start: &Vector<'_>) -> bool {
        is_valid_config(&self.base)
            && is_finite_vector(start)
            && self.regularization.is_finite()
            && self.regularization > 0.0
    }
}
struct NewtonState {
    x: Vector<'static>,
    g: Vector<'static>,
    dir: Vector<'static>,
    next: Vector<'static>,
    next_g: Vector<'static>,
    h: Matrix<'static>,
    fx: f64,
    iterations: u32,
}
impl NewtonState {
    fn initialize(
        f: &mut impl MultipleVarFunctionWithGradient,
        start: &Vector<'_>,
    ) -> Result<Self, Status> {
        let x: Vector<'static> = zero_based(start);
        let n: usize = x.length() as usize;
        let mut fx: f64 = 0.0;
        let status: Status = value_status(f, &x, &mut fx);
        if status != Status::Ok {
            return Err(status);
        }
        let mut g: Vector<'static> = Vector::with_size(n);
        let status: Status = gradient_status(f, &x, &mut g);
        if status != Status::Ok {
            return Err(status);
        }
        Ok(Self {
            x,
            g,
            dir: Vector::with_size(n),
            next: Vector::with_size(n),
            next_g: Vector::with_size(n),
            h: Matrix::with_bounds(0, n as i32 - 1, 0, n as i32 - 1).unwrap(),
            fx,
            iterations: 0,
        })
    }
    fn finish(self, status: Status) -> VectorResult {
        VectorResult {
            status,
            solution: Some(self.x),
            value: Some(self.fx),
            gradient: Some(self.g),
            iterations: self.iterations,
            ..Default::default()
        }
    }
    fn steepest(&mut self) {
        for i in 0..self.g.length() {
            self.dir[i] = -self.g[i];
        }
    }
    fn direction(&mut self, minimum: f64) -> bool {
        self.steepest();
        if !regularize_hessian(&mut self.h, minimum) {
            return false;
        }
        let linear: LinearResult = crate::math_lin::solve(&self.h, &self.dir, 1e-15);
        if !linear.is_done() {
            return true;
        }
        self.dir = linear.solution.unwrap();
        let mut derivative: f64 = 0.0;
        for i in 0..self.g.length() {
            derivative += self.g[i] * self.dir[i];
        }
        if derivative >= 0.0 {
            self.steepest();
        }
        true
    }
    fn advance(&mut self, alpha: f64) {
        for i in 0..self.x.length() {
            self.next[i] = self.x[i] + alpha * self.dir[i];
        }
    }
    fn line(&mut self, f: &mut impl MultipleVarFunction, cfg: &Config) -> Result<f64, Status> {
        let first: LineSearchResult =
            backtrack(f, &self.x, &self.dir, &self.g, self.fx, 1.0, cfg.step_min);
        if first.is_valid {
            self.advance(first.alpha);
            return Ok(first.f_new);
        }
        self.steepest();
        let last: LineSearchResult =
            backtrack(f, &self.x, &self.dir, &self.g, self.fx, 1.0, cfg.step_min);
        if !last.is_valid {
            return Err(failure_after_fallback(first.status, last.status));
        }
        self.advance(last.alpha);
        Ok(last.f_new)
    }
    fn next_value(
        &mut self,
        f: &mut impl MultipleVarFunction,
        cfg: &NewtonConfig,
    ) -> Result<f64, Status> {
        if cfg.use_line_search {
            return self.line(f, &cfg.base);
        }
        self.advance(1.0);
        let mut value: f64 = 0.0;
        let status: Status = value_status(f, &self.next, &mut value);
        if status != Status::Ok {
            return Err(status);
        }
        Ok(value)
    }
    fn accept(&mut self, value: f64) {
        self.x.initialized(&self.next).unwrap();
        self.g.initialized(&self.next_g).unwrap();
        self.fx = value;
    }
    fn run(mut self, f: &mut impl HessianFunction, cfg: &NewtonConfig) -> VectorResult {
        if norm(&self.g) < cfg.base.f_tolerance {
            return self.finish(Status::Ok);
        }
        for iteration in 1..=cfg.base.max_iterations {
            self.iterations = iteration;
            let status: Status = hessian_status(f, &self.x, &mut self.h);
            if status != Status::Ok {
                return self.finish(status);
            }
            if !self.direction(cfg.regularization) {
                return VectorResult {
                    status: Status::NumericalError,
                    iterations: iteration,
                    ..Default::default()
                };
            }
            let value: f64 = match self.next_value(f, cfg) {
                Ok(v) => v,
                Err(s) => return self.finish(s),
            };
            let diff: f64 = max_difference(&self.next, &self.x);
            let status: Status = gradient_status(f, &self.next, &mut self.next_g);
            if status != Status::Ok {
                return self.finish(status);
            }
            self.accept(value);
            if norm(&self.g) < cfg.base.f_tolerance {
                return self.finish(Status::Ok);
            }
            if diff < cfg.base.x_tolerance {
                return self.finish(Status::NotConverged);
            }
        }
        self.finish(Status::MaxIterations)
    }
}
/// Minimizes with native Hessian regularization; `newton(&mut f,&start,&NewtonConfig::default())`.
pub fn newton(
    f: &mut impl HessianFunction,
    start: &Vector<'_>,
    cfg: &NewtonConfig,
) -> VectorResult {
    if !cfg.valid(start) {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    match NewtonState::initialize(f, start) {
        Ok(state) => state.run(f, cfg),
        Err(status) => VectorResult {
            status,
            ..Default::default()
        },
    }
}
/// Preserves the native alias to Newton; `newton_modified(&mut f,&start,&NewtonConfig::default())`.
pub fn newton_modified(
    f: &mut impl HessianFunction,
    start: &Vector<'_>,
    cfg: &NewtonConfig,
) -> VectorResult {
    newton(f, start, cfg)
}

mod bounded;
mod numerical;
pub use bounded::newton_bounded;
pub use numerical::{newton_numerical, newton_numerical_hessian};
