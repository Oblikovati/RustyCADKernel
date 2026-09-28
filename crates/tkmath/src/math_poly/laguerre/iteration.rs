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

//! `Utils::EvaluateComplex`, `LaguerreIteration` and `FindRoot` of `MathPoly_Laguerre.hxx`.
use super::COMPUTATIONAL;
use crate::math_poly::Complex;
use crate::math_utils::{Status, TWO_PI};
/// Number of deterministic complex starts used to recover a Laguerre root.
const LAGUERRE_NB_STARTS: usize = 16;
/// Native `Utils::IterationResult`.
#[derive(Clone, Copy, Default)]
pub(super) struct IterationResult {
    pub(super) status: Status,
    pub(super) root: Complex,
    pub(super) root_error: Option<f64>,
}
impl IterationResult {
    fn failed(status: Status) -> Self {
        Self {
            status,
            ..Self::default()
        }
    }
    fn accepted(root: Complex, root_error: f64) -> Self {
        Self {
            status: Status::Ok,
            root,
            root_error: Some(root_error),
        }
    }
}
/// Value, first and second derivative, and absolute magnitude bound at a complex point.
struct ComplexEvaluation {
    value: Complex,
    derivative: Complex,
    second: Complex,
    magnitude: f64,
}
impl ComplexEvaluation {
    fn finite(&self) -> bool {
        self.value.is_finite()
            && self.derivative.is_finite()
            && self.second.is_finite()
            && self.magnitude.is_finite()
    }
}
fn evaluate_complex(c: &[f64], x: Complex) -> ComplexEvaluation {
    let degree: usize = c.len() - 1;
    let mut e: ComplexEvaluation = ComplexEvaluation {
        value: Complex::new(c[degree], 0.0),
        derivative: Complex::new(0.0, 0.0),
        second: Complex::new(0.0, 0.0),
        magnitude: c[degree].abs(),
    };
    let abs_x: f64 = x.abs();
    for &coefficient in c[..degree].iter().rev() {
        e.second = e.second * x + 2.0 * e.derivative;
        e.derivative = e.derivative * x + e.value;
        e.value = e.value * x + coefficient;
        e.magnitude = e.magnitude * abs_x + coefficient.abs();
    }
    e
}
/// `Utils::HasSmallResidual(const double*, int, complex, double)`; real roots arrive as `(x, +0)`.
pub(super) fn has_small_residual(c: &[f64], root: Complex, tolerance: f64) -> bool {
    let e: ComplexEvaluation = evaluate_complex(c, root);
    e.value.is_finite() && e.value.abs() <= tolerance * 1.0_f64.max(e.magnitude)
}
/// One Laguerre correction; `None` where native code reports a non-finite step.
fn laguerre_step(
    e: &ComplexEvaluation,
    degree: usize,
    root: Complex,
) -> Option<(Complex, Complex)> {
    let g: Complex = e.derivative / e.value;
    let h: Complex = g * g - e.second / e.value;
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
        Some(b) => candidate.root_error.unwrap() < b.root_error.unwrap(),
    };
    if better {
        *best = Some(candidate);
    }
}
/// Records `root` when its residual is small; returns it when the step has also converged.
fn record_candidate(
    best: &mut Option<IterationResult>,
    e: &ComplexEvaluation,
    root: Complex,
    step: Complex,
    tolerance: f64,
) -> Option<IterationResult> {
    let root_scale: f64 = 1.0_f64.max(root.abs());
    if e.value.abs() > tolerance * 1.0_f64.max(e.magnitude) {
        return None;
    }
    let candidate: IterationResult =
        IterationResult::accepted(root, step.abs() + COMPUTATIONAL * root_scale);
    keep_best(best, candidate);
    (step.abs() <= tolerance * root_scale).then_some(candidate)
}
/// `Utils::LaguerreIteration`; the native iteration counter is not observable here.
fn laguerre_iteration(
    c: &[f64],
    start: Complex,
    tolerance: f64,
    max_iterations: u32,
) -> IterationResult {
    let degree: usize = c.len() - 1;
    let mut best: Option<IterationResult> = None;
    let mut root: Complex = start;
    for _ in 0..max_iterations {
        let e: ComplexEvaluation = evaluate_complex(c, root);
        if !e.finite() {
            return best.unwrap_or(IterationResult::failed(Status::NumericalError));
        }
        if e.value == Complex::new(0.0, 0.0) {
            return IterationResult::accepted(root, COMPUTATIONAL * 1.0_f64.max(root.abs()));
        }
        let Some((step, next)) = laguerre_step(&e, degree, root) else {
            return best.unwrap_or(IterationResult::failed(Status::NumericalError));
        };
        if let Some(converged) = record_candidate(&mut best, &e, root, step, tolerance) {
            return converged;
        }
        root = next;
    }
    best.unwrap_or(IterationResult {
        status: Status::MaxIterations,
        root,
        root_error: None,
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
/// `Utils::FindRoot`: first start whose accepted root also passes the residual check.
pub(super) fn find_root(c: &[f64], tolerance: f64, max_iterations: u32) -> IterationResult {
    for index in 0..LAGUERRE_NB_STARTS {
        let result: IterationResult =
            laguerre_iteration(c, start_point(index), tolerance, max_iterations);
        if result.status == Status::Ok && has_small_residual(c, result.root, tolerance) {
            return result;
        }
    }
    IterationResult::failed(Status::MaxIterations)
}
