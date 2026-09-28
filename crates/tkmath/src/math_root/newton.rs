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

//! MathRoot_Newton.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math::FunctionWithDerivative;
use crate::math_utils::{Config, ScalarResult, Status, clamp};
pub(super) fn valid_config(config: &Config) -> bool {
    config.max_iterations > 0
        && config.x_tolerance.is_finite()
        && config.x_tolerance > 0.0
        && config.f_tolerance.is_finite()
        && config.f_tolerance >= 0.0
}
pub(super) struct NewtonState {
    pub(super) x: f64,
    pub(super) fx: f64,
    pub(super) df: f64,
    pub(super) result: ScalarResult,
}
impl NewtonState {
    pub(super) fn evaluate(&mut self, f: &mut impl FunctionWithDerivative, iteration: u32) -> bool {
        self.result.iterations = iteration + 1;
        let success: bool = f.values(self.x, &mut self.fx, &mut self.df);
        if !success || !self.fx.is_finite() || !self.df.is_finite() {
            self.result.status = if success {
                Status::NumericalError
            } else {
                Status::CallbackError
            };
            self.result.root = Some(self.x);
            return false;
        }
        true
    }
    pub(super) fn finish(mut self, status: Status) -> ScalarResult {
        self.result.status = status;
        self.result.root = Some(self.x);
        self.result.value = Some(self.fx);
        self.result.derivative = Some(self.df);
        self.result
    }
}
/// Preserves native residual and step stopping rules; `newton(&mut f,1.0,&Config::default())`.
pub fn newton(f: &mut impl FunctionWithDerivative, guess: f64, config: &Config) -> ScalarResult {
    if !guess.is_finite() || !valid_config(config) {
        return ScalarResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut state: NewtonState = NewtonState {
        x: guess,
        fx: 0.0,
        df: 0.0,
        result: ScalarResult::default(),
    };
    let mut delta: Option<f64> = None;
    for iteration in 0..config.max_iterations {
        if !state.evaluate(f, iteration) {
            return state.result;
        }
        if state.fx == 0.0
            || (delta.is_some_and(|dx| dx.abs() <= config.x_tolerance)
                && state.fx.abs() <= config.f_tolerance)
        {
            return state.finish(Status::Ok);
        }
        if state.df == 0.0 {
            return state.finish(Status::NumericalError);
        }
        let step: f64 = state.fx / state.df;
        delta = Some(step);
        let next: f64 = state.x - step;
        if !next.is_finite() {
            return state.finish(Status::NumericalError);
        }
        if iteration + 1 < config.max_iterations {
            state.x = next;
        }
    }
    state.finish(Status::MaxIterations)
}
pub(super) struct Bounds {
    pub(super) lower: f64,
    pub(super) upper: f64,
    pub(super) f_lower: f64,
    pub(super) bracketed: bool,
}
fn endpoint(f: &mut impl FunctionWithDerivative, x: f64) -> Result<(f64, f64), Status> {
    let (mut value, mut derivative): (f64, f64) = (0.0, 0.0);
    if !f.values(x, &mut value, &mut derivative) {
        return Err(Status::CallbackError);
    }
    if !value.is_finite() || !derivative.is_finite() {
        return Err(Status::NumericalError);
    }
    Ok((value, derivative))
}
pub(super) fn bounded_endpoints(
    f: &mut impl FunctionWithDerivative,
    lower: f64,
    upper: f64,
    config: &Config,
) -> Result<Bounds, ScalarResult> {
    let fail = |status: Status| ScalarResult {
        status,
        ..Default::default()
    };
    let (fl, dl): (f64, f64) = endpoint(f, lower).map_err(fail)?;
    let (fu, du): (f64, f64) = endpoint(f, upper).map_err(fail)?;
    if fl.abs() <= config.f_tolerance {
        return Err(ScalarResult {
            status: Status::Ok,
            root: Some(lower),
            value: Some(fl),
            derivative: Some(dl),
            ..Default::default()
        });
    }
    if fu.abs() <= config.f_tolerance {
        return Err(ScalarResult {
            status: Status::Ok,
            root: Some(upper),
            value: Some(fu),
            derivative: Some(du),
            ..Default::default()
        });
    }
    Ok(Bounds {
        lower,
        upper,
        f_lower: fl,
        bracketed: fl.is_sign_negative() != fu.is_sign_negative(),
    })
}
impl Bounds {
    fn next(&mut self, state: &NewtonState) -> f64 {
        if self.bracketed {
            if self.f_lower.is_sign_negative() != state.fx.is_sign_negative() {
                self.upper = state.x;
            } else {
                self.lower = state.x;
                self.f_lower = state.fx;
            }
        }
        let mut next: f64 = state.x;
        if state.df != 0.0 {
            next = state.x - state.fx / state.df;
        }
        if !next.is_finite() || next <= self.lower || next >= self.upper {
            next = if self.bracketed {
                0.5 * self.lower + 0.5 * self.upper
            } else {
                clamp(next, self.lower, self.upper)
            };
        }
        next
    }
}
/// Normalizes finite bounds before native safeguarded Newton iteration; `newton_bounded(&mut f,1.0,0.0,2.0,&config)`.
pub fn newton_bounded(
    f: &mut impl FunctionWithDerivative,
    guess: f64,
    mut lower: f64,
    mut upper: f64,
    config: &Config,
) -> ScalarResult {
    if !guess.is_finite()
        || !lower.is_finite()
        || !upper.is_finite()
        || lower == upper
        || !valid_config(config)
    {
        return ScalarResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    if lower > upper {
        std::mem::swap(&mut lower, &mut upper);
    }
    let mut bounds: Bounds = match bounded_endpoints(f, lower, upper, config) {
        Ok(bounds) => bounds,
        Err(result) => return result,
    };
    let mut state: NewtonState = NewtonState {
        x: clamp(guess, lower, upper),
        fx: 0.0,
        df: 0.0,
        result: ScalarResult::default(),
    };
    for iteration in 0..config.max_iterations {
        if !state.evaluate(f, iteration) {
            return state.result;
        }
        if state.fx.abs() <= config.f_tolerance {
            return state.finish(Status::Ok);
        }
        let next: f64 = bounds.next(&state);
        if !next.is_finite() {
            return state.finish(Status::NumericalError);
        }
        if (next - state.x).abs() <= config.x_tolerance && state.fx.abs() <= config.f_tolerance {
            return state.finish(Status::Ok);
        }
        if iteration + 1 < config.max_iterations {
            state.x = next;
        }
    }
    state.finish(Status::MaxIterations)
}
