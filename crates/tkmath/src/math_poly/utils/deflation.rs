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

//! `DeflateReal` and `DeflateComplex`: checked synthetic division that leaves the
//! polynomial untouched when the remainder is not small.
use super::preconditions::bounded_degree;
use crate::math_poly::{Complex, MAX_POLY_DEGREE};
use rustycad_tkernel::standard::Failure;
type Quotient = [f64; MAX_POLY_DEGREE + 1];
/// `DeflateReal`: divides `c[0..=degree]` by `(x - root)` when the remainder is within
/// `32 * tolerance * max(1, sum |c_i| |root|^i)`, then decrements `degree`.
/// Standard_OutOfRange unless `1 <= degree <= MAX_POLY_DEGREE` with `degree + 1`
/// coefficients (native code writes `aQuotient[degree - 1]`);
/// `utils::deflate_real(&mut c, &mut degree, 1.0, 1e-12)?`.
pub fn deflate_real(
    coefficients: &mut [f64],
    degree: &mut i32,
    root: f64,
    tolerance: f64,
) -> Result<bool, Failure> {
    let n: usize = bounded_degree(
        "MathPoly::Utils::DeflateReal",
        coefficients.len(),
        *degree,
        1,
    )?;
    let c: &mut [f64] = &mut coefficients[..=n];
    let mut quotient: Quotient = [0.0; MAX_POLY_DEGREE + 1];
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
        return Ok(false);
    }
    c[..n].copy_from_slice(&quotient[..n]);
    *degree -= 1;
    Ok(true)
}
/// Quotient of the division by `x^2 + b x + q`, returned with both remainders.
fn quadratic_division(c: &[f64], n: usize, b: f64, q: f64) -> (Quotient, f64, f64) {
    let mut quotient: Quotient = [0.0; MAX_POLY_DEGREE + 1];
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
/// `std::max(a, b)`: returns `b` only when `a < b`.
fn native_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}
/// `DeflateComplex`: divides by the conjugate-pair factor `x^2 - 2 Re(r) x + |r|^2` when
/// both remainders are within `64 * tolerance * max(1, max |c_i|)`, then subtracts 2 from
/// `degree`. After the finiteness check of the factor, Standard_OutOfRange unless
/// `2 <= degree <= MAX_POLY_DEGREE` with `degree + 1` coefficients (native code writes
/// `aQuotient[degree - 2]`); `utils::deflate_complex(&mut c, &mut degree, root, 1e-12)?`.
pub fn deflate_complex(
    coefficients: &mut [f64],
    degree: &mut i32,
    root: Complex,
    tolerance: f64,
) -> Result<bool, Failure> {
    let b: f64 = -2.0 * root.real();
    let q: f64 = root.norm();
    if !b.is_finite() || !q.is_finite() {
        return Ok(false);
    }
    let n: usize = bounded_degree(
        "MathPoly::Utils::DeflateComplex",
        coefficients.len(),
        *degree,
        2,
    )?;
    let c: &mut [f64] = &mut coefficients[..=n];
    let (quotient, remainder0, remainder1) = quadratic_division(c, n, b, q);
    let norm: f64 = c.iter().fold(0.0_f64, |m, x| native_max(m, x.abs()));
    let remainder: f64 = native_max(remainder0.abs(), remainder1.abs());
    if !remainder0.is_finite()
        || !remainder1.is_finite()
        || remainder > 64.0 * tolerance * 1.0_f64.max(norm)
    {
        return Ok(false);
    }
    c[..=n - 2].copy_from_slice(&quotient[..=n - 2]);
    *degree -= 2;
    Ok(true)
}
