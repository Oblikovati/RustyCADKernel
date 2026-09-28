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

//! `MathPoly_Laguerre.hxx`: checked Laguerre iteration with deflation for degree <= 20.
use super::extended::{Extended as E, scale_double};
use super::long_poly::scaled_polynomial;
use super::{Complex, GeneralPolyResult, MAX_POLY_DEGREE, PolyResult, quartic};
use crate::math_utils::Status;
mod consolidate;
mod deflation;
mod iteration;
mod multiplicity;
mod refine;
use consolidate::{ScaledPolynomial, consolidate_complex_roots, consolidate_real_roots};
use deflation::{deflate_complex, deflate_real};
use iteration::{IterationResult, find_root, has_small_residual};
use multiplicity::has_multiplicity;
/// `Precision::Computational()`, i.e. `RealEpsilon()` = `DBL_EPSILON`.
const COMPUTATIONAL: f64 = f64::EPSILON;
const WORK_SIZE: usize = MAX_POLY_DEGREE + 1;
/// Working coefficient storage of the scaled, progressively deflated polynomial.
type Work = [f64; WORK_SIZE];
/// Default relative residual tolerance of `MathPoly::Laguerre`; `LAGUERRE_TOLERANCE`.
pub const LAGUERRE_TOLERANCE: f64 = 1.0e-12;
/// Default per-start iteration limit of `MathPoly::Laguerre`; `LAGUERRE_MAX_ITERATIONS`.
pub const LAGUERRE_MAX_ITERATIONS: u32 = 100;
/// `Utils::Validate`. Rust also rejects a slice shorter than `degree + 1`, where native
/// code would read past the caller's array.
fn is_valid(c: &[f64], degree: i32, tolerance: f64) -> bool {
    if degree < 0 || degree > MAX_POLY_DEGREE as i32 || !tolerance.is_finite() || tolerance <= 0.0 {
        return false;
    }
    let size: usize = degree as usize + 1;
    c.len() >= size && c[..size].iter().all(|x| x.is_finite())
}
/// Solves a real-coefficient polynomial for all real and complex roots.
///
/// Coefficients are ascending; `tolerance` is a positive relative residual tolerance and
/// `max_iterations` the per-start iteration limit. Failures keep the roots found so far.
/// `laguerre(&[-6.0, 11.0, -6.0, 1.0], 3, LAGUERRE_TOLERANCE, LAGUERRE_MAX_ITERATIONS)`
/// finds 1, 2 and 3.
pub fn laguerre(
    coefficients: &[f64],
    degree: i32,
    tolerance: f64,
    max_iterations: u32,
) -> PolyResult {
    if !is_valid(coefficients, degree, tolerance)
        || degree < 1
        || coefficients[degree as usize] == 0.0
    {
        return PolyResult::failed(Status::InvalidInput);
    }
    let settings: Deflation = Deflation {
        tolerance,
        max_iterations,
        exponent: 0,
    };
    solve_valid(&coefficients[..=degree as usize], settings)
}
/// Narrows the long-double scaled coefficients into the double working polynomial.
fn narrowed(long_scaled: &[E]) -> Work {
    let mut work: Work = [0.0; WORK_SIZE];
    for (slot, value) in work.iter_mut().zip(long_scaled) {
        *slot = value.narrow();
    }
    work
}
/// Solves a validated polynomial whose last coefficient is nonzero.
fn solve_valid(coefficients: &[f64], mut settings: Deflation) -> PolyResult {
    let (long_scaled, exponent) = scaled_polynomial(coefficients);
    settings.exponent = exponent;
    let original: Work = narrowed(&long_scaled);
    let mut result: PolyResult = PolyResult::failed(Status::NotConverged);
    if let Err(status) = settings.extract_roots(&mut result, original, coefficients.len() - 1) {
        result.status = status;
        return result;
    }
    let scaled: ScaledPolynomial = ScaledPolynomial {
        coefficients: &original[..coefficients.len()],
        exponent,
    };
    consolidate_real_roots(&mut result, settings.tolerance, &scaled);
    consolidate_complex_roots(&mut result, settings.tolerance, &scaled);
    refine::polish_simple_roots(&mut result, &long_scaled, exponent);
    result.status = refine::verified_status(&result, &long_scaled, &scaled, settings.tolerance);
    result
}
/// Solver settings and the variable exponent shared by the deflation branches.
struct Deflation {
    tolerance: f64,
    max_iterations: u32,
    exponent: i32,
}
impl Deflation {
    /// Deflation loop; returns the failing status while keeping roots already accepted.
    fn extract_roots(
        &self,
        result: &mut PolyResult,
        mut work: Work,
        mut degree: usize,
    ) -> Result<(), Status> {
        while degree > 0 {
            let found: IterationResult =
                find_root(&work[..=degree], self.tolerance, self.max_iterations);
            if found.status != Status::Ok || !found.root.is_finite() {
                return Err(found.status);
            }
            let uncertainty: f64 =
                found.root_error.unwrap() + COMPUTATIONAL * 1.0_f64.max(found.root.abs());
            if found.root.imag().abs() <= uncertainty {
                self.real_root(result, &mut work, &mut degree, &found)?;
            } else {
                self.complex_root(result, &mut work, &mut degree, found.root)?;
            }
        }
        Ok(())
    }
    /// Native `ToDouble(scalbn((long double)root, exponent))`.
    fn unscaled(&self, root: f64) -> Result<f64, Status> {
        E::new(root)
            .scale(self.exponent)
            .double()
            .ok_or(Status::NumericalError)
    }
    /// Refines through a double-root check when the iteration error is large.
    fn real_root(
        &self,
        result: &mut PolyResult,
        work: &mut Work,
        degree: &mut usize,
        found: &IterationResult,
    ) -> Result<(), Status> {
        let mut root: f64 = found.root.real();
        let mut repeated: f64 = root;
        if found.root_error.unwrap() > self.tolerance * 1.0_f64.max(root.abs())
            && *degree > 1
            && has_multiplicity(&work[..=*degree], &mut repeated, 2)
        {
            root = repeated;
        }
        if !has_small_residual(&work[..=*degree], Complex::new(root, 0.0), self.tolerance)
            || !deflate_real(work, degree, root, self.tolerance)
        {
            return Err(Status::NumericalError);
        }
        result.append(self.unscaled(root)?, 1);
        Ok(())
    }
    /// Prefers deflating the real part when it is itself an acceptable root.
    fn complex_root(
        &self,
        result: &mut PolyResult,
        work: &mut Work,
        degree: &mut usize,
        root: Complex,
    ) -> Result<(), Status> {
        if *degree < 2 || !has_small_residual(&work[..=*degree], root, self.tolerance) {
            return Err(Status::NumericalError);
        }
        let mut real_root: f64 = root.real();
        let mut repeated: f64 = real_root;
        if has_multiplicity(&work[..=*degree], &mut repeated, 2) {
            real_root = repeated;
        }
        let mut real_work: Work = *work;
        let mut real_degree: usize = *degree;
        if has_small_residual(
            &work[..=*degree],
            Complex::new(real_root, 0.0),
            self.tolerance,
        ) && deflate_real(&mut real_work, &mut real_degree, real_root, self.tolerance)
        {
            let unscaled: f64 = self.unscaled(real_root)?;
            (*work, *degree) = (real_work, real_degree);
            result.append(unscaled, 1);
            return Ok(());
        }
        self.conjugate_pair(result, work, degree, root)
    }
    fn conjugate_pair(
        &self,
        result: &mut PolyResult,
        work: &mut Work,
        degree: &mut usize,
        root: Complex,
    ) -> Result<(), Status> {
        if !deflate_complex(work, degree, root, self.tolerance) {
            return Err(Status::NumericalError);
        }
        let unscaled: Complex = root * scale_double(1.0, self.exponent);
        if !unscaled.is_finite() {
            return Err(Status::NumericalError);
        }
        let positive: Complex = if unscaled.imag() > 0.0 {
            unscaled
        } else {
            unscaled.conj()
        };
        result.append_complex(positive, 1);
        result.append_complex(positive.conj(), 1);
        Ok(())
    }
}
/// `MathPoly::LaguerreN`: the slice length is the native coefficient count `theSize`.
/// `laguerre_n(&[-1.0, 0.0, 1.0], LAGUERRE_TOLERANCE, LAGUERRE_MAX_ITERATIONS)` finds -1 and 1.
pub fn laguerre_n(coefficients: &[f64], tolerance: f64, max_iterations: u32) -> GeneralPolyResult {
    let size: usize = coefficients.len();
    if !(2..=WORK_SIZE).contains(&size) {
        return PolyResult::failed(Status::InvalidInput);
    }
    laguerre(coefficients, (size - 1) as i32, tolerance, max_iterations)
}
/// Solves a*x^5 + ... + f = 0, falling back to [`quartic`] when `a == 0`;
/// `quintic(1.0, -15.0, 85.0, -225.0, 274.0, -120.0)` finds 1..5.
pub fn quintic(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> GeneralPolyResult {
    if a == 0.0 {
        return quartic(b, c, d, e, f);
    }
    laguerre(
        &[f, e, d, c, b, a],
        5,
        LAGUERRE_TOLERANCE,
        LAGUERRE_MAX_ITERATIONS,
    )
}
/// Solves a*x^6 + ... + g = 0, falling back to [`quintic`] when `a == 0`;
/// `sextic(1.0, -21.0, 175.0, -735.0, 1624.0, -1764.0, 720.0)` finds 1..6.
#[allow(clippy::too_many_arguments)] // Mirrors the native seven-coefficient signature.
pub fn sextic(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64) -> GeneralPolyResult {
    if a == 0.0 {
        return quintic(b, c, d, e, f, g);
    }
    laguerre(
        &[g, f, e, d, c, b, a],
        6,
        LAGUERRE_TOLERANCE,
        LAGUERRE_MAX_ITERATIONS,
    )
}
/// Solves a degree-8 polynomial from ascending coefficients with default settings;
/// `octic(&[40320.0, -109584.0, 118124.0, -67284.0, 22449.0, -4536.0, 546.0, -36.0, 1.0])`.
pub fn octic(coefficients: &[f64; 9]) -> GeneralPolyResult {
    laguerre(coefficients, 8, LAGUERRE_TOLERANCE, LAGUERRE_MAX_ITERATIONS)
}
