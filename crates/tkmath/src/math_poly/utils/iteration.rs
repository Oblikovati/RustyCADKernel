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

//! `EvaluateComplex`, the complex `HasSmallResidual`, `LaguerreIteration` and `FindRoot`.
use super::preconditions::slice_degree;
use crate::math_poly::Complex;
use crate::math_utils::{Status, TWO_PI};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
/// `THE_LAGUERRE_NB_STARTS`: deterministic complex starts tried by `find_root`.
pub const LAGUERRE_NB_STARTS: usize = 16;
/// `Utils::IterationResult`; `IterationResult::default()` is `NotConverged` at the origin.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IterationResult {
    /// Completion status; `result.status`.
    pub status: Status,
    /// Accepted (or last) iterate; `result.root`.
    pub root: Complex,
    /// Error estimate of an accepted root; `result.root_error`.
    pub root_error: Option<f64>,
    /// Iteration index at which the result was produced; `result.nb_iterations`.
    pub nb_iterations: u32,
}
impl IterationResult {
    fn failed(status: Status) -> Self {
        Self {
            status,
            ..Self::default()
        }
    }
    fn accepted(root: Complex, root_error: f64, nb_iterations: u32) -> Self {
        Self {
            status: Status::Ok,
            root,
            root_error: Some(root_error),
            nb_iterations,
        }
    }
}
/// The outputs of `EvaluateComplex`; `evaluation.value`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ComplexEvaluation {
    /// Polynomial value.
    pub value: Complex,
    /// First derivative.
    pub derivative: Complex,
    /// Second derivative.
    pub second_derivative: Complex,
    /// Absolute bound `sum |c_i| |x|^i`.
    pub magnitude: f64,
}
impl ComplexEvaluation {
    fn is_finite(&self) -> bool {
        self.value.is_finite()
            && self.derivative.is_finite()
            && self.second_derivative.is_finite()
            && self.magnitude.is_finite()
    }
}
/// `EvaluateComplex`: Horner value and first two derivatives with GCC complex arithmetic;
/// Standard_OutOfRange for a negative degree or fewer than `degree + 1` coefficients;
/// `utils::evaluate_complex(&[1.0, 0.0, 1.0], 2, Complex::new(0.0, 1.0))?`.
pub fn evaluate_complex(
    coefficients: &[f64],
    degree: i32,
    x: Complex,
) -> Result<ComplexEvaluation, Failure> {
    slice_degree(
        "MathPoly::Utils::EvaluateComplex",
        coefficients.len(),
        degree,
        0,
    )?;
    Ok(complex_horner(coefficients, degree, x))
}
/// [`evaluate_complex`] on a degree the caller has validated.
fn complex_horner(coefficients: &[f64], degree: i32, x: Complex) -> ComplexEvaluation {
    let d: usize = degree as usize;
    let mut e: ComplexEvaluation = ComplexEvaluation {
        value: Complex::new(coefficients[d], 0.0),
        derivative: Complex::new(0.0, 0.0),
        second_derivative: Complex::new(0.0, 0.0),
        magnitude: coefficients[d].abs(),
    };
    let abs_x: f64 = x.abs();
    for &coefficient in coefficients[..d].iter().rev() {
        e.second_derivative = e.second_derivative * x + 2.0 * e.derivative;
        e.derivative = e.derivative * x + e.value;
        e.value = e.value * x + coefficient;
        e.magnitude = e.magnitude * abs_x + coefficient.abs();
    }
    e
}
/// `HasSmallResidual(const double*, int, complex, double)`; real roots arrive as `(x, +0)`.
/// Standard_OutOfRange as for [`evaluate_complex`];
/// `utils::has_small_complex_residual(&[-1.0, 1.0], 1, Complex::new(1.0, 0.0), 1e-12)?`.
pub fn has_small_complex_residual(
    coefficients: &[f64],
    degree: i32,
    root: Complex,
    tolerance: f64,
) -> Result<bool, Failure> {
    slice_degree(
        "MathPoly::Utils::HasSmallResidual",
        coefficients.len(),
        degree,
        0,
    )?;
    Ok(small_complex_residual(
        coefficients,
        degree,
        root,
        tolerance,
    ))
}
/// One Laguerre correction; `None` where native code reports a non-finite step.
fn laguerre_step(e: &ComplexEvaluation, degree: i32, root: Complex) -> Option<(Complex, Complex)> {
    let g: Complex = e.derivative / e.value;
    let h: Complex = g * g - e.second_derivative / e.value;
    let n: f64 = degree as f64;
    let square_root: Complex = ((n - 1.0) * (n * h - g * g)).sqrt();
    let plus: Complex = g + square_root;
    let minus: Complex = g - square_root;
    let denominator: Complex = if plus.abs() >= minus.abs() {
        plus
    } else {
        minus
    };
    if !denominator.is_finite() || denominator.abs() == 0.0 {
        return None;
    }
    // `double / complex` widens the dividend to `(n, +0)` and calls `__divdc3`.
    let step: Complex = Complex::new(n, 0.0) / denominator;
    let next: Complex = root - step;
    (step.is_finite() && next.is_finite()).then_some((step, next))
}
/// Keeps the candidate with the smallest root error; native ties keep the earlier one.
fn keep_best(best: &mut Option<IterationResult>, candidate: IterationResult) {
    let better: bool = match best {
        None => true,
        Some(b) => candidate.root_error < b.root_error,
    };
    if better {
        *best = Some(candidate);
    }
}
/// Records `root` when its residual is small; returns it when the step has also converged.
fn record_candidate(
    best: &mut Option<IterationResult>,
    e: &ComplexEvaluation,
    (root, step): (Complex, Complex),
    tolerance: f64,
    iteration: u32,
) -> Option<IterationResult> {
    let root_scale: f64 = 1.0_f64.max(root.abs());
    // Native code records only when `|value| <= bound`; a NaN tolerance records nothing.
    let small: bool = e.value.abs() <= tolerance * 1.0_f64.max(e.magnitude);
    if !small {
        return None;
    }
    let error: f64 = step.abs() + precision::computational() * root_scale;
    let candidate: IterationResult = IterationResult::accepted(root, error, iteration);
    keep_best(best, candidate);
    (step.abs() <= tolerance * root_scale).then_some(candidate)
}
/// `LaguerreIteration`: Laguerre's method from `start`, returning the first converged
/// candidate, else the best small-residual candidate, else `MaxIterations` at the last
/// iterate. Standard_OutOfRange as for [`evaluate_complex`] once an iteration runs;
/// `utils::laguerre_iteration(&c, 3, Complex::new(0.0, 0.0), 1e-12, 100)?`.
pub fn laguerre_iteration(
    coefficients: &[f64],
    degree: i32,
    start: Complex,
    tolerance: f64,
    max_iterations: u32,
) -> Result<IterationResult, Failure> {
    if max_iterations > 0 {
        slice_degree(
            "MathPoly::Utils::LaguerreIteration",
            coefficients.len(),
            degree,
            0,
        )?;
    }
    Ok(iterated(
        coefficients,
        degree,
        start,
        tolerance,
        max_iterations,
    ))
}
/// [`laguerre_iteration`] on a degree the caller has validated.
fn iterated(
    coefficients: &[f64],
    degree: i32,
    start: Complex,
    tolerance: f64,
    max_iterations: u32,
) -> IterationResult {
    let mut best: Option<IterationResult> = None;
    let mut root: Complex = start;
    for iteration in 0..max_iterations {
        let e: ComplexEvaluation = complex_horner(coefficients, degree, root);
        if !e.is_finite() {
            return best.unwrap_or(IterationResult::failed(Status::NumericalError));
        }
        if e.value == Complex::new(0.0, 0.0) {
            let error: f64 = precision::computational() * 1.0_f64.max(root.abs());
            return IterationResult::accepted(root, error, iteration);
        }
        let Some((step, next)) = laguerre_step(&e, degree, root) else {
            return best.unwrap_or(IterationResult::failed(Status::NumericalError));
        };
        if let Some(converged) = record_candidate(&mut best, &e, (root, step), tolerance, iteration)
        {
            return converged;
        }
        root = next;
    }
    best.unwrap_or(IterationResult {
        status: Status::MaxIterations,
        root,
        root_error: None,
        nb_iterations: max_iterations,
    })
}
/// Deterministic start `k`: radius 0 for k = 0, else 0.25 + 1.75*(k%4)/3 at angle 2*pi*k/16.
fn start_point(index: usize) -> Complex {
    let radius: f64 = if index == 0 {
        0.0
    } else {
        0.25 + 1.75 * ((index % 4) as f64) / 3.0
    };
    let angle: f64 = TWO_PI * (index as f64) / (LAGUERRE_NB_STARTS as f64);
    Complex::polar(radius, angle)
}
/// `FindRoot`: the first start whose accepted root also passes the residual check, else
/// `MaxIterations`. Standard_OutOfRange as for [`evaluate_complex`] once an iteration runs;
/// `utils::find_root(&[-6.0, 11.0, -6.0, 1.0], 3, 1e-12, 100)?`.
pub fn find_root(
    coefficients: &[f64],
    degree: i32,
    tolerance: f64,
    max_iterations: u32,
) -> Result<IterationResult, Failure> {
    if max_iterations > 0 {
        slice_degree("MathPoly::Utils::FindRoot", coefficients.len(), degree, 0)?;
    }
    for index in 0..LAGUERRE_NB_STARTS {
        let start: Complex = start_point(index);
        let result: IterationResult =
            iterated(coefficients, degree, start, tolerance, max_iterations);
        if result.status == Status::Ok
            && small_complex_residual(coefficients, degree, result.root, tolerance)
        {
            return Ok(result);
        }
    }
    Ok(IterationResult::failed(Status::MaxIterations))
}
/// [`has_small_complex_residual`] on a degree the caller has validated.
pub(in crate::math_poly) fn small_complex_residual(
    coefficients: &[f64],
    degree: i32,
    root: Complex,
    tolerance: f64,
) -> bool {
    let e: ComplexEvaluation = complex_horner(coefficients, degree, root);
    e.value.is_finite() && e.value.abs() <= tolerance * 1.0_f64.max(e.magnitude)
}
