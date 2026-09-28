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

//! MathLin_SVD.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::*;
use crate::math::{Matrix, Svd, Vector};
use crate::math_utils::{InverseResult, LinearResult, Status};
/// Scaled native singular-value factors; `SvdResult::default()`.
#[derive(Default)]
pub struct SvdResult {
    /// Native completion status; `result.status`.
    pub status: Status,
    /// Truncated left vectors; `result.u`.
    pub u: Option<Matrix<'static>>,
    /// Descending singular values; `result.singular_values`.
    pub singular_values: Option<Vector<'static>>,
    /// Right vectors; `result.v`.
    pub v: Option<Matrix<'static>>,
    /// Accepted singular values; `result.rank`.
    pub rank: usize,
    /// Full-rank condition number; `result.condition_number`.
    pub condition_number: Option<f64>,
}
impl SvdResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
    fn threshold(&self, tolerance: f64) -> f64 {
        let u: &Matrix<'_> = self.u.as_ref().unwrap();
        let w: &Vector<'_> = self.singular_values.as_ref().unwrap();
        let maximum: f64 = (0..w.length()).fold(0.0_f64, |s, i| s.max(w[i]));
        relative_tolerance(tolerance, u.row_number().max(u.col_number()) as usize) * maximum
    }
}
impl From<&SvdResult> for bool {
    fn from(result: &SvdResult) -> bool {
        result.is_done()
    }
}
fn scaled_input(a: &Matrix<'_>) -> (Matrix<'static>, f64) {
    let m: i32 = a.row_number();
    let n: i32 = a.col_number();
    let mut scale: f64 = 0.0;
    for i in 0..m {
        for j in 0..n {
            scale = scale.max(a.at(i as usize, j as usize).unwrap().abs());
        }
    }
    let mut normalized: Matrix<'static> = Matrix::filled(0, m.max(n) - 1, 0, n - 1, 0.0).unwrap();
    for i in 0..m {
        for j in 0..n {
            normalized[(i, j)] = if scale == 0.0 {
                0.0
            } else {
                a.at(i as usize, j as usize).unwrap() / scale
            };
        }
    }
    (normalized, scale)
}
fn swap_columns(matrix: &mut Matrix<'_>, first: i32, second: i32) {
    for row in 1..=matrix.row_number() {
        let saved: f64 = matrix[(row, first)];
        matrix[(row, first)] = matrix[(row, second)];
        matrix[(row, second)] = saved;
    }
}
fn sort_factors(u: &mut Matrix<'_>, w: &mut Vector<'_>, v: &mut Matrix<'_>) {
    for i in 1..w.length() {
        let mut largest: i32 = i;
        for j in i + 1..=w.length() {
            if w[j] > w[largest] {
                largest = j;
            }
        }
        if largest == i {
            continue;
        }
        let saved: f64 = w[i];
        w[i] = w[largest];
        w[largest] = saved;
        swap_columns(u, i, largest);
        swap_columns(v, i, largest);
    }
}
fn zero_based(matrix: &Matrix<'_>, rows: i32) -> Matrix<'static> {
    let mut out: Matrix<'static> =
        Matrix::filled(0, rows - 1, 0, matrix.col_number() - 1, 0.0).unwrap();
    for i in 0..rows {
        for j in 0..matrix.col_number() {
            out[(i, j)] = matrix[(i + 1, j + 1)];
        }
    }
    out
}
fn singular_values_zero_based(w: &Vector<'_>) -> Vector<'static> {
    let mut values: Vector<'static> = Vector::with_size(w.length() as usize);
    for i in 0..w.length() {
        values[i] = w[i + 1];
    }
    values
}
fn rank_condition(values: &Vector<'_>, rows: i32, relative: f64) -> (usize, Option<f64>) {
    let maximum: f64 = (0..values.length()).fold(0.0_f64, |s, i| s.max(values[i]));
    let rank: usize = (0..values.length())
        .filter(|&i| values[i] > relative * maximum)
        .count();
    let full: usize = rows.min(values.length()) as usize;
    if rank != full {
        return (rank, None);
    }
    let condition: f64 = values[0] / values[full as i32 - 1];
    (rank, condition.is_finite().then_some(condition))
}
fn finite_status(finite: bool) -> Status {
    if finite {
        Status::Ok
    } else {
        Status::NumericalError
    }
}
fn collect_factors(
    u: Matrix<'_>,
    w: Vector<'_>,
    v: Matrix<'_>,
    rows: i32,
    relative: f64,
) -> SvdResult {
    let u: Matrix<'static> = zero_based(&u, rows);
    let v: Matrix<'static> = zero_based(&v, v.row_number());
    let values: Vector<'static> = singular_values_zero_based(&w);
    let (rank, condition_number) = rank_condition(&values, rows, relative);
    let finite: bool = is_finite_matrix(&u) && is_finite_matrix(&v) && is_finite_vector(&values);
    SvdResult {
        status: finite_status(finite),
        u: Some(u),
        v: Some(v),
        singular_values: Some(values),
        rank,
        condition_number,
    }
}
/// Preserves scaling, sorting and padding of native SVD; `svd(&a, 1e-15)`.
pub fn svd(a: &Matrix<'_>, tolerance: f64) -> SvdResult {
    let relative: f64 = relative_tolerance(tolerance, a.row_number().max(a.col_number()) as usize);
    if a.row_number() == 0 || a.col_number() == 0 || relative < 0.0 || !is_finite_matrix(a) {
        return SvdResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let (normalized, scale) = scaled_input(a);
    let (done, mut u, mut w, mut v) = Svd::new(&normalized).unwrap().into_factors();
    if !done {
        return SvdResult {
            status: Status::NumericalError,
            ..Default::default()
        };
    }
    for i in 1..=w.length() {
        w[i] *= scale;
    }
    sort_factors(&mut u, &mut w, &mut v);
    collect_factors(u, w, v, a.row_number(), relative)
}
fn projection(factors: &SvdResult, b: &Vector<'_>, threshold: f64) -> Vector<'static> {
    let u: &Matrix<'_> = factors.u.as_ref().unwrap();
    let w: &Vector<'_> = factors.singular_values.as_ref().unwrap();
    let mut tmp: Vector<'static> = Vector::with_size(w.length() as usize);
    for j in 0..w.length() {
        let mut sum: f64 = 0.0;
        for i in 0..u.row_number() {
            sum += u[(i, j)] * b.at(i as usize).unwrap();
        }
        if w[j] > threshold {
            tmp[j] = sum / w[j];
        }
    }
    tmp
}
fn right_product(v: &Matrix<'_>, tmp: &Vector<'_>) -> Vector<'static> {
    let mut x: Vector<'static> = Vector::with_size(tmp.length() as usize);
    for i in 0..tmp.length() {
        let mut sum: f64 = 0.0;
        for j in 0..tmp.length() {
            sum += v[(i, j)] * tmp[j];
        }
        x[i] = sum;
    }
    x
}
/// Solves using the native singular-value threshold; `solve_svd(&a,&b,1e-6)`.
pub fn solve_svd(a: &Matrix<'_>, b: &Vector<'_>, tolerance: f64) -> LinearResult {
    let factors: SvdResult = svd(a, tolerance);
    if !factors.is_done() {
        return LinearResult {
            status: factors.status,
            ..Default::default()
        };
    }
    if b.length() != a.row_number() || !is_finite_vector(b) {
        return LinearResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let tmp: Vector<'static> = projection(&factors, b, factors.threshold(tolerance));
    let x: Vector<'static> = right_product(factors.v.as_ref().unwrap(), &tmp);
    LinearResult {
        status: finite_status(is_finite_vector(&x)),
        solution: Some(x),
        ..Default::default()
    }
}
fn inverse_entry(factors: &SvdResult, i: i32, j: i32, threshold: f64) -> f64 {
    let u: &Matrix<'_> = factors.u.as_ref().unwrap();
    let v: &Matrix<'_> = factors.v.as_ref().unwrap();
    let w: &Vector<'_> = factors.singular_values.as_ref().unwrap();
    let mut sum: f64 = 0.0;
    for k in 0..w.length() {
        if w[k] > threshold {
            sum += v[(i, k)] * u[(j, k)] / w[k];
        }
    }
    sum
}
/// Computes the native Moore-Penrose inverse; `pseudo_inverse(&a,1e-6)`.
pub fn pseudo_inverse(a: &Matrix<'_>, tolerance: f64) -> InverseResult {
    let factors: SvdResult = svd(a, tolerance);
    if !factors.is_done() {
        return InverseResult {
            status: factors.status,
            ..Default::default()
        };
    }
    let threshold: f64 = factors.threshold(tolerance);
    let mut inverse: Matrix<'static> =
        Matrix::filled(0, a.col_number() - 1, 0, a.row_number() - 1, 0.0).unwrap();
    for i in 0..a.col_number() {
        for j in 0..a.row_number() {
            inverse[(i, j)] = inverse_entry(&factors, i, j, threshold);
        }
    }
    InverseResult {
        status: finite_status(is_finite_matrix(&inverse)),
        inverse: Some(inverse),
        ..Default::default()
    }
}
/// Returns a condition number only for successful full rank; `condition_number(&a)`.
pub fn condition_number(a: &Matrix<'_>) -> Option<f64> {
    let result: SvdResult = svd(a, 1e-15);
    if result.is_done() {
        result.condition_number
    } else {
        None
    }
}
/// Counts singular values accepted by native tolerance; `numerical_rank(&a,1e-15)`.
pub fn numerical_rank(a: &Matrix<'_>, tolerance: f64) -> usize {
    let result: SvdResult = svd(a, tolerance);
    if result.is_done() { result.rank } else { 0 }
}
