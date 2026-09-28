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

//! Long-double polynomial kernels of `MathPoly_Utils.hxx`, shared by the direct solvers
//! and by `MathPoly_Laguerre.hxx`. Coefficients are ascending; `c.len() - 1` is the degree.
use super::extended::{Extended as E, exponent};
/// `LDBL_EPSILON` of the pinned x87 profile (64-bit significand).
pub(crate) const LONG_EPSILON: f64 = 1.0 / 9_223_372_036_854_775_808.0;
/// Coefficients of the first derivative, `(long double)i * c[i]`.
pub(crate) fn derivative(c: &[E]) -> Vec<E> {
    (1..c.len()).map(|i| E::new(i as f64) * c[i]).collect()
}
/// `Utils::Evaluate`: Horner value, first derivative and absolute magnitude bound.
pub(crate) fn evaluated(c: &[E], x: E) -> (E, E, E) {
    let mut value: E = *c.last().unwrap();
    let mut derivative: E = E::ZERO;
    let mut magnitude: E = value.abs();
    for coefficient in c[..c.len() - 1].iter().rev() {
        derivative = derivative * x + value;
        value = value * x + *coefficient;
        magnitude = magnitude * x.abs() + coefficient.abs();
    }
    (value, derivative, magnitude)
}
/// `Utils::HasSmallResidual` with `tolerance` given in units of `LDBL_EPSILON`.
pub(crate) fn small_residual(c: &[E], root: E, tolerance: f64) -> bool {
    small_residual_long(c, root, E::new(tolerance * LONG_EPSILON))
}
/// `Utils::HasSmallResidual` with an absolute long-double tolerance.
pub(crate) fn small_residual_long(c: &[E], root: E, tolerance: E) -> bool {
    let (value, _, magnitude) = evaluated(c, root);
    value.finite() && value.abs() <= tolerance * E::new(1.0).max(magnitude)
}
fn variable_exponent(c: &[f64]) -> i32 {
    let degree: usize = c.len() - 1;
    let leading: i32 = exponent(c[degree].abs());
    let mut scale: E = E::new(-1000.0);
    for (i, &coefficient) in c.iter().enumerate().take(degree) {
        if coefficient == 0.0 {
            continue;
        }
        let difference: i32 = exponent(coefficient.abs()) - leading;
        scale = scale.max(E::new(difference as f64) / E::new((degree - i) as f64));
    }
    scale.ceil_integer().clamp(-1000, 1000)
}
/// `Utils::ScalePolynomial`: x = 2^power * y, coefficients normalized by a common power of two.
pub(crate) fn scaled_polynomial(c: &[f64]) -> (Vec<E>, i32) {
    let power: i32 = variable_exponent(c);
    let mut common: i32 = exponent(c[c.len() - 1].abs()) + (c.len() - 1) as i32 * power;
    for (i, &value) in c.iter().enumerate() {
        if value != 0.0 {
            common = common.max(exponent(value.abs()) + i as i32 * power);
        }
    }
    (
        c.iter()
            .enumerate()
            .map(|(i, &value)| E::new(value).scale(i as i32 * power - common))
            .collect(),
        power,
    )
}
