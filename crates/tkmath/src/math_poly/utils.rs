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
// Rust translation of FoundationClasses/TKMath/MathPoly/MathPoly_Utils.hxx and the Utils part of
// MathPoly_Laguerre.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MathPoly::Utils`: the helpers behind the direct and Laguerre polynomial solvers.
//!
//! Native pointers become slices plus the native `int` degree; a slice shorter than
//! `degree + 1`, a degree outside `0..=MAX_POLY_DEGREE` and root buffers that would
//! overflow are undefined behavior upstream and raise Standard_OutOfRange here, with the
//! offending sizes and the expected shape.
//! `long double` is [`LongDouble`], the pinned x87 profile. Output references become
//! return values or `&mut` parameters.
use super::extended::{LongDouble, exponent};
use super::{Complex, MAX_POLY_DEGREE};
use rustycad_tkernel::standard::Failure;
mod consolidate;
mod deflation;
mod direct;
mod isolation;
mod iteration;
mod multiplicity;
mod preconditions;
pub use consolidate::{consolidate_complex_roots, consolidate_real_roots};
pub use deflation::{deflate_complex, deflate_real};
pub use direct::solve_direct;
pub use isolation::{append_unique, solve_distinct_real};
pub(super) use iteration::small_complex_residual;
pub use iteration::{
    ComplexEvaluation, IterationResult, LAGUERRE_NB_STARTS, evaluate_complex, find_root,
    has_small_complex_residual, laguerre_iteration,
};
pub use multiplicity::{
    MULTIPLICITY_REFINEMENT_MAX_ITERATIONS, has_complex_multiplicity, has_multiplicity,
};
use preconditions::{bounded_degree, fixed_degree};
/// `THE_DISTINCT_ROOT_BISECTION_ITERATIONS`: bisection steps per bracketed root.
pub const DISTINCT_ROOT_BISECTION_ITERATIONS: u32 = 200;
/// `LongCoefficients`: ascending long-double coefficients, `THE_MAX_POLY_DEGREE + 1` slots.
pub type LongCoefficients = [LongDouble; MAX_POLY_DEGREE + 1];
/// `THE_DISTINCT_ROOT_RESIDUAL_TOLERANCE` = `64 * LDBL_EPSILON`;
/// `utils::distinct_root_residual_tolerance()`.
pub fn distinct_root_residual_tolerance() -> LongDouble {
    LongDouble::new(64.0) * LongDouble::epsilon()
}
/// `THE_ROOT_MULTIPLICITY_TOLERANCE` = `32768 * LDBL_EPSILON`;
/// `utils::root_multiplicity_tolerance()`.
pub fn root_multiplicity_tolerance() -> LongDouble {
    LongDouble::new(32768.0) * LongDouble::epsilon()
}
/// A zero-filled `LongCoefficients` (`LongCoefficients aValue = {}`); `utils::long_coefficients()`.
pub fn long_coefficients() -> LongCoefficients {
    [LongDouble::ZERO; MAX_POLY_DEGREE + 1]
}
/// `IsFinite(std::complex<double>)`; `utils::is_finite(Complex::new(1.0, 0.0))`.
pub fn is_finite(value: Complex) -> bool {
    value.is_finite()
}
/// `ToDouble`: `None` when the value or its double rounding is not finite;
/// `utils::to_double(LongDouble::new(0.5))`.
pub fn to_double(value: LongDouble) -> Option<f64> {
    if !value.is_finite() {
        return None;
    }
    let narrowed: f64 = value.narrow();
    narrowed.is_finite().then_some(narrowed)
}
/// `Validate`: `None` models a null pointer. The degree must lie in `0..=MAX_POLY_DEGREE`,
/// the tolerance must be finite and positive, and coefficients `0..=degree` finite. A slice
/// shorter than `degree + 1` is rejected, where native code would read past the caller's
/// array; `utils::validate(Some(&[1.0, 2.0]), 1, 1.0)`.
pub fn validate(coefficients: Option<&[f64]>, degree: i32, tolerance: f64) -> bool {
    let Some(coefficients) = coefficients else {
        return false;
    };
    if degree < 0 || degree > MAX_POLY_DEGREE as i32 || !tolerance.is_finite() || tolerance <= 0.0 {
        return false;
    }
    let size: usize = degree as usize + 1;
    coefficients.len() >= size && coefficients[..size].iter().all(|x| x.is_finite())
}
/// `ScalePolynomial`: substitutes `x = 2^e * y` and normalizes by a common power of two so
/// the coefficient exponents are balanced. `None` when the leading coefficient is zero.
/// Integer exponent arithmetic wraps for non-finite input, which native code leaves
/// undefined. Standard_OutOfRange for a degree outside `0..=MAX_POLY_DEGREE` or fewer than
/// `degree + 1` coefficients; `utils::scale_polynomial(&[1.0, 0.0, 4.0], 2)?`.
pub fn scale_polynomial(
    coefficients: &[f64],
    degree: i32,
) -> Result<Option<(LongCoefficients, i32)>, Failure> {
    bounded_degree(
        "MathPoly::Utils::ScalePolynomial",
        coefficients.len(),
        degree,
        0,
    )?;
    Ok(scaled(coefficients, degree))
}
/// [`scale_polynomial`] on a degree the caller has validated.
pub(super) fn scaled(coefficients: &[f64], degree: i32) -> Option<(LongCoefficients, i32)> {
    let c: &[f64] = &coefficients[..=degree as usize];
    if c[degree as usize] == 0.0 {
        return None;
    }
    let power: i32 = variable_exponent(c);
    let mut common: i32 = exponent(c[c.len() - 1].abs()).wrapping_add(degree.wrapping_mul(power));
    for (i, &value) in c.iter().enumerate() {
        if value != 0.0 {
            common = common.max(exponent(value.abs()).wrapping_add(i as i32 * power));
        }
    }
    let mut scaled: LongCoefficients = long_coefficients();
    for (i, &value) in c.iter().enumerate() {
        scaled[i] = LongDouble::new(value).scale((i as i32 * power).wrapping_sub(common));
    }
    Some((scaled, power))
}
fn variable_exponent(c: &[f64]) -> i32 {
    let degree: usize = c.len() - 1;
    let leading: i32 = exponent(c[degree].abs());
    let mut scale: LongDouble = LongDouble::new(-1000.0);
    for (i, &coefficient) in c.iter().enumerate().take(degree) {
        if coefficient == 0.0 {
            continue;
        }
        let difference: i32 = exponent(coefficient.abs()).wrapping_sub(leading);
        let ratio = LongDouble::new(difference as f64) / LongDouble::new((degree - i) as f64);
        scale = scale.max(ratio);
    }
    scale.ceil_integer().clamp(-1000, 1000)
}
/// `Evaluate`: Horner value, first derivative and the absolute magnitude bound
/// `sum |c_i| |x|^i`; Standard_OutOfRange for a degree outside `0..=MAX_POLY_DEGREE`;
/// `utils::evaluate(&c, 2, LongDouble::new(1.0))?`.
pub fn evaluate(
    coefficients: &LongCoefficients,
    degree: i32,
    x: LongDouble,
) -> Result<(LongDouble, LongDouble, LongDouble), Failure> {
    fixed_degree("MathPoly::Utils::Evaluate", degree, 0)?;
    Ok(horner(coefficients, degree, x))
}
/// [`evaluate`] on a degree the caller has validated.
pub(super) fn horner(
    coefficients: &LongCoefficients,
    degree: i32,
    x: LongDouble,
) -> (LongDouble, LongDouble, LongDouble) {
    let d: usize = degree as usize;
    let mut value: LongDouble = coefficients[d];
    let mut derivative: LongDouble = LongDouble::ZERO;
    let mut magnitude: LongDouble = coefficients[d].abs();
    let abs_x: LongDouble = x.abs();
    for &coefficient in coefficients[..d].iter().rev() {
        derivative = derivative * x + value;
        value = value * x + coefficient;
        magnitude = magnitude * abs_x + coefficient.abs();
    }
    (value, derivative, magnitude)
}
/// `HasSmallResidual(LongCoefficients, ...)`: `|p(root)| <= tolerance * max(1, magnitude)`
/// with a finite value; Standard_OutOfRange for a degree outside `0..=MAX_POLY_DEGREE`;
/// `utils::has_small_residual(&c, 2, root, tolerance)?`.
pub fn has_small_residual(
    coefficients: &LongCoefficients,
    degree: i32,
    root: LongDouble,
    tolerance: LongDouble,
) -> Result<bool, Failure> {
    fixed_degree("MathPoly::Utils::HasSmallResidual", degree, 0)?;
    Ok(small_residual(coefficients, degree, root, tolerance))
}
/// [`has_small_residual`] on a degree the caller has validated.
pub(super) fn small_residual(
    coefficients: &LongCoefficients,
    degree: i32,
    root: LongDouble,
    tolerance: LongDouble,
) -> bool {
    let (value, _, magnitude) = horner(coefficients, degree, root);
    value.is_finite() && value.abs() <= tolerance * LongDouble::new(1.0).max(magnitude)
}
/// `RootBound`: Cauchy bound `1 + max |c_i / c_degree|`; Standard_OutOfRange for a degree
/// outside `0..=MAX_POLY_DEGREE`; `utils::root_bound(&c, 2)?`.
pub fn root_bound(coefficients: &LongCoefficients, degree: i32) -> Result<LongDouble, Failure> {
    fixed_degree("MathPoly::Utils::RootBound", degree, 0)?;
    Ok(cauchy_bound(coefficients, degree))
}
/// [`root_bound`] on a degree the caller has validated.
pub(super) fn cauchy_bound(coefficients: &LongCoefficients, degree: i32) -> LongDouble {
    let d: usize = degree as usize;
    let leading: LongDouble = coefficients[d].abs();
    let ratio: LongDouble = coefficients[..d]
        .iter()
        .fold(LongDouble::ZERO, |s, x| s.max(x.abs() / leading));
    LongDouble::new(1.0) + ratio
}
/// The first derivative's coefficients, `(long double)i * c[i]`, in a zero-filled array.
pub(super) fn derivative(coefficients: &LongCoefficients, degree: i32) -> LongCoefficients {
    let mut next: LongCoefficients = long_coefficients();
    for i in 1..=degree.max(0) as usize {
        next[i - 1] = LongDouble::new(i as f64) * coefficients[i];
    }
    next
}
/// Widens `coefficients[0..=degree]` into a zero-filled `LongCoefficients`.
pub(super) fn widened(coefficients: &[f64], degree: i32) -> LongCoefficients {
    let mut long: LongCoefficients = long_coefficients();
    for (slot, &value) in long.iter_mut().zip(&coefficients[..=degree as usize]) {
        *slot = LongDouble::new(value);
    }
    long
}
