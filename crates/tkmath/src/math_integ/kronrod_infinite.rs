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

//! MathInteg_Kronrod.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{
    KronrodConfig,
    kronrod::{invalid, kronrod},
};
use crate::{math::Function, math_utils::IntegResult};
/// Maps a semi-infinite domain onto `[0,1]`; `kronrod_semi_infinite(&mut f, 0.0, &KronrodConfig::default())`.
pub fn kronrod_semi_infinite(
    f: &mut impl Function,
    lower: f64,
    config: &KronrodConfig,
) -> IntegResult {
    if !lower.is_finite() {
        return invalid();
    }
    let mut transformed = |t: f64, value: &mut f64| {
        if t >= 1.0 {
            *value = 0.0;
            return true;
        }
        let x: f64 = lower + t / (1.0 - t);
        let jacobian: f64 = 1.0 / ((1.0 - t) * (1.0 - t));
        transform_value(f, x, jacobian, value)
    };
    kronrod(&mut transformed, 0.0, 1.0, config)
}
/// Maps both infinite tails onto (-1,1); `kronrod_infinite(&mut f, &KronrodConfig::default())`.
pub fn kronrod_infinite(f: &mut impl Function, config: &KronrodConfig) -> IntegResult {
    let mut transformed = |t: f64, value: &mut f64| {
        if t.abs() >= 1.0 {
            *value = 0.0;
            return true;
        }
        let square: f64 = t * t;
        let x: f64 = t / (1.0 - square);
        let jacobian: f64 = (1.0 + square) / ((1.0 - square) * (1.0 - square));
        transform_value(f, x, jacobian, value)
    };
    kronrod(&mut transformed, -1.0, 1.0, config)
}
fn transform_value(f: &mut impl Function, x: f64, jacobian: f64, value: &mut f64) -> bool {
    let mut original: f64 = 0.0;
    if !f.value(x, &mut original) {
        return false;
    }
    *value = original * jacobian;
    true
}
