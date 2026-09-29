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

//! `MathPoly_Laguerre.hxx`: checked Laguerre iteration with deflation for degree <= 20.
use super::extended::{LongDouble, scale_double};
use super::utils::{
    self, IterationResult, LongCoefficients, consolidate_complex_roots, consolidate_real_roots,
    deflate_complex, deflate_real, find_root, has_multiplicity, has_small_complex_residual,
};
use super::{Complex, GeneralPolyResult, MAX_POLY_DEGREE, PolyResult, quartic};
use crate::math_utils::Status;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
mod refine;
/// The Utils helpers check the preconditions `Laguerre` has already validated (degree
/// within `1..=MAX_POLY_DEGREE`, full coefficient arrays, at most `degree` roots), so
/// their failure arm is unreachable here; it would report the input as invalid.
fn validated<T>(checked: Result<T, Failure>) -> Result<T, Status> {
    checked.map_err(|_| Status::InvalidInput)
}
const WORK_SIZE: usize = MAX_POLY_DEGREE + 1;
/// Working coefficient storage of the scaled, progressively deflated polynomial.
type Work = [f64; WORK_SIZE];
/// Default relative residual tolerance of `MathPoly::Laguerre`; `LAGUERRE_TOLERANCE`.
pub const LAGUERRE_TOLERANCE: f64 = 1.0e-12;
/// Default per-start iteration limit of `MathPoly::Laguerre`; `LAGUERRE_MAX_ITERATIONS`.
pub const LAGUERRE_MAX_ITERATIONS: u32 = 100;
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
    if !utils::validate(Some(coefficients), degree, tolerance)
        || degree < 1
        || coefficients[degree as usize] == 0.0
    {
        return PolyResult::failed(Status::InvalidInput);
    }
    let (long_scaled, exponent) =
        utils::scaled(coefficients, degree).unwrap_or((utils::long_coefficients(), 0));
    let settings: Deflation = Deflation {
        tolerance,
        max_iterations,
        exponent,
    };
    settings.solve(&long_scaled, degree)
}
/// Narrows the long-double scaled coefficients into the double working polynomial.
fn narrowed(long_scaled: &LongCoefficients) -> Work {
    long_scaled.map(LongDouble::narrow)
}
/// Solver settings and the variable exponent shared by the deflation branches.
struct Deflation {
    tolerance: f64,
    max_iterations: u32,
    exponent: i32,
}
impl Deflation {
    /// Solves a validated scaled polynomial whose leading coefficient is nonzero.
    fn solve(&self, long_scaled: &LongCoefficients, degree: i32) -> PolyResult {
        let original: Work = narrowed(long_scaled);
        let mut result: PolyResult = PolyResult::failed(Status::NotConverged);
        if let Err(status) = self
            .extract_roots(&mut result, original, degree)
            .and_then(|()| self.consolidate(&mut result, &original, degree))
        {
            result.status = status;
            return result;
        }
        refine::polish_simple_roots(&mut result, long_scaled, degree, self.exponent);
        let scaled = refine::Scaled {
            long: long_scaled,
            original: &original,
            degree,
            exponent: self.exponent,
        };
        result.status = refine::verified_status(&result, &scaled, self.tolerance);
        result
    }
    /// Merges the clusters of real, then complex, roots.
    fn consolidate(
        &self,
        result: &mut PolyResult,
        original: &Work,
        degree: i32,
    ) -> Result<(), Status> {
        validated(consolidate_real_roots(
            result,
            self.tolerance,
            original,
            degree,
            self.exponent,
        ))?;
        validated(consolidate_complex_roots(
            result,
            self.tolerance,
            original,
            degree,
            self.exponent,
        ))
    }
    /// Deflation loop; returns the failing status while keeping roots already accepted.
    fn extract_roots(
        &self,
        result: &mut PolyResult,
        mut work: Work,
        mut degree: i32,
    ) -> Result<(), Status> {
        while degree > 0 {
            let found: IterationResult = validated(find_root(
                &work,
                degree,
                self.tolerance,
                self.max_iterations,
            ))?;
            if found.status != Status::Ok || !utils::is_finite(found.root) {
                return Err(found.status);
            }
            let error: f64 = found.root_error.unwrap_or(f64::NAN);
            let uncertainty: f64 =
                error + precision::computational() * 1.0_f64.max(found.root.abs());
            if found.root.imag().abs() <= uncertainty {
                self.real_root(result, &mut work, &mut degree, (found.root.real(), error))?;
            } else {
                self.complex_root(result, &mut work, &mut degree, found.root)?;
            }
        }
        Ok(())
    }
    /// Native `ToDouble(scalbn((long double)root, exponent))`.
    fn unscaled(&self, root: f64) -> Result<f64, Status> {
        utils::to_double(LongDouble::new(root).scale(self.exponent)).ok_or(Status::NumericalError)
    }
    /// Refines through a double-root check when the iteration error is large.
    fn real_root(
        &self,
        result: &mut PolyResult,
        work: &mut Work,
        degree: &mut i32,
        (mut root, error): (f64, f64),
    ) -> Result<(), Status> {
        let mut repeated: f64 = root;
        if error > self.tolerance * 1.0_f64.max(root.abs())
            && *degree > 1
            && validated(has_multiplicity(work, *degree, &mut repeated, 2))?
        {
            root = repeated;
        }
        let real: Complex = Complex::new(root, 0.0);
        if !validated(has_small_complex_residual(
            work,
            *degree,
            real,
            self.tolerance,
        ))? || !validated(deflate_real(work, degree, root, self.tolerance))?
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
        degree: &mut i32,
        root: Complex,
    ) -> Result<(), Status> {
        if *degree < 2
            || !validated(has_small_complex_residual(
                work,
                *degree,
                root,
                self.tolerance,
            ))?
        {
            return Err(Status::NumericalError);
        }
        let mut real_root: f64 = root.real();
        let mut repeated: f64 = real_root;
        if validated(has_multiplicity(work, *degree, &mut repeated, 2))? {
            real_root = repeated;
        }
        let (mut real_work, mut real_degree): (Work, i32) = (*work, *degree);
        let real: Complex = Complex::new(real_root, 0.0);
        if validated(has_small_complex_residual(
            work,
            *degree,
            real,
            self.tolerance,
        ))? && validated(deflate_real(
            &mut real_work,
            &mut real_degree,
            real_root,
            self.tolerance,
        ))? {
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
        degree: &mut i32,
        root: Complex,
    ) -> Result<(), Status> {
        if !validated(deflate_complex(work, degree, root, self.tolerance))? {
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
