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

//! `Utils::DeflateReal` and `Utils::DeflateComplex`: checked synthetic division that
//! leaves the working polynomial untouched when the remainder is not small.
use super::Work;
use crate::math_poly::Complex;
/// Divides by `(x - root)`; `degree` must be at least 1.
pub(super) fn deflate_real(c: &mut Work, degree: &mut usize, root: f64, tolerance: f64) -> bool {
    let n: usize = *degree;
    let mut quotient: Work = [0.0; super::WORK_SIZE];
    quotient[n - 1] = c[n];
    for i in (0..n - 1).rev() {
        quotient[i] = c[i + 1] + root * quotient[i + 1];
    }
    let remainder: f64 = c[0] + root * quotient[0];
    let mut magnitude: f64 = c[n].abs();
    for i in (0..n).rev() {
        magnitude = magnitude * root.abs() + c[i].abs();
    }
    if !remainder.is_finite() || remainder.abs() > 32.0 * tolerance * 1.0_f64.max(magnitude) {
        return false;
    }
    c[..n].copy_from_slice(&quotient[..n]);
    *degree = n - 1;
    true
}
/// Quotient of the division by `x^2 + b x + c`, returned with both remainders.
fn quadratic_division(c: &Work, n: usize, b: f64, q: f64) -> (Work, f64, f64) {
    let mut quotient: Work = [0.0; super::WORK_SIZE];
    quotient[n - 2] = c[n];
    if n > 2 {
        quotient[n - 3] = c[n - 1] - b * quotient[n - 2];
    }
    for i in (0..n.saturating_sub(3)).rev() {
        quotient[i] = c[i + 2] - b * quotient[i + 1] - q * quotient[i + 2];
    }
    let tail: f64 = if n > 2 { q * quotient[1] } else { 0.0 };
    let remainder1: f64 = c[1] - b * quotient[0] - tail;
    let remainder0: f64 = c[0] - q * quotient[0];
    (quotient, remainder0, remainder1)
}
/// Divides by the conjugate-pair factor of `root`; `degree` must be at least 2.
pub(super) fn deflate_complex(
    c: &mut Work,
    degree: &mut usize,
    root: Complex,
    tolerance: f64,
) -> bool {
    let b: f64 = -2.0 * root.real();
    let q: f64 = root.norm();
    if !b.is_finite() || !q.is_finite() {
        return false;
    }
    let n: usize = *degree;
    let (quotient, remainder0, remainder1) = quadratic_division(c, n, b, q);
    let norm: f64 = c[..=n]
        .iter()
        .fold(0.0_f64, |m, x| if m < x.abs() { x.abs() } else { m });
    let remainder: f64 = if remainder0.abs() < remainder1.abs() {
        remainder1.abs()
    } else {
        remainder0.abs()
    };
    if !remainder0.is_finite()
        || !remainder1.is_finite()
        || remainder > 64.0 * tolerance * 1.0_f64.max(norm)
    {
        return false;
    }
    c[..=n - 2].copy_from_slice(&quotient[..=n - 2]);
    *degree = n - 2;
    true
}
