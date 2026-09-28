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

//! MathOpt_Brent.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::valid_config;
use crate::math::Function;
use crate::math_utils::{
    BrentState, Config, GOLDEN_SECTION, MinBracketOptions, ScalarResult, Status, ZERO_TOL,
    bracket_minimum,
};
fn scalar_status(f: &mut impl Function, x: f64, value: &mut f64) -> Status {
    if !x.is_finite() {
        return Status::NumericalError;
    }
    if !f.value(x, value) {
        return Status::CallbackError;
    }
    if !value.is_finite() {
        return Status::NumericalError;
    }
    Status::Ok
}
fn invalid_interval(a: f64, b: f64, config: &Config) -> bool {
    !valid_config(config) || !a.is_finite() || !b.is_finite() || a >= b
}
fn finished(status: Status, iterations: u32, x: f64, fx: f64) -> ScalarResult {
    ScalarResult {
        status,
        iterations,
        root: Some(x),
        value: Some(fx),
        ..Default::default()
    }
}
fn failure(status: Status) -> ScalarResult {
    ScalarResult {
        status,
        ..Default::default()
    }
}
/// Minimizes within the native ordered interval; `brent(&mut f,-1.0,1.0,&Config::default())`.
pub fn brent(f: &mut impl Function, a: f64, b: f64, config: &Config) -> ScalarResult {
    if invalid_interval(a, b, config) {
        return failure(Status::InvalidInput);
    }
    let x: f64 = a + GOLDEN_SECTION * (b - a);
    let mut fx: f64 = 0.0;
    let status: Status = scalar_status(f, x, &mut fx);
    if status != Status::Ok {
        return failure(status);
    }
    brent_iterations(f, BrentState::new(a, b, x, fx), config)
}
fn brent_iterations(f: &mut impl Function, mut state: BrentState, config: &Config) -> ScalarResult {
    for iteration in 1..=config.max_iterations {
        let mid: f64 = 0.5 * (state.a + state.b);
        let tol: f64 = config.x_tolerance * state.x.abs() + ZERO_TOL / 10.0;
        if state.converged(mid, 2.0 * tol) {
            return finished(Status::Ok, iteration, state.x, state.fx);
        }
        let u: f64 = state.candidate(mid, tol, false);
        let mut fu: f64 = 0.0;
        let status: Status = scalar_status(f, u, &mut fu);
        if status != Status::Ok {
            return finished(status, iteration, state.x, state.fx);
        }
        state.update(u, fu);
    }
    finished(
        Status::MaxIterations,
        config.max_iterations,
        state.x,
        state.fx,
    )
}
/// Brackets before minimizing, preserving bracket errors; `brent_with_bracket(&mut f,0.0,1.0,&Config::default())`.
pub fn brent_with_bracket(
    f: &mut impl Function,
    guess: f64,
    step: f64,
    config: &Config,
) -> ScalarResult {
    if !valid_config(config)
        || !guess.is_finite()
        || !step.is_finite()
        || step == 0.0
        || !(guess + step).is_finite()
    {
        return failure(Status::InvalidInput);
    }
    let bracket = bracket_minimum(f, guess, guess + step, &MinBracketOptions::default());
    if !bracket.is_valid {
        return failure(bracket.status);
    }
    if !bracket.a.is_finite() || !bracket.c.is_finite() || bracket.a == bracket.c {
        return failure(Status::NumericalError);
    }
    brent(f, bracket.a, bracket.c, config)
}
struct GoldenState {
    a: f64,
    b: f64,
    x1: f64,
    x2: f64,
    f1: f64,
    f2: f64,
}
impl GoldenState {
    fn result(&self, status: Status, iterations: u32) -> ScalarResult {
        let (x, fx): (f64, f64) = if self.f1 < self.f2 {
            (self.x1, self.f1)
        } else {
            (self.x2, self.f2)
        };
        finished(status, iterations, x, fx)
    }
    fn advance(&mut self, f: &mut impl Function, iteration: u32) -> Option<ScalarResult> {
        if self.f1 < self.f2 {
            self.b = self.x2;
            self.x2 = self.x1;
            self.f2 = self.f1;
            self.x1 = self.a + GOLDEN_SECTION * (self.b - self.a);
            let status: Status = scalar_status(f, self.x1, &mut self.f1);
            return (status != Status::Ok).then(|| finished(status, iteration, self.x2, self.f2));
        }
        self.a = self.x1;
        self.x1 = self.x2;
        self.f1 = self.f2;
        self.x2 = self.a + (1.0 - GOLDEN_SECTION) * (self.b - self.a);
        let status: Status = scalar_status(f, self.x2, &mut self.f2);
        (status != Status::Ok).then(|| finished(status, iteration, self.x1, self.f1))
    }
    fn run(mut self, f: &mut impl Function, config: &Config) -> ScalarResult {
        for iteration in 1..=config.max_iterations {
            if self.b - self.a < config.x_tolerance * (self.x1.abs() + self.x2.abs()) {
                return self.result(Status::Ok, iteration);
            }
            if let Some(result) = self.advance(f, iteration) {
                return result;
            }
        }
        self.result(Status::MaxIterations, config.max_iterations)
    }
}
/// Uses golden-section minimization with native tie handling; `golden(&mut f,-1.0,1.0,&Config::default())`.
pub fn golden(f: &mut impl Function, a: f64, b: f64, config: &Config) -> ScalarResult {
    if invalid_interval(a, b, config) {
        return failure(Status::InvalidInput);
    }
    let mut state: GoldenState = GoldenState {
        a,
        b,
        x1: a + GOLDEN_SECTION * (b - a),
        x2: a + (1.0 - GOLDEN_SECTION) * (b - a),
        f1: 0.0,
        f2: 0.0,
    };
    let status: Status = scalar_status(f, state.x1, &mut state.f1);
    if status != Status::Ok {
        return failure(status);
    }
    let status: Status = scalar_status(f, state.x2, &mut state.f2);
    if status != Status::Ok {
        return failure(status);
    }
    state.run(f, config)
}
