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

//! MathUtils_Deriv.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math::Function;

fn perturbations(x: f64, step: f64) -> Option<(f64, f64)> {
    if !x.is_finite() || !step.is_finite() || step <= 0.0 {
        return None;
    }
    let mut plus: f64 = x + step;
    let mut minus: f64 = x - step;
    if plus == x {
        plus = x.next_up();
    }
    if minus == x {
        minus = x.next_down();
    }
    (minus.is_finite() && plus.is_finite() && minus < x && x < plus).then_some((minus, plus))
}
fn finite_value(f: &mut impl Function, x: f64) -> Option<f64> {
    let mut value: f64 = 0.0;
    let success: bool = f.value(x, &mut value);
    (success && value.is_finite()).then_some(value)
}
fn save_finite(value: f64, output: &mut f64) -> bool {
    if !value.is_finite() {
        return false;
    }
    *output = value;
    true
}
/// Uses representable perturbations and preserves output on failure; `central_difference(&mut f, 2.0, &mut df, 1e-8)`.
pub fn central_difference(f: &mut impl Function, x: f64, derivative: &mut f64, step: f64) -> bool {
    let Some((minus, plus)) = perturbations(x, step) else {
        return false;
    };
    let Some(f_plus) = finite_value(f, plus) else {
        return false;
    };
    let Some(f_minus) = finite_value(f, minus) else {
        return false;
    };
    save_finite((f_plus - f_minus) / (plus - minus), derivative)
}
/// Takes one forward sample using a supplied base value; `forward_difference(&mut f, 2.0, 4.0, &mut df, 1e-8)`.
pub fn forward_difference(
    f: &mut impl Function,
    x: f64,
    fx: f64,
    derivative: &mut f64,
    step: f64,
) -> bool {
    if !x.is_finite() || !fx.is_finite() || !step.is_finite() || step <= 0.0 {
        return false;
    }
    let mut plus: f64 = x + step;
    if plus == x {
        plus = x.next_up();
    }
    if !plus.is_finite() || plus <= x {
        return false;
    }
    let Some(f_plus) = finite_value(f, plus) else {
        return false;
    };
    save_finite((f_plus - fx) / (plus - x), derivative)
}
/// Adapts perturbation to x and respects unequal representable spacing; `second_derivative(&mut f, 2.0, 4.0, &mut d2, 1e-5)`.
pub fn second_derivative(
    f: &mut impl Function,
    x: f64,
    fx: f64,
    derivative: &mut f64,
    step: f64,
) -> bool {
    if !fx.is_finite() {
        return false;
    }
    let Some((minus, plus)) = perturbations(x, step * x.abs().max(1.0)) else {
        return false;
    };
    let Some(f_plus) = finite_value(f, plus) else {
        return false;
    };
    let Some(f_minus) = finite_value(f, minus) else {
        return false;
    };
    let h_plus: f64 = plus - x;
    let h_minus: f64 = x - minus;
    save_finite(
        2.0 * ((f_plus - fx) / h_plus - (fx - f_minus) / h_minus) / (h_plus + h_minus),
        derivative,
    )
}

mod gradient;
pub use gradient::{numerical_gradient, numerical_gradient_adaptive};

mod hessian;
mod jacobian;
pub use hessian::numerical_hessian;
pub use jacobian::numerical_jacobian;
