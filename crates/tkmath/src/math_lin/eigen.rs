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

//! MathLin_EigenSearch.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math::{Matrix, Vector};
use crate::math_utils::Status;
/// Tridiagonal eigenpairs in native iteration order; `TridiagonalResult::default()`.
#[derive(Default)]
pub struct TridiagonalResult {
    /// Native completion status; `result.status`.
    pub status: Status,
    /// Optional eigenvalues, even after final residual rejection; `result.eigen_values`.
    pub eigen_values: Option<Vector<'static>>,
    /// Eigenvectors stored as columns; `result.eigen_vectors`.
    pub eigen_vectors: Option<Matrix<'static>>,
    /// Validated input dimension; `result.dimension`.
    pub dimension: usize,
}
impl TridiagonalResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&TridiagonalResult> for bool {
    fn from(r: &TridiagonalResult) -> bool {
        r.is_done()
    }
}
struct TridiagonalState {
    diag: Vector<'static>,
    sub: Vector<'static>,
    vectors: Matrix<'static>,
}
impl TridiagonalState {
    fn new(diag: &Vector<'_>, sub: &Vector<'_>) -> Self {
        let n: usize = diag.length() as usize;
        let mut d: Vector<'static> = Vector::with_size(n);
        d.initialized(diag).unwrap();
        let mut e: Vector<'static> = Vector::with_size(n);
        let offset: usize = usize::from(sub.length() == diag.length());
        for i in 0..n - 1 {
            *e.change_at(i).unwrap() = *sub.at(i + offset).unwrap();
        }
        let mut v: Matrix<'static> = Matrix::filled(0, n as i32 - 1, 0, n as i32 - 1, 0.0).unwrap();
        for i in 0..n as i32 {
            v[(i, i)] = 1.0;
        }
        Self {
            diag: d,
            sub: e,
            vectors: v,
        }
    }
    fn end(&self, start: i32) -> i32 {
        for end in start..self.diag.length() - 1 {
            let scale: f64 = self.diag[end].abs().max(self.diag[end + 1].abs());
            if self.sub[end].abs() <= f64::EPSILON * scale {
                return end;
            }
        }
        self.diag.length() - 1
    }
    fn shift(&self, start: i32, end: i32) -> f64 {
        let shift: f64 = (self.diag[start + 1] - self.diag[start]) / (2.0 * self.sub[start]);
        let radius: f64 = 1.0_f64.hypot(shift);
        self.diag[end] - self.diag[start]
            + self.sub[start]
                / if shift < 0.0 {
                    shift - radius
                } else {
                    shift + radius
                }
    }
    fn rotate(&mut self, row: i32, sine: f64, cosine: f64) {
        for i in 0..self.diag.length() {
            let old: f64 = self.vectors[(i, row + 1)];
            self.vectors[(i, row + 1)] = sine * self.vectors[(i, row)] + cosine * old;
            self.vectors[(i, row)] = cosine * self.vectors[(i, row)] - sine * old;
        }
    }
    fn step(&mut self, start: i32, end: i32, mut shift: f64) {
        let mut sine: f64 = 1.0;
        let mut cosine: f64 = 1.0;
        let mut previous: f64 = 0.0;
        for row in (start..end).rev() {
            let temp: f64 = sine * self.sub[row];
            let subtemp: f64 = cosine * self.sub[row];
            let radius: f64 = temp.hypot(shift);
            self.sub[row + 1] = radius;
            if radius == 0.0 {
                self.diag[row + 1] -= previous;
                self.sub[end] = 0.0;
                return;
            }
            sine = temp / radius;
            cosine = shift / radius;
            shift = self.diag[row + 1] - previous;
            let radius_temp: f64 = (self.diag[row] - shift) * sine + 2.0 * cosine * subtemp;
            previous = sine * radius_temp;
            self.diag[row + 1] = shift + previous;
            shift = cosine * radius_temp - subtemp;
            self.rotate(row, sine, cosine);
        }
        self.diag[start] -= previous;
        self.sub[start] = shift;
        self.sub[end] = 0.0;
    }
    fn finite(&self) -> bool {
        self.diag
            .values()
            .iter()
            .chain(self.sub.values())
            .all(|x| x.is_finite())
            && super::jacobi::finite_matrix(&self.vectors)
    }
    fn converge(&mut self, start: i32, limit: u32) -> Status {
        for iteration in 0..=limit {
            let end: i32 = self.end(start);
            if end == start {
                return Status::Ok;
            }
            if iteration == limit {
                return Status::MaxIterations;
            }
            let shift: f64 = self.shift(start, end);
            if !shift.is_finite() {
                return Status::NumericalError;
            }
            self.step(start, end, shift);
            if !self.finite() {
                return Status::NumericalError;
            }
        }
        unreachable!()
    }
}
fn residual_row(
    original: &TridiagonalState,
    result: &TridiagonalState,
    i: i32,
    j: i32,
    scale: f64,
    lambda: f64,
) -> f64 {
    let v: &Matrix<'_> = &result.vectors;
    let mut residual: f64 = original.diag[i] / scale * v[(i, j)] - lambda * v[(i, j)];
    if i > 0 {
        residual += original.sub[i - 1] / scale * v[(i - 1, j)];
    }
    if i + 1 < original.diag.length() {
        residual += original.sub[i] / scale * v[(i + 1, j)];
    }
    residual
}
fn valid_pairs(original: &TridiagonalState, result: &TridiagonalState) -> bool {
    let n: i32 = original.diag.length();
    let mut scale: f64 = 0.0;
    for i in 0..n {
        scale = scale.max(original.diag[i].abs());
        if i + 1 < n {
            scale = scale.max(original.sub[i].abs());
        }
    }
    if scale == 0.0 {
        return true;
    }
    for j in 0..n {
        let lambda: f64 = result.diag[j] / scale;
        let tolerance: f64 = 256.0 * f64::EPSILON * n.max(1) as f64 * lambda.abs().max(1.0);
        for i in 0..n {
            let residual: f64 = residual_row(original, result, i, j, scale, lambda);
            if !residual.is_finite() || residual.abs() > tolerance {
                return false;
            }
        }
    }
    true
}
/// Accepts natural or legacy subdiagonal layout; `eigen_tridiagonal(&diag,&sub,30)`.
pub fn eigen_tridiagonal(diag: &Vector<'_>, sub: &Vector<'_>, limit: u32) -> TridiagonalResult {
    let n: usize = diag.length() as usize;
    if n == 0
        || limit == 0
        || !(sub.length() == diag.length() || sub.length() + 1 == diag.length())
        || !diag
            .values()
            .iter()
            .chain(sub.values())
            .all(|x| x.is_finite())
    {
        return TridiagonalResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let original: TridiagonalState = TridiagonalState::new(diag, sub);
    let mut state: TridiagonalState = TridiagonalState::new(diag, sub);
    for start in 0..n as i32 {
        let status: Status = state.converge(start, limit);
        if status != Status::Ok {
            return TridiagonalResult {
                status,
                dimension: n,
                ..Default::default()
            };
        }
    }
    let status: Status = if state.finite() && valid_pairs(&original, &state) {
        Status::Ok
    } else {
        Status::NumericalError
    };
    TridiagonalResult {
        status,
        dimension: n,
        eigen_values: Some(state.diag),
        eigen_vectors: Some(state.vectors),
    }
}
/// Copies a zero-based column; absent vectors produce the native zero singleton; `get_eigen_vector(&result,0)`.
pub fn get_eigen_vector(
    result: &TridiagonalResult,
    index: usize,
) -> Result<Vector<'static>, rustycad_tkernel::standard::Failure> {
    let Some(v) = result.eigen_vectors.as_ref() else {
        return Ok(Vector::with_size(1));
    };
    v.col(index as i32)
}
