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

//! Final stage of `MathPoly::Laguerre`: long-double Newton polish of simple real roots
//! and residual verification of every reported root against the undeflated polynomial.
use crate::math_poly::PolyResult;
use crate::math_poly::extended::{LongDouble, scale_double};
use crate::math_poly::utils::{
    LongCoefficients, evaluate, has_small_complex_residual, has_small_residual,
};
use crate::math_utils::Status;
/// Fixed Newton step budget of the native polish loop.
const POLISH_ITERATIONS: u32 = 12;
/// The scaled polynomial in long double and narrowed to double, with its variable exponent.
pub(super) struct Scaled<'a> {
    pub(super) long: &'a LongCoefficients,
    pub(super) original: &'a [f64],
    pub(super) degree: i32,
    pub(super) exponent: i32,
}
fn polished(c: &LongCoefficients, degree: i32, mut root: LongDouble) -> LongDouble {
    for _ in 0..POLISH_ITERATIONS {
        let (value, slope, _) = evaluate(c, degree, root);
        if slope == LongDouble::ZERO || !slope.is_finite() {
            break;
        }
        let step: LongDouble = value / slope;
        if !step.is_finite() {
            break;
        }
        root = root - step;
        if step.abs() <= LongDouble::epsilon() * LongDouble::new(1.0).max(root.abs()) {
            break;
        }
    }
    root
}
/// Polishes roots of multiplicity one in the scaled variable; the result is narrowed
/// without the finiteness check used elsewhere, as in native code.
pub(super) fn polish_simple_roots(
    result: &mut PolyResult,
    long_scaled: &LongCoefficients,
    degree: i32,
    exponent: i32,
) {
    for index in 0..result.nb_roots {
        if result.multiplicities[index] > 1 {
            continue;
        }
        let start: LongDouble = LongDouble::new(result.roots[index]).scale(-exponent);
        result.roots[index] = polished(long_scaled, degree, start)
            .scale(exponent)
            .narrow();
    }
}
/// `Status::Ok` when every real and complex root has a residual within `128 * tolerance`.
pub(super) fn verified_status(result: &PolyResult, scaled: &Scaled, tolerance: f64) -> Status {
    let bound: LongDouble = LongDouble::new(128.0) * LongDouble::new(tolerance);
    let real_ok: bool = result.roots[..result.nb_roots].iter().all(|&root| {
        let scaled_root: LongDouble = LongDouble::new(root).scale(-scaled.exponent);
        has_small_residual(scaled.long, scaled.degree, scaled_root, bound)
    });
    let unscale: f64 = scale_double(1.0, -scaled.exponent);
    let complex_ok: bool = result.complex_roots[..result.nb_complex_roots]
        .iter()
        .all(|&root| {
            has_small_complex_residual(
                scaled.original,
                scaled.degree,
                root * unscale,
                128.0 * tolerance,
            )
        });
    if real_ok && complex_ok {
        Status::Ok
    } else {
        Status::NumericalError
    }
}
