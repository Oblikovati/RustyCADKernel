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
use crate::math::{Matrix, Vector};
use crate::math_lin::utils::is_finite_vector;
pub(super) fn solve_scaled_qr(
    a: &Matrix<'_>,
    b: &Vector<'_>,
    x: &mut Vector<'_>,
    tolerance: f64,
) -> bool {
    let rows: usize = a.row_number() as usize;
    let cols: usize = a.col_number() as usize;
    if rows < cols {
        return false;
    }
    let mut work: Matrix<'static> = super::zero_matrix(rows, cols);
    let mut rhs: Vector<'static> = Vector::with_size(rows);
    for row in 0..rows {
        *rhs.change_at(row).unwrap() = *b.at(row).unwrap();
        for col in 0..cols {
            *work.change_at(row, col).unwrap() = *a.at(row, col).unwrap();
        }
    }
    for col in 0..cols {
        if !reflect_column(&mut work, &mut rhs, col) {
            return false;
        }
    }
    back_substitute(&work, &rhs, x, tolerance)
}
fn reflect_column(work: &mut Matrix<'_>, rhs: &mut Vector<'_>, col: usize) -> bool {
    let rows: usize = work.row_number() as usize;
    let norm: f64 = (col..rows).fold(0.0_f64, |norm, row| norm.hypot(*work.at(row, col).unwrap()));
    if !(norm > 0.0 && norm.is_finite()) {
        return false;
    }
    let alpha: f64 = norm.copysign(*work.at(col, col).unwrap());
    *work.change_at(col, col).unwrap() += alpha;
    let squared: f64 = (col..rows)
        .map(|row| {
            let x = *work.at(row, col).unwrap();
            x * x
        })
        .fold(0.0, |sum, value| sum + value);
    if !(squared > 0.0 && squared.is_finite()) {
        return false;
    }
    for next in col + 1..work.col_number() as usize {
        reflect_next(work, col, next, squared);
    }
    let dot: f64 = (col..rows)
        .map(|row| work.at(row, col).unwrap() * rhs.at(row).unwrap())
        .fold(0.0, |sum, value| sum + value);
    let factor: f64 = 2.0 * dot / squared;
    for row in col..rows {
        *rhs.change_at(row).unwrap() -= factor * work.at(row, col).unwrap();
    }
    *work.change_at(col, col).unwrap() = -alpha;
    true
}
fn reflect_next(work: &mut Matrix<'_>, col: usize, next: usize, squared: f64) {
    let rows: usize = work.row_number() as usize;
    let dot: f64 = (col..rows)
        .map(|row| work.at(row, col).unwrap() * work.at(row, next).unwrap())
        .fold(0.0, |sum, value| sum + value);
    let factor: f64 = 2.0 * dot / squared;
    for row in col..rows {
        let reflection: f64 = factor * work.at(row, col).unwrap();
        *work.change_at(row, next).unwrap() -= reflection;
    }
}
fn back_substitute(
    work: &Matrix<'_>,
    rhs: &Vector<'_>,
    x: &mut Vector<'_>,
    tolerance: f64,
) -> bool {
    for col in (0..work.col_number() as usize).rev() {
        let mut value: f64 = *rhs.at(col).unwrap();
        for next in col + 1..work.col_number() as usize {
            value -= work.at(col, next).unwrap() * x.at(next).unwrap();
        }
        let diagonal: f64 = *work.at(col, col).unwrap();
        if !(diagonal.abs() > tolerance && diagonal.is_finite()) {
            return false;
        }
        *x.change_at(col).unwrap() = value / diagonal;
    }
    is_finite_vector(x)
}
