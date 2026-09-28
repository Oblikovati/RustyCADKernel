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

use super::utils::{have_opposite_signs, is_valid_bounds, is_valid_config};
use crate::math::Function;
use crate::math_utils::{Config, ScalarResult, Status};
pub(super) fn finite_value(
    f: &mut impl Function,
    x: f64,
    value: &mut f64,
    result: &mut ScalarResult,
) -> bool {
    if !f.value(x, value) {
        result.status = Status::CallbackError;
        return false;
    }
    if !value.is_finite() {
        result.status = Status::NumericalError;
        return false;
    }
    true
}
fn endpoint_result(root: f64, value: f64) -> ScalarResult {
    ScalarResult {
        status: Status::Ok,
        root: Some(root),
        value: Some(value),
        ..Default::default()
    }
}
pub(super) fn initialize(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &Config,
) -> Result<[f64; 4], ScalarResult> {
    let mut result: ScalarResult = ScalarResult::default();
    if !is_valid_bounds(lower, upper) || !is_valid_config(config) {
        result.status = Status::InvalidInput;
        return Err(result);
    }
    let (mut fa, mut fb): (f64, f64) = (0.0, 0.0);
    if !finite_value(f, lower, &mut fa, &mut result)
        || !finite_value(f, upper, &mut fb, &mut result)
    {
        return Err(result);
    }
    if fa.abs() <= config.f_tolerance {
        return Err(endpoint_result(lower, fa));
    }
    if fb.abs() <= config.f_tolerance {
        return Err(endpoint_result(upper, fb));
    }
    if !have_opposite_signs(fa, fb) {
        result.status = Status::InvalidInput;
        return Err(result);
    }
    Ok([lower, upper, fa, fb])
}
