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

//! MathRoot_Bisection.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::bracket::{finite_value, initialize};
use super::newton::{Bounds, NewtonState, bounded_endpoints};
use super::utils::{have_opposite_signs, is_valid_bounds, is_valid_config, midpoint};
use crate::math::{Function, FunctionWithDerivative};
use crate::math_utils::{Config, ScalarResult, Status};
fn finish(mut result: ScalarResult, status: Status, x: f64, value: f64) -> ScalarResult {
    result.status = status;
    result.root = Some(x);
    result.value = Some(value);
    result
}
/// Bisects an ordered sign bracket; `bisection(&mut f,0.0,2.0,&Config::default())`.
pub fn bisection(f: &mut impl Function, lower: f64, upper: f64, config: &Config) -> ScalarResult {
    let [mut a, mut b, mut fa, mut fb]: [f64; 4] = match initialize(f, lower, upper, config) {
        Ok(values) => values,
        Err(result) => return result,
    };
    let mut result: ScalarResult = ScalarResult::default();
    for iteration in 0..config.max_iterations {
        let mid: f64 = midpoint(a, b);
        let mut value: f64 = 0.0;
        result.iterations = iteration + 1;
        if !finite_value(f, mid, &mut value, &mut result) {
            result.root = Some(mid);
            return result;
        }
        if value.abs() <= config.f_tolerance || 0.5 * (b - a).abs() <= config.x_tolerance {
            return finish(result, Status::Ok, mid, value);
        }
        if mid == a || mid == b {
            return finish(result, Status::NotConverged, mid, value);
        }
        if have_opposite_signs(fa, value) {
            b = mid;
            fb = value;
        } else {
            a = mid;
            fa = value;
        }
    }
    let (x, value): (f64, f64) = if fa.abs() <= fb.abs() {
        (a, fa)
    } else {
        (b, fb)
    };
    finish(result, Status::MaxIterations, x, value)
}
fn hybrid_update(bounds: &mut Bounds, state: &NewtonState) {
    if have_opposite_signs(bounds.f_lower, state.fx) {
        bounds.upper = state.x;
    } else {
        bounds.lower = state.x;
        bounds.f_lower = state.fx;
    }
}
fn hybrid_next(bounds: &Bounds, state: &NewtonState, tolerance: f64) -> f64 {
    let middle: f64 = midpoint(bounds.lower, bounds.upper);
    if state.df == 0.0 {
        return middle;
    }
    let newton: f64 = state.x - state.fx / state.df;
    if newton.is_finite()
        && newton > bounds.lower
        && newton < bounds.upper
        && (newton - state.x).abs() > tolerance
    {
        newton
    } else {
        middle
    }
}
/// Combines safeguarded Newton steps with interval localization; `bisection_newton(&mut f,0.0,2.0,&config)`.
pub fn bisection_newton(
    f: &mut impl FunctionWithDerivative,
    lower: f64,
    upper: f64,
    config: &Config,
) -> ScalarResult {
    if !is_valid_bounds(lower, upper) || !is_valid_config(config) {
        return ScalarResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut bounds: Bounds = match bounded_endpoints(f, lower, upper, config) {
        Ok(bounds) => bounds,
        Err(result) => return result,
    };
    if !bounds.bracketed {
        return ScalarResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut state: NewtonState = NewtonState {
        x: midpoint(lower, upper),
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
        hybrid_update(&mut bounds, &state);
        if 0.5 * (bounds.upper - bounds.lower).abs() <= config.x_tolerance {
            return state.finish(Status::Ok);
        }
        let next: f64 = hybrid_next(&bounds, &state, config.x_tolerance);
        if next == state.x || next == bounds.lower || next == bounds.upper {
            return state.finish(Status::NotConverged);
        }
        if iteration + 1 < config.max_iterations {
            state.x = next;
        }
    }
    state.finish(Status::MaxIterations)
}
