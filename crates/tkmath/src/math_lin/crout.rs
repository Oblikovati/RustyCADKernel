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

//! MathLin_Crout.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::*;
use crate::math::{Matrix, Vector};
use crate::math_utils::{InverseResult, LinearResult, Status};
/// Native LDL-transpose factors and diagnostics; `CroutResult::default()`.
#[derive(Default)]
pub struct CroutResult {
    /// Native outcome; `result.status`.
    pub status: Status,
    /// Unit lower factor; `result.lower`.
    pub lower: Option<Matrix<'static>>,
    /// Signed diagonal pivots; `result.diagonal`.
    pub diagonal: Option<Vector<'static>>,
    /// Full symmetric inverse; `result.inverse`.
    pub inverse: Option<Matrix<'static>>,
    /// Absent if determinant is unrepresentable; `result.determinant`.
    pub determinant: Option<f64>,
    /// Smallest completed pivot; `result.min_abs_pivot`.
    pub min_abs_pivot: f64,
    /// Largest completed pivot; `result.max_abs_pivot`.
    pub max_abs_pivot: f64,
    /// Finite ratio of pivot magnitudes; `result.condition_estimate`.
    pub condition_estimate: Option<f64>,
}
impl CroutResult {
    /// Reports native completion; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&CroutResult> for bool {
    fn from(r: &CroutResult) -> bool {
        r.is_done()
    }
}
fn row_scales(a: &Matrix<'_>) -> Option<Vector<'static>> {
    let n: i32 = a.row_number();
    let mut scale: Vector<'static> = Vector::with_size(n as usize);
    for i in 0..n {
        for j in 0..=i {
            let value: f64 = *a.at(i as usize, j as usize).unwrap();
            if !value.is_finite() {
                return None;
            }
            scale[i] = scale[i].max(value.abs());
        }
    }
    Some(scale)
}
fn lower_row(a: &Matrix<'_>, l: &mut Matrix<'_>, d: &Vector<'_>, i: i32) -> bool {
    for j in 0..i {
        let mut sum: f64 = 0.0;
        for k in 0..j {
            sum += l[(i, k)] * l[(j, k)] * d[k];
        }
        l[(i, j)] = (a.at(i as usize, j as usize).unwrap() - sum) / d[j];
        if !sum.is_finite() || !l[(i, j)].is_finite() {
            return false;
        }
    }
    true
}
fn diagonal_pivot(
    a: &Matrix<'_>,
    l: &Matrix<'_>,
    d: &mut Vector<'_>,
    i: i32,
    scale: f64,
    tol: f64,
) -> Status {
    let mut sum: f64 = 0.0;
    for k in 0..i {
        sum += l[(i, k)] * l[(i, k)] * d[k];
    }
    d[i] = a.at(i as usize, i as usize).unwrap() - sum;
    if !sum.is_finite() || !d[i].is_finite() {
        return Status::NumericalError;
    }
    if scale == 0.0 || d[i].abs() / scale <= tol {
        Status::Singular
    } else {
        Status::Ok
    }
}
fn factor(
    a: &Matrix<'_>,
    l: &mut Matrix<'_>,
    d: &mut Vector<'_>,
    scale: &Vector<'_>,
    tol: f64,
    r: &mut CroutResult,
) -> Option<f64> {
    let mut determinant: f64 = 1.0;
    let mut finite: bool = true;
    for i in 0..a.row_number() {
        if !lower_row(a, l, d, i) {
            r.status = Status::NumericalError;
            return None;
        }
        r.status = diagonal_pivot(a, l, d, i, scale[i], tol);
        if r.status != Status::Ok {
            return None;
        }
        if finite {
            determinant *= d[i];
            finite = determinant.is_finite() && determinant != 0.0;
        }
        let pivot: f64 = d[i].abs();
        r.max_abs_pivot = r.max_abs_pivot.max(pivot);
        r.min_abs_pivot = if i == 0 {
            pivot
        } else {
            r.min_abs_pivot.min(pivot)
        };
        l[(i, i)] = 1.0;
    }
    finite.then_some(determinant)
}
fn reciprocal_diagonal(d: &Vector<'_>) -> Option<Vector<'static>> {
    let mut out: Vector<'static> = Vector::with_size(d.length() as usize);
    for i in 0..d.length() {
        out[i] = 1.0 / d[i];
        if !out[i].is_finite() {
            return None;
        }
    }
    Some(out)
}
fn inverse_lower(l: &mut Matrix<'_>) -> bool {
    l[(0, 0)] = 1.0 / l[(0, 0)];
    for i in 1..l.row_number() {
        for k in 0..i {
            let mut sum: f64 = 0.0;
            for j in k..i {
                sum += l[(i, j)] * l[(j, k)];
            }
            l[(i, k)] = -sum / l[(i, i)];
            if !sum.is_finite() || !l[(i, k)].is_finite() {
                return false;
            }
        }
        l[(i, i)] = 1.0 / l[(i, i)];
        if !l[(i, i)].is_finite() {
            return false;
        }
    }
    true
}
fn inverse_entry(l: &Matrix<'_>, d: &Vector<'_>, i: i32, j: i32) -> f64 {
    let mut sum: f64 = l[(i, j)] * l[(i, i)] * d[i];
    for k in i + 1..l.row_number() {
        sum += l[(k, j)] * l[(k, i)] * d[k];
    }
    sum
}
fn inverse_factors(l: &mut Matrix<'_>, d: &Vector<'_>) -> Option<Matrix<'static>> {
    let diagonal: Vector<'static> = reciprocal_diagonal(d)?;
    if !inverse_lower(l) {
        return None;
    }
    let n: i32 = l.row_number();
    let mut inv: Matrix<'static> = Matrix::filled(0, n - 1, 0, n - 1, 0.0).unwrap();
    for j in 0..n {
        for i in j..n {
            let value: f64 = inverse_entry(l, &diagonal, i, j);
            if !value.is_finite() {
                return None;
            }
            inv[(i, j)] = value;
            inv[(j, i)] = value;
        }
    }
    Some(inv)
}
/// Uses only the lower triangle, including its finite-input checks; `crout(&a,1e-15)`.
pub fn crout(a: &Matrix<'_>, tolerance: f64) -> CroutResult {
    let mut r: CroutResult = CroutResult::default();
    let n: i32 = a.row_number();
    let tol: f64 = relative_tolerance(tolerance, n as usize);
    if n != a.col_number() || n == 0 || tol < 0.0 {
        r.status = Status::InvalidInput;
        return r;
    }
    let Some(scale) = row_scales(a) else {
        r.status = Status::InvalidInput;
        return r;
    };
    let mut l: Matrix<'static> = Matrix::filled(0, n - 1, 0, n - 1, 0.0).unwrap();
    let mut d: Vector<'static> = Vector::with_size(n as usize);
    let determinant: Option<f64> = factor(a, &mut l, &mut d, &scale, tol, &mut r);
    if r.status != Status::Ok {
        return r;
    }
    let factor_l: Matrix<'static> = l.clone();
    let Some(inverse) = inverse_factors(&mut l, &d) else {
        r.status = Status::NumericalError;
        return r;
    };
    r.lower = Some(factor_l);
    r.diagonal = Some(d);
    r.inverse = Some(inverse);
    r.determinant = determinant;
    let condition: f64 = r.max_abs_pivot / r.min_abs_pivot;
    if condition.is_finite() {
        r.condition_estimate = Some(condition);
    }
    r.status = Status::Ok;
    r
}
/// Multiplies by the native Crout inverse, retaining nonfinite solutions; `solve_crout(&a,&b,1e-15)`.
pub fn solve_crout(a: &Matrix<'_>, b: &Vector<'_>, tolerance: f64) -> LinearResult {
    let r: CroutResult = crout(a, tolerance);
    if !r.is_done() {
        return LinearResult {
            status: r.status,
            ..Default::default()
        };
    }
    if b.length() != a.row_number() || !is_finite_vector(b) {
        return LinearResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let inv: &Matrix<'_> = r.inverse.as_ref().unwrap();
    let mut x: Vector<'static> = Vector::with_size(b.length() as usize);
    for i in 0..b.length() {
        let mut sum: f64 = 0.0;
        for j in 0..b.length() {
            sum += inv[(i, j)] * b.at(j as usize).unwrap();
        }
        x[i] = sum;
    }
    let status: Status = if is_finite_vector(&x) {
        Status::Ok
    } else {
        Status::NumericalError
    };
    LinearResult {
        status,
        solution: Some(x),
        determinant: r.determinant,
    }
}
/// Returns the complete symmetric inverse; `invert_crout(&a,1e-15)`.
pub fn invert_crout(a: &Matrix<'_>, tolerance: f64) -> InverseResult {
    let r: CroutResult = crout(a, tolerance);
    if !r.is_done() {
        return InverseResult {
            status: r.status,
            ..Default::default()
        };
    }
    InverseResult {
        status: Status::Ok,
        inverse: r.inverse,
        determinant: r.determinant,
    }
}
