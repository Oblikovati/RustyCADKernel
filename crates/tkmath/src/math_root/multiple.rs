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

//! Multiple root search preserving MathRoot_MultipleUtils.hxx evaluation order.
use crate::math::{Function, FunctionWithDerivative};
use crate::math_utils::{Config, ScalarResult, Status};
mod derivative;
mod extrema;
/// Native multi-root configuration; `MultipleConfig::default()`.
#[derive(Clone, Copy)]
pub struct MultipleConfig {
    /// Initial sampling request, doubled internally; `config.nb_samples`.
    pub nb_samples: usize,
    /// Root-position tolerance; `config.x_tolerance`.
    pub x_tolerance: f64,
    /// Function residual tolerance; `config.f_tolerance`.
    pub f_tolerance: f64,
    /// All-null classification threshold; `config.null_tolerance`.
    pub null_tolerance: f64,
    /// Brent iterations per refinement; `config.max_iterations`.
    pub max_iterations: u32,
    /// Requested function level; `config.offset`.
    pub offset: f64,
}
impl Default for MultipleConfig {
    fn default() -> Self {
        Self {
            nb_samples: 100,
            x_tolerance: 1e-10,
            f_tolerance: 1e-10,
            null_tolerance: 1e-12,
            max_iterations: 100,
            offset: 0.0,
        }
    }
}
/// Found roots retain native insertion order on early failure; `MultipleResult::default()`.
#[derive(Default)]
pub struct MultipleResult {
    /// Native completion status; `result.status`.
    pub status: Status,
    /// Total Brent iterations; `result.iterations`.
    pub iterations: u32,
    /// Root locations; `result.roots`.
    pub roots: Vec<f64>,
    /// Original function values at each root; `result.values`.
    pub values: Vec<f64>,
    /// All samples satisfy null tolerance; `result.is_all_null`.
    pub is_all_null: bool,
}
impl MultipleResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
    /// Counts stored roots; `result.nb_roots()`.
    pub fn nb_roots(&self) -> usize {
        self.roots.len()
    }
    fn failed(status: Status) -> Self {
        Self {
            status,
            ..Default::default()
        }
    }
    fn add(&mut self, epsilon: f64, root: f64, value: f64) {
        if self.roots.iter().any(|old| (root - old).abs() < epsilon) {
            return;
        }
        self.roots.push(root);
        self.values.push(value);
    }
    fn sort(&mut self) {
        for i in 1..self.roots.len() {
            let root: f64 = self.roots[i];
            let value: f64 = self.values[i];
            let mut j: usize = i;
            while j > 0 && self.roots[j - 1] > root {
                self.roots[j] = self.roots[j - 1];
                self.values[j] = self.values[j - 1];
                j -= 1;
            }
            self.roots[j] = root;
            self.values[j] = value;
        }
    }
}
impl From<&MultipleResult> for bool {
    fn from(result: &MultipleResult) -> bool {
        result.is_done()
    }
}
impl std::ops::Index<usize> for MultipleResult {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        &self.roots[index]
    }
}
fn valid_input(lower: f64, upper: f64, cfg: &MultipleConfig) -> bool {
    lower.is_finite()
        && upper.is_finite()
        && lower < upper
        && cfg.nb_samples >= 2
        && cfg.max_iterations > 0
        && cfg.x_tolerance.is_finite()
        && cfg.x_tolerance > 0.0
        && cfg.f_tolerance.is_finite()
        && cfg.f_tolerance >= 0.0
        && cfg.null_tolerance.is_finite()
        && cfg.null_tolerance >= 0.0
        && cfg.offset.is_finite()
}
fn effective_tolerance(lower: f64, upper: f64, tolerance: f64) -> f64 {
    let maximum: f64 = lower.abs().max(upper.abs());
    let minimum: f64 = lower.abs().min(upper.abs());
    if maximum == 0.0 {
        return tolerance;
    }
    tolerance.max(1e-10 * maximum * (1.0 + minimum / maximum))
}
fn finite(value: f64) -> Result<f64, Status> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Status::NumericalError)
    }
}
fn opposite(a: f64, b: f64) -> bool {
    a.is_sign_negative() != b.is_sign_negative()
}
struct Search<'a, F> {
    function: &'a mut F,
    config: MultipleConfig,
    lower: f64,
    upper: f64,
    count: usize,
    epsilon: f64,
    dx: f64,
    xs: Vec<f64>,
    ys: Vec<f64>,
    result: MultipleResult,
}
impl<'a, F: Function> Search<'a, F> {
    fn new(
        function: &'a mut F,
        lower: f64,
        upper: f64,
        config: &MultipleConfig,
    ) -> Result<Self, Status> {
        if !valid_input(lower, upper, config) {
            return Err(Status::InvalidInput);
        }
        let count: usize = config.nb_samples.wrapping_mul(2).max(20);
        Ok(Self {
            function,
            config: *config,
            lower,
            upper,
            count,
            epsilon: effective_tolerance(lower, upper, config.x_tolerance),
            dx: upper / count as f64 - lower / count as f64,
            xs: Vec::with_capacity(count + 1),
            ys: Vec::with_capacity(count + 1),
            result: MultipleResult::failed(Status::Ok),
        })
    }
    fn value(&mut self, x: f64) -> Result<f64, Status> {
        let mut value: f64 = 0.0;
        if !self.function.value(x, &mut value) {
            return Err(Status::CallbackError);
        }
        finite(value)
    }
    fn shifted_into(&mut self, x: f64, value: &mut f64) -> Result<(), Status> {
        if !self.function.value(x, value) {
            return Err(Status::CallbackError);
        }
        finite(*value)?;
        *value -= self.config.offset;
        finite(*value)?;
        Ok(())
    }
    fn shifted(&mut self, x: f64) -> Result<f64, Status> {
        let mut value: f64 = 0.0;
        self.shifted_into(x, &mut value)?;
        Ok(value)
    }
    fn sample(&mut self) -> Result<(), Status> {
        for i in 0..=self.count {
            let ratio: f64 = i as f64 / self.count as f64;
            let x: f64 = (1.0 - ratio) * self.lower + ratio * self.upper;
            let value: f64 = self.shifted(x)?;
            self.xs.push(x);
            self.ys.push(value);
        }
        self.result.is_all_null = self
            .ys
            .iter()
            .all(|y| y.abs() <= self.config.null_tolerance);
        Ok(())
    }
    fn configuration(&self, tolerance: f64) -> Config {
        Config {
            x_tolerance: tolerance,
            f_tolerance: self.config.f_tolerance,
            max_iterations: self.config.max_iterations,
            ..Default::default()
        }
    }
    fn bracket(&mut self, lower: f64, upper: f64) -> Result<f64, Status> {
        let config: Config = self.configuration(self.epsilon);
        let offset: f64 = self.config.offset;
        let mut wrapper = |x: f64, out: &mut f64| {
            if !self.function.value(x, out) {
                return false;
            }
            *out -= offset;
            true
        };
        let refined: ScalarResult = super::brent(&mut wrapper, lower, upper, &config);
        self.result.iterations = self.result.iterations.wrapping_add(refined.iterations);
        if !refined.is_done() {
            return Err(refined.status);
        }
        refined.root.ok_or(refined.status)
    }
    fn add_at(&mut self, x: f64) -> Result<(), Status> {
        let value: f64 = self.value(x)?;
        self.result.add(self.epsilon, x, value);
        Ok(())
    }
    fn value_interval(&mut self, i: usize) -> Result<(), Status> {
        if self.ys[i].abs() <= self.config.f_tolerance {
            return self.add_at(self.xs[i]);
        }
        if opposite(self.ys[i], self.ys[i + 1]) {
            let root: f64 = self.bracket(self.xs[i], self.xs[i + 1])?;
            self.add_at(root)?;
        }
        Ok(())
    }
    fn value_search(&mut self) -> Result<(), Status> {
        self.sample()?;
        if self.result.is_all_null {
            return Ok(());
        }
        for i in 0..self.count {
            self.value_interval(i)?;
        }
        if self.ys[self.count].abs() <= self.config.f_tolerance {
            self.add_at(self.xs[self.count])?;
        }
        self.result.sort();
        Ok(())
    }
    fn finish(mut self, status: Result<(), Status>) -> MultipleResult {
        if let Err(status) = status {
            self.result.status = status;
        }
        self.result
    }
}
/// Finds crossings and sampled zeros; `find_all_roots(&mut function,0.0,1.0,&MultipleConfig::default())`.
pub fn find_all_roots(
    function: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &MultipleConfig,
) -> MultipleResult {
    let mut search = match Search::new(function, lower, upper, config) {
        Ok(search) => search,
        Err(status) => return MultipleResult::failed(status),
    };
    let status: Result<(), Status> = search.value_search();
    search.finish(status)
}
/// Uses the native convenience sampling count; `find_all_roots_samples(&mut function,0.0,1.0,50)`.
pub fn find_all_roots_samples(
    function: &mut impl Function,
    lower: f64,
    upper: f64,
    samples: usize,
) -> MultipleResult {
    find_all_roots(
        function,
        lower,
        upper,
        &MultipleConfig {
            nb_samples: samples,
            ..Default::default()
        },
    )
}
/// Includes native derivative extrema heuristics; `find_all_roots_with_derivative(&mut function,0.0,1.0,&MultipleConfig::default())`.
pub fn find_all_roots_with_derivative(
    function: &mut impl FunctionWithDerivative,
    lower: f64,
    upper: f64,
    config: &MultipleConfig,
) -> MultipleResult {
    let mut search = match Search::new(function, lower, upper, config) {
        Ok(search) => search,
        Err(status) => return MultipleResult::failed(status),
    };
    let status: Result<(), Status> = search.derivative_search();
    search.finish(status)
}
