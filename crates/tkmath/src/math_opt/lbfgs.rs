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

//! MathOpt_BFGS.hxx limited-memory algorithm at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::*;
use super::*;
use crate::math_utils::dot_product;
struct History {
    steps: Vec<Vector<'static>>,
    differences: Vec<Vector<'static>>,
    rho: Vec<f64>,
    alpha: Vec<f64>,
    head: usize,
    count: usize,
}
impl History {
    fn new(n: usize, memory: usize) -> Self {
        Self {
            steps: (0..memory).map(|_| Vector::with_size(n)).collect(),
            differences: (0..memory).map(|_| Vector::with_size(n)).collect(),
            rho: vec![0.0; memory],
            alpha: vec![0.0; memory],
            head: 0,
            count: 0,
        }
    }
    fn index(&self, k: usize) -> usize {
        (self.head + k) % self.rho.len()
    }
    fn backward(&mut self, q: &mut Vector<'_>) {
        for k in (0..self.count).rev() {
            let idx: usize = self.index(k);
            self.alpha[idx] = self.rho[idx] * dot_product(&self.steps[idx], q);
            for i in 0..q.length() {
                q[i] -= self.alpha[idx] * self.differences[idx][i];
            }
        }
    }
    fn gamma(&self) -> f64 {
        if self.count == 0 {
            return 1.0;
        }
        let last: usize = self.index(self.count - 1);
        let yy: f64 = dot_product(&self.differences[last], &self.differences[last]);
        if yy > ZERO_TOL {
            1.0 / (self.rho[last] * yy)
        } else {
            1.0
        }
    }
    fn direction(&mut self, gradient: &Vector<'_>, dir: &mut Vector<'_>) {
        let mut q: Vector<'static> = zero_based(gradient);
        self.backward(&mut q);
        let gamma: f64 = self.gamma();
        for i in 0..dir.length() {
            dir[i] = gamma * q[i];
        }
        for k in 0..self.count {
            let idx: usize = self.index(k);
            let beta: f64 = self.rho[idx] * dot_product(&self.differences[idx], dir);
            for i in 0..dir.length() {
                dir[i] += (self.alpha[idx] - beta) * self.steps[idx][i];
            }
        }
        for i in 0..dir.length() {
            dir[i] = -dir[i];
        }
    }
    fn insert(&mut self, x: &Vector<'_>, next: &Vector<'_>, g: &Vector<'_>, next_g: &Vector<'_>) {
        let idx: usize = self.index(self.count);
        // Native storage overwrites the selected slot even when curvature is rejected.
        for i in 0..x.length() {
            self.steps[idx][i] = next[i] - x[i];
            self.differences[idx][i] = next_g[i] - g[i];
        }
        let sy: f64 = dot_product(&self.steps[idx], &self.differences[idx]);
        if sy > ZERO_TOL {
            self.rho[idx] = 1.0 / sy;
            if self.count < self.rho.len() {
                self.count += 1;
            } else {
                self.head = (self.head + 1) % self.rho.len();
            }
        }
    }
}
struct LbfgsState {
    x: Vector<'static>,
    g: Vector<'static>,
    dir: Vector<'static>,
    next: Vector<'static>,
    next_g: Vector<'static>,
    history: History,
    fx: f64,
    iterations: u32,
}
impl LbfgsState {
    fn initialize(
        f: &mut impl MultipleVarFunctionWithGradient,
        start: &Vector<'_>,
        memory: usize,
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
        if n.checked_mul(memory).is_none() {
            return Err(Status::InvalidInput);
        }
        Ok(Self {
            x,
            g,
            dir: Vector::with_size(n),
            next: Vector::with_size(n),
            next_g: Vector::with_size(n),
            history: History::new(n, memory),
            fx,
            iterations: 0,
        })
    }
    fn finish(self, status: Status) -> VectorResult {
        VectorResult {
            status,
            iterations: self.iterations,
            solution: Some(self.x),
            value: Some(self.fx),
            gradient: Some(self.g),
            ..Default::default()
        }
    }
    fn line(
        &mut self,
        f: &mut impl MultipleVarFunction,
        cfg: &Config,
    ) -> Result<LineSearchResult, Status> {
        self.history.direction(&self.g, &mut self.dir);
        let first: LineSearchResult =
            backtrack(f, &self.x, &self.dir, &self.g, self.fx, 1.0, cfg.step_min);
        if first.is_valid {
            return Ok(first);
        }
        for i in 0..self.x.length() {
            self.dir[i] = -self.g[i];
        }
        let last: LineSearchResult =
            backtrack(f, &self.x, &self.dir, &self.g, self.fx, 1.0, cfg.step_min);
        if !last.is_valid {
            return Err(failure_after_fallback(first.status, last.status));
        }
        self.history.count = 0;
        Ok(last)
    }
    fn accept(&mut self, value: f64) {
        self.x.initialized(&self.next).unwrap();
        self.g.initialized(&self.next_g).unwrap();
        self.fx = value;
    }
    fn iteration(
        &mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        cfg: &Config,
    ) -> Result<Option<Status>, Status> {
        let line: LineSearchResult = self.line(f, cfg)?;
        for i in 0..self.x.length() {
            self.next[i] = self.x[i] + line.alpha * self.dir[i];
        }
        let diff: f64 = max_difference(&self.next, &self.x);
        let status: Status = gradient_status(f, &self.next, &mut self.next_g);
        if status != Status::Ok {
            return Err(status);
        }
        if diff < cfg.x_tolerance {
            let status: Status = if norm(&self.next_g) < cfg.f_tolerance {
                Status::Ok
            } else {
                Status::NotConverged
            };
            self.accept(line.f_new);
            return Ok(Some(status));
        }
        self.history
            .insert(&self.x, &self.next, &self.g, &self.next_g);
        self.accept(line.f_new);
        Ok(None)
    }
    fn run(mut self, f: &mut impl MultipleVarFunctionWithGradient, cfg: &Config) -> VectorResult {
        for iteration in 1..=cfg.max_iterations {
            self.iterations = iteration;
            if norm(&self.g) < cfg.f_tolerance {
                return self.finish(Status::Ok);
            }
            match self.iteration(f, cfg) {
                Ok(Some(status)) | Err(status) => return self.finish(status),
                Ok(None) => {}
            }
        }
        self.finish(Status::MaxIterations)
    }
}
/// Uses native circular history and compensated products; `lbfgs(&mut f,&start,10,&Config::default())`.
pub fn lbfgs(
    f: &mut impl MultipleVarFunctionWithGradient,
    start: &Vector<'_>,
    memory: usize,
    cfg: &Config,
) -> VectorResult {
    if !is_valid_config(cfg) || !is_finite_vector(start) || memory == 0 {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    match LbfgsState::initialize(f, start, memory) {
        Ok(state) => state.run(f, cfg),
        Err(status) => VectorResult {
            status,
            ..Default::default()
        },
    }
}
