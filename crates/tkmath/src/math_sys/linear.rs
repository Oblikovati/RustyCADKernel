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

//! MathSys native behavior at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::{
    math::{Matrix, Vector},
    math_lin::{
        self,
        utils::{is_finite_matrix, is_finite_vector},
    },
    math_utils::Status,
};
mod qr;
/// Native rank threshold independent of variable units; `RANK_TOLERANCE`.
pub const RANK_TOLERANCE: f64 = 64.0 * f64::EPSILON;
/// Rank-aware linearized step; `LinearStep::default()`.
#[derive(Default)]
pub struct LinearStep {
    /// Native linear solve status; `step.status`.
    pub status: Status,
    /// Optional direction; `step.solution`.
    pub solution: Option<Vector<'static>>,
    /// Rank after column equilibration; `step.rank`.
    pub rank: usize,
}
/// Sequential hypot norm used by MathSys; `safe_norm(&residual)`.
pub fn safe_norm(vector: &Vector<'_>) -> f64 {
    vector
        .values()
        .iter()
        .fold(0.0_f64, |norm, value| norm.hypot(*value))
}
fn equilibrate(matrix: &Matrix<'_>) -> (Matrix<'static>, Vector<'static>) {
    let mut scaled: Matrix<'static> =
        zero_matrix(matrix.row_number() as usize, matrix.col_number() as usize);
    let mut scales: Vector<'static> = Vector::with_size(matrix.col_number() as usize);
    for col in 0..matrix.col_number() as usize {
        let norm: f64 = (0..matrix.row_number() as usize).fold(0.0_f64, |norm, row| {
            norm.hypot(*matrix.at(row, col).unwrap())
        });
        let scale: f64 = if norm > 0.0 { norm } else { 1.0 };
        *scales.change_at(col).unwrap() = scale;
        for row in 0..matrix.row_number() as usize {
            *scaled.change_at(row, col).unwrap() = *matrix.at(row, col).unwrap() / scale;
        }
    }
    (scaled, scales)
}
/// Rank after native column equilibration; `scaled_rank(&jacobian, RANK_TOLERANCE)`.
pub fn scaled_rank(matrix: &Matrix<'_>, tolerance: f64) -> Option<usize> {
    let (scaled, _): (Matrix<'static>, Vector<'static>) = equilibrate(matrix);
    let result: math_lin::SvdResult = math_lin::svd(&scaled, tolerance);
    result.is_done().then_some(result.rank)
}
/// Uses scaled LU/QR for full rank and original-matrix SVD otherwise; `solve_linearized(&jacobian,&rhs,RANK_TOLERANCE)`.
pub fn solve_linearized(matrix: &Matrix<'_>, rhs: &Vector<'_>, tolerance: f64) -> LinearStep {
    if !is_finite_matrix(matrix)
        || !is_finite_vector(rhs)
        || matrix.row_number() as usize != rhs.length() as usize
    {
        return LinearStep {
            status: Status::NumericalError,
            ..Default::default()
        };
    }
    let (scaled, scales): (Matrix<'static>, Vector<'static>) = equilibrate(matrix);
    let svd: math_lin::SvdResult = math_lin::svd(&scaled, tolerance);
    if !svd.is_done() {
        return LinearStep {
            status: svd.status,
            ..Default::default()
        };
    }
    if let Some(mut solution) = full_rank_step(&scaled, rhs, svd.rank, tolerance) {
        for col in 0..matrix.col_number() as usize {
            *solution.change_at(col).unwrap() /= scales.at(col).unwrap();
        }
        return completed_step(solution, svd.rank);
    }
    let result = math_lin::solve_svd(matrix, rhs, tolerance);
    if !result.is_done() {
        return LinearStep {
            status: result.status,
            rank: svd.rank,
            solution: None,
        };
    }
    completed_step(result.solution.unwrap(), svd.rank)
}
fn full_rank_step(
    scaled: &Matrix<'_>,
    rhs: &Vector<'_>,
    rank: usize,
    tolerance: f64,
) -> Option<Vector<'static>> {
    if rank != scaled.col_number() as usize {
        return None;
    }
    if scaled.row_number() as usize == scaled.col_number() as usize {
        let result = math_lin::solve(scaled, rhs, 1e-15);
        return if result.is_done() {
            result.solution
        } else {
            None
        };
    }
    let mut result: Vector<'static> = Vector::with_size(scaled.col_number() as usize);
    qr::solve_scaled_qr(scaled, rhs, &mut result, tolerance).then_some(result)
}
fn completed_step(solution: Vector<'static>, rank: usize) -> LinearStep {
    let status: Status = if is_finite_vector(&solution) {
        Status::Ok
    } else {
        Status::NumericalError
    };
    LinearStep {
        status,
        solution: Some(solution),
        rank,
    }
}

pub(super) fn zero_matrix(rows: usize, cols: usize) -> Matrix<'static> {
    Matrix::filled(0, rows as i32 - 1, 0, cols as i32 - 1, 0.0).unwrap()
}
