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

//! MathLin_Householder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::*;
use crate::math::{Matrix, Vector};
use crate::math_utils::{LinearMultipleResult, LinearResult, Status};
/// Native Householder factors and rank diagnostics; `QrResult::default()`.
#[derive(Default)]
pub struct QrResult {
    /// Completion status; `result.status`.
    pub status: Status,
    /// Full orthogonal factor; `result.q`.
    pub q: Option<Matrix<'static>>,
    /// Upper triangular factor; `result.r`.
    pub r: Option<Matrix<'static>>,
    /// Count of accepted diagonal pivots; `result.rank`.
    pub rank: usize,
    /// Full-rank diagonal ratio; `result.condition_estimate`.
    pub condition_estimate: Option<f64>,
    /// Scale-relative rank threshold; `result.diagonal_threshold`.
    pub diagonal_threshold: f64,
}
impl QrResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
    fn diagnostics(&mut self, r: &Matrix<'_>, relative: f64) {
        let mut maximum: f64 = 0.0;
        for j in 0..r.col_number() {
            maximum = maximum.max(r[(j, j)].abs());
        }
        let mut minimum: f64 = maximum;
        self.diagonal_threshold = relative * maximum;
        for j in 0..r.col_number() {
            let d: f64 = r[(j, j)].abs();
            if d > self.diagonal_threshold {
                self.rank += 1;
                minimum = minimum.min(d);
            }
        }
        if self.rank == r.col_number() as usize {
            let estimate: f64 = maximum / minimum;
            if estimate.is_finite() {
                self.condition_estimate = Some(estimate);
            }
        }
    }
}
impl From<&QrResult> for bool {
    fn from(result: &QrResult) -> bool {
        result.is_done()
    }
}
fn column_norm(r: &Matrix<'_>, j: i32) -> Option<f64> {
    let mut scale: f64 = 0.0;
    let mut sum: f64 = 1.0;
    for i in j..r.row_number() {
        if !accumulate_norm(r[(i, j)], &mut scale, &mut sum) {
            return None;
        }
    }
    norm(scale, sum)
}
fn reflection_vector(r: &Matrix<'_>, v: &mut Vector<'_>, j: i32, norm: f64) -> Option<f64> {
    let alpha: f64 = r[(j, j)];
    let beta: f64 = if alpha >= 0.0 { -norm } else { norm };
    for i in 0..j {
        v[i] = 0.0;
    }
    v[j] = alpha - beta;
    for i in j + 1..v.length() {
        v[i] = r[(i, j)];
    }
    let mut scale: f64 = 0.0;
    for i in j..v.length() {
        scale = scale.max(v[i].abs());
    }
    if scale == 0.0 {
        return None;
    }
    let mut norm_sq: f64 = 0.0;
    for i in j..v.length() {
        v[i] /= scale;
        norm_sq += v[i] * v[i];
    }
    Some(2.0 / norm_sq)
}
fn reflect_r(r: &mut Matrix<'_>, v: &Vector<'_>, j: i32, tau: f64) {
    for k in j..r.col_number() {
        let mut dot: f64 = 0.0;
        for i in j..r.row_number() {
            dot += v[i] * r[(i, k)];
        }
        for i in j..r.row_number() {
            r[(i, k)] -= tau * dot * v[i];
        }
    }
}
fn reflect_q(q: &mut Matrix<'_>, v: &Vector<'_>, j: i32, tau: f64) {
    for k in 0..q.row_number() {
        let mut dot: f64 = 0.0;
        for i in j..q.col_number() {
            dot += q[(k, i)] * v[i];
        }
        for i in j..q.col_number() {
            q[(k, i)] -= tau * dot * v[i];
        }
    }
}
fn reflections(q: &mut Matrix<'_>, r: &mut Matrix<'_>) -> bool {
    let mut v: Vector<'static> = Vector::with_size(r.row_number() as usize);
    for j in 0..r.col_number() {
        let Some(norm) = column_norm(r, j) else {
            return false;
        };
        if norm == 0.0 {
            continue;
        }
        let Some(tau) = reflection_vector(r, &mut v, j, norm) else {
            continue;
        };
        reflect_r(r, &v, j, tau);
        reflect_q(q, &v, j, tau);
    }
    true
}
fn qr_storage(a: &Matrix<'_>) -> (Matrix<'static>, Matrix<'static>) {
    let m: i32 = a.row_number();
    let n: i32 = a.col_number();
    let mut r: Matrix<'static> = Matrix::with_bounds(0, m - 1, 0, n - 1).unwrap();
    r.initialized(a).unwrap();
    let mut q: Matrix<'static> = Matrix::filled(0, m - 1, 0, m - 1, 0.0).unwrap();
    for i in 0..m {
        q[(i, i)] = 1.0;
    }
    (q, r)
}
/// Computes full native Householder factors and rank; `qr(&a,1e-15)`.
pub fn qr(a: &Matrix<'_>, tolerance: f64) -> QrResult {
    let m: i32 = a.row_number();
    let n: i32 = a.col_number();
    let relative: f64 = relative_tolerance(tolerance, m.max(n) as usize);
    if m < n || n == 0 || relative < 0.0 || !is_finite_matrix(a) {
        return QrResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let (mut q, mut r) = qr_storage(a);
    if !reflections(&mut q, &mut r) {
        return QrResult {
            status: Status::NumericalError,
            ..Default::default()
        };
    }
    let mut result: QrResult = QrResult::default();
    result.diagnostics(&r, relative);
    for i in 0..m {
        for j in 0..i.min(n) {
            r[(i, j)] = 0.0;
        }
    }
    result.status = if is_finite_matrix(&q) && is_finite_matrix(&r) {
        Status::Ok
    } else {
        Status::NumericalError
    };
    result.q = Some(q);
    result.r = Some(r);
    result
}
fn projected_rhs(q: &Matrix<'_>, rhs: impl Fn(i32) -> f64) -> Vector<'static> {
    let mut c: Vector<'static> = Vector::with_size(q.row_number() as usize);
    for i in 0..q.col_number() {
        let mut sum: f64 = 0.0;
        for k in 0..q.row_number() {
            sum += q[(k, i)] * rhs(k);
        }
        c[i] = sum;
    }
    c
}
fn back_substitute(r: &Matrix<'_>, c: &Vector<'_>, x: &mut Vector<'_>, threshold: f64) -> bool {
    let n: i32 = r.col_number();
    for i in (0..n).rev() {
        let d: f64 = r[(i, i)];
        if d.abs() <= threshold {
            return false;
        }
        let mut sum: f64 = c[i];
        for k in i + 1..n {
            sum -= r[(i, k)] * x[k];
        }
        x[i] = sum / d;
    }
    true
}
/// Solves with the native QR rank test and partial solution; `solve_qr(&a,&b,1e-15)`.
pub fn solve_qr(a: &Matrix<'_>, b: &Vector<'_>, tolerance: f64) -> LinearResult {
    if b.length() != a.row_number() || !is_finite_vector(b) {
        return LinearResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let factors: QrResult = qr(a, tolerance);
    if !factors.is_done() {
        return LinearResult {
            status: factors.status,
            ..Default::default()
        };
    }
    if factors.rank != a.col_number() as usize {
        return LinearResult {
            status: Status::Singular,
            ..Default::default()
        };
    }
    let c: Vector<'static> =
        projected_rhs(factors.q.as_ref().unwrap(), |k| *b.at(k as usize).unwrap());
    let mut x: Vector<'static> = Vector::with_size(a.col_number() as usize);
    let ok: bool = back_substitute(
        factors.r.as_ref().unwrap(),
        &c,
        &mut x,
        factors.diagonal_threshold,
    );
    let status: Status = if !ok {
        Status::Singular
    } else if is_finite_vector(&x) {
        Status::Ok
    } else {
        Status::NumericalError
    };
    LinearResult {
        status,
        solution: Some(x),
        ..Default::default()
    }
}
/// Solves each right-hand side with native projection order; `solve_qr_multiple(&a,&b,1e-15)`.
pub fn solve_qr_multiple(a: &Matrix<'_>, b: &Matrix<'_>, tolerance: f64) -> LinearMultipleResult {
    if b.row_number() != a.row_number() || !is_finite_matrix(b) {
        return LinearMultipleResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let factors: QrResult = qr(a, tolerance);
    if !factors.is_done() {
        return LinearMultipleResult {
            status: factors.status,
            ..Default::default()
        };
    }
    if factors.rank != a.col_number() as usize {
        return LinearMultipleResult {
            status: Status::Singular,
            ..Default::default()
        };
    }
    let mut out: Matrix<'static> =
        Matrix::filled(0, a.col_number() - 1, 0, b.col_number() - 1, 0.0).unwrap();
    for j in 0..b.col_number() {
        let c: Vector<'static> = projected_rhs(factors.q.as_ref().unwrap(), |k| {
            *b.at(k as usize, j as usize).unwrap()
        });
        let mut x: Vector<'static> = Vector::with_size(a.col_number() as usize);
        if !back_substitute(
            factors.r.as_ref().unwrap(),
            &c,
            &mut x,
            factors.diagonal_threshold,
        ) {
            return LinearMultipleResult {
                status: Status::Singular,
                ..Default::default()
            };
        }
        for i in 0..a.col_number() {
            out[(i, j)] = x[i];
        }
    }
    let status: Status = if is_finite_matrix(&out) {
        Status::Ok
    } else {
        Status::NumericalError
    };
    LinearMultipleResult {
        status,
        solutions: Some(out),
        ..Default::default()
    }
}
