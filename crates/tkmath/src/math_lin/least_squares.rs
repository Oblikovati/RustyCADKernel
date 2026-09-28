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

//! MathLin_LeastSquares.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{utils::*, *};
use crate::math::{Matrix, Vector};
use crate::math_utils::{LinearResult, Status};
mod regularization;
pub use regularization::{optimal_regularization, regularized_least_squares};
/// Native solver selection; `LeastSquaresMethod::Qr`.
#[derive(Clone, Copy, Default)]
pub enum LeastSquaresMethod {
    /// Forms normal equations; `LeastSquaresMethod::NormalEquations`.
    NormalEquations,
    /// Uses Householder factors; `LeastSquaresMethod::Qr`.
    #[default]
    Qr,
    /// Handles deficient rank; `LeastSquaresMethod::Svd`.
    Svd,
    /// Preserves the native invalid-enum fallthrough; `LeastSquaresMethod::Invalid(3)`.
    Invalid(i32),
}
/// Native least-squares solution and residual diagnostics; `LeastSquaresResult::default()`.
#[derive(Default)]
pub struct LeastSquaresResult {
    /// Native status; `result.status`.
    pub status: Status,
    /// Accepted solution; `result.solution`.
    pub solution: Option<Vector<'static>>,
    /// Residual norm; `result.residual`.
    pub residual: Option<f64>,
    /// Finite residual square; `result.residual_sq`.
    pub residual_sq: Option<f64>,
    /// Solver rank; `result.rank`.
    pub rank: usize,
    /// Solver condition diagnostic; `result.condition_number`.
    pub condition_number: Option<f64>,
}
impl LeastSquaresResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
    fn failed(status: Status) -> Self {
        Self {
            status,
            ..Default::default()
        }
    }
    fn finish(&mut self, a: &Matrix<'_>, b: &Vector<'_>) {
        let x: &Vector<'_> = self.solution.as_ref().unwrap();
        let residual: Option<f64> = residual_norm(a, b, x);
        if residual.is_none() || !is_finite_vector(x) {
            self.status = Status::NumericalError;
            return;
        }
        let value: f64 = residual.unwrap();
        self.residual_sq = (value <= f64::MAX.sqrt()).then_some(value * value);
        self.residual = residual;
        self.status = Status::Ok;
    }
}
impl From<&LeastSquaresResult> for bool {
    fn from(result: &LeastSquaresResult) -> bool {
        result.is_done()
    }
}
fn valid_system(a: &Matrix<'_>, b: &Vector<'_>) -> bool {
    b.length() == a.row_number() && is_finite_matrix(a) && is_finite_vector(b)
}
fn row_residual(a: &Matrix<'_>, b: &Vector<'_>, x: &Vector<'_>, row: usize) -> f64 {
    let mut sum: f64 = 0.0;
    for j in 0..a.col_number() {
        sum += a.at(row, j as usize).unwrap() * x[j];
    }
    sum - b.at(row).unwrap()
}
fn residual_norm(a: &Matrix<'_>, b: &Vector<'_>, x: &Vector<'_>) -> Option<f64> {
    let mut scale: f64 = 0.0;
    let mut sum: f64 = 1.0;
    let mut valid: bool = true;
    for i in 0..a.row_number() {
        valid = accumulate_norm(row_residual(a, b, x, i as usize), &mut scale, &mut sum) && valid;
    }
    if valid { norm(scale, sum) } else { None }
}
fn normal_matrix(a: &Matrix<'_>) -> Matrix<'static> {
    let n: i32 = a.col_number();
    let mut ata: Matrix<'static> = Matrix::filled(0, n - 1, 0, n - 1, 0.0).unwrap();
    for i in 0..n {
        for j in 0..n {
            ata[(i, j)] = normal_entry(a, i as usize, j as usize);
        }
    }
    ata
}
fn normal_entry(a: &Matrix<'_>, i: usize, j: usize) -> f64 {
    let mut sum: f64 = 0.0;
    for k in 0..a.row_number() as usize {
        sum += a.at(k, i).unwrap() * a.at(k, j).unwrap();
    }
    sum
}
fn normal_rhs(a: &Matrix<'_>, b: &Vector<'_>) -> Vector<'static> {
    let mut rhs: Vector<'static> = Vector::with_size(a.col_number() as usize);
    for i in 0..a.col_number() {
        let mut sum: f64 = 0.0;
        for k in 0..a.row_number() as usize {
            sum += a.at(k, i as usize).unwrap() * b.at(k).unwrap();
        }
        rhs[i] = sum;
    }
    rhs
}
fn qr_solution(
    a: &Matrix<'_>,
    b: &Vector<'_>,
    tol: f64,
    out: &mut LeastSquaresResult,
) -> LinearResult {
    let factors: QrResult = qr(a, tol);
    if !factors.is_done() || factors.rank != a.col_number() as usize {
        return LinearResult {
            status: if factors.is_done() {
                Status::Singular
            } else {
                factors.status
            },
            ..Default::default()
        };
    }
    out.rank = factors.rank;
    out.condition_number = factors.condition_estimate;
    solve_qr(a, b, tol)
}
fn svd_solution(
    a: &Matrix<'_>,
    b: &Vector<'_>,
    tol: f64,
    out: &mut LeastSquaresResult,
) -> LinearResult {
    let result: LinearResult = solve_svd(a, b, tol);
    if result.is_done() {
        let factors: SvdResult = svd(a, tol);
        if factors.is_done() {
            out.rank = factors.rank;
            out.condition_number = factors.condition_number;
        }
    }
    result
}
fn selected_solution(
    a: &Matrix<'_>,
    b: &Vector<'_>,
    method: LeastSquaresMethod,
    tol: f64,
    out: &mut LeastSquaresResult,
) -> LinearResult {
    match method {
        LeastSquaresMethod::Qr => qr_solution(a, b, tol, out),
        LeastSquaresMethod::Svd => svd_solution(a, b, tol, out),
        LeastSquaresMethod::Invalid(_) => LinearResult::default(),
        LeastSquaresMethod::NormalEquations => {
            let result: LinearResult = solve(&normal_matrix(a), &normal_rhs(a, b), tol);
            out.rank = if result.is_done() {
                a.col_number() as usize
            } else {
                0
            };
            result
        }
    }
}
/// Solves overdetermined systems with native diagnostics; `least_squares(&a,&b,LeastSquaresMethod::Qr,1e-15)`.
pub fn least_squares(
    a: &Matrix<'_>,
    b: &Vector<'_>,
    method: LeastSquaresMethod,
    tolerance: f64,
) -> LeastSquaresResult {
    if a.row_number() < a.col_number() || !valid_system(a, b) {
        return LeastSquaresResult::failed(Status::InvalidInput);
    }
    let mut out: LeastSquaresResult = LeastSquaresResult::default();
    let solution: LinearResult = selected_solution(a, b, method, tolerance, &mut out);
    if !solution.is_done() {
        out.status = solution.status;
        return out;
    }
    out.solution = solution.solution;
    out.finish(a, b);
    out
}
fn weighted_system(
    a: &Matrix<'_>,
    b: &Vector<'_>,
    w: &Vector<'_>,
) -> (Matrix<'static>, Vector<'static>) {
    let mut wa: Matrix<'static> =
        Matrix::filled(0, a.row_number() - 1, 0, a.col_number() - 1, 0.0).unwrap();
    let mut wb: Vector<'static> = Vector::with_size(a.row_number() as usize);
    for i in 0..a.row_number() {
        let weight: f64 = w.at(i as usize).unwrap().sqrt();
        for j in 0..a.col_number() {
            wa[(i, j)] = weight * a.at(i as usize, j as usize).unwrap();
        }
        wb[i] = weight * b.at(i as usize).unwrap();
    }
    (wa, wb)
}
/// Applies positive weights before solving; `weighted_least_squares(&a,&b,&w,LeastSquaresMethod::Qr,1e-15)`.
pub fn weighted_least_squares(
    a: &Matrix<'_>,
    b: &Vector<'_>,
    w: &Vector<'_>,
    method: LeastSquaresMethod,
    tolerance: f64,
) -> LeastSquaresResult {
    if b.length() != a.row_number() || w.length() != a.row_number() || !is_finite_vector(w) {
        return LeastSquaresResult::failed(Status::InvalidInput);
    }
    if (0..w.length()).any(|i| *w.at(i as usize).unwrap() <= 0.0) {
        return LeastSquaresResult::failed(Status::InvalidInput);
    }
    let (wa, wb) = weighted_system(a, b, w);
    least_squares(&wa, &wb, method, tolerance)
}
