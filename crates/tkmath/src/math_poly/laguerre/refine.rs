// Copyright (c) 2026 OPEN CASCADE SAS
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
use super::consolidate::ScaledPolynomial;
use super::iteration::has_small_residual;
use crate::math_poly::PolyResult;
use crate::math_poly::extended::{Extended as E, scale_double};
use crate::math_poly::long_poly::{LONG_EPSILON, evaluated, small_residual_long};
use crate::math_utils::Status;
/// Fixed Newton step budget of the native polish loop.
const POLISH_ITERATIONS: u32 = 12;
fn polished(c: &[E], mut root: E) -> E {
    for _ in 0..POLISH_ITERATIONS {
        let (value, slope, _) = evaluated(c, root);
        if slope == E::ZERO || !slope.finite() {
            break;
        }
        let step: E = value / slope;
        if !step.finite() {
            break;
        }
        root = root - step;
        if step.abs() <= E::new(LONG_EPSILON) * E::new(1.0).max(root.abs()) {
            break;
        }
    }
    root
}
/// Polishes roots of multiplicity one in the scaled variable; the result is narrowed
/// without the finiteness check used elsewhere, as in native code.
pub(super) fn polish_simple_roots(result: &mut PolyResult, long_scaled: &[E], exponent: i32) {
    for index in 0..result.nb_roots {
        if result.multiplicities[index] > 1 {
            continue;
        }
        let start: E = E::new(result.roots[index]).scale(-exponent);
        result.roots[index] = polished(long_scaled, start).scale(exponent).narrow();
    }
}
/// `Status::Ok` when every real and complex root has a residual within `128 * tolerance`.
pub(super) fn verified_status(
    result: &PolyResult,
    long_scaled: &[E],
    scaled: &ScaledPolynomial,
    tolerance: f64,
) -> Status {
    let bound: E = E::new(128.0) * E::new(tolerance);
    let real_ok: bool = result.roots[..result.nb_roots]
        .iter()
        .all(|&root| small_residual_long(long_scaled, E::new(root).scale(-scaled.exponent), bound));
    let unscale: f64 = scale_double(1.0, -scaled.exponent);
    let complex_ok: bool = result.complex_roots[..result.nb_complex_roots]
        .iter()
        .all(|&root| has_small_residual(scaled.coefficients, root * unscale, 128.0 * tolerance));
    if real_ok && complex_ok {
        Status::Ok
    } else {
        Status::NumericalError
    }
}
