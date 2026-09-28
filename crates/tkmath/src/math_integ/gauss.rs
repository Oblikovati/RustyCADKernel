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

//! MathInteg_Gauss.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::{
    math::{Function, Vector, ordered_gauss_points_and_weights},
    math_utils::{IntegResult, Status},
};

/// Highest tabulated Gauss-Legendre order; larger requests are clamped to it
/// (`MathInteg::THE_GAUSS_MAX_ORDER`); `assert_eq!(GAUSS_MAX_ORDER, 61)`.
pub const GAUSS_MAX_ORDER: usize = 61;

/// Native fixed Gauss rule, including order clamping; `gauss(&mut f, 0.0, 1.0, 15)`.
pub fn gauss(f: &mut impl Function, lower: f64, upper: f64, order: usize) -> IntegResult {
    if order == 0 || !valid_interval(lower, upper) {
        return IntegResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    if lower == upper {
        return IntegResult {
            status: Status::Ok,
            value: Some(0.0),
            ..Default::default()
        };
    }
    if lower > upper {
        let mut result: IntegResult = gauss(f, upper, lower, order);
        result.value = result.value.map(|value| -value);
        return result;
    }
    fixed_rule(f, lower, upper, order.min(GAUSS_MAX_ORDER))
}
pub(super) fn valid_interval(lower: f64, upper: f64) -> bool {
    lower.is_finite() && upper.is_finite() && (upper - lower).is_finite()
}
fn fixed_rule(f: &mut impl Function, lower: f64, upper: f64, order: usize) -> IntegResult {
    let mut points: Vector<'static> = Vector::with_bounds(0, order as i32 - 1).unwrap();
    let mut weights: Vector<'static> = Vector::with_bounds(0, order as i32 - 1).unwrap();
    let mut result: IntegResult = IntegResult::default();
    if !ordered_gauss_points_and_weights(order as i32, &mut points, &mut weights) {
        result.status = Status::InvalidInput;
        return result;
    }
    let half: f64 = 0.5 * (upper - lower);
    let mid: f64 = lower + half;
    match accumulate_rule(f, &points, &weights, [mid, half], &mut result) {
        Ok(sum) if (half * sum).is_finite() => {
            result.status = Status::Ok;
            result.value = Some(half * sum);
            result.iterations = 1;
        }
        Ok(_) => result.status = Status::NumericalError,
        Err(status) => result.status = status,
    }
    result
}
fn accumulate_rule(
    f: &mut impl Function,
    points: &Vector<'_>,
    weights: &Vector<'_>,
    transform: [f64; 2],
    result: &mut IntegResult,
) -> Result<f64, Status> {
    let mut sum: f64 = 0.0;
    for i in 0..points.length() as usize {
        let x: f64 = transform[0] + transform[1] * points.at(i).unwrap();
        result.number_of_points += 1;
        let value: f64 = finite_value(f, x)?;
        sum += weights.at(i).unwrap() * value;
        if !sum.is_finite() {
            return Err(Status::NumericalError);
        }
    }
    Ok(sum)
}
pub(super) fn finite_value(f: &mut impl Function, x: f64) -> Result<f64, Status> {
    if !x.is_finite() {
        return Err(Status::NumericalError);
    }
    let mut value: f64 = 0.0;
    if !f.value(x, &mut value) {
        return Err(Status::CallbackError);
    }
    if !value.is_finite() {
        return Err(Status::NumericalError);
    }
    Ok(value)
}
