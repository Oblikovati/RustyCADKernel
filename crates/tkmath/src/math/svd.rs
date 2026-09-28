// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! Singular-value decomposition preserving native reduction and QR arithmetic.
use super::{Matrix, Vector};
use rustycad_tkernel::standard::Failure;
mod accumulate;
mod qr;
mod reduce;
mod solve;
/// Stateful SVD solver; `Svd::new(&matrix)?`.
pub struct Svd {
    u: Matrix<'static>,
    v: Matrix<'static>,
    diagonal: Vector<'static>,
    rows: i32,
    done: bool,
}
impl Svd {
    /// Pads wide matrices with zero rows before decomposition; `Svd::new(&matrix)?`.
    pub fn new(matrix: &Matrix<'_>) -> Result<Self, Failure> {
        let rows: i32 = matrix.row_number();
        let cols: i32 = matrix.col_number();
        let mut solver: Self = Self {
            u: Matrix::with_bounds(1, rows.max(cols), 1, cols).unwrap(),
            v: Matrix::with_bounds(1, cols, 1, cols).unwrap(),
            diagonal: Vector::with_bounds(1, cols)?,
            rows,
            done: false,
        };
        solver.copy_input(matrix);
        let mut residual: Vector<'static> = Vector::with_bounds(1, cols)?;
        let norm: f64 = solver.bidiagonalize(&mut residual);
        solver.accumulate_right(&residual);
        solver.accumulate_left();
        solver.done = solver.diagonalize(&mut residual, norm);
        Ok(solver)
    }
    fn copy_input(&mut self, matrix: &Matrix<'_>) {
        for row in 0..self.rows {
            for col in 0..self.diagonal.length() {
                self.u[(row + 1, col + 1)] = *matrix.at(row as usize, col as usize).unwrap();
            }
        }
    }
    /// Reports successful convergence; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    pub(crate) fn into_factors(self) -> (bool, Matrix<'static>, Vector<'static>, Matrix<'static>) {
        (self.done, self.u, self.diagonal, self.v)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
fn sign(magnitude: f64, direction: f64) -> f64 {
    if direction >= 0.0 {
        magnitude.abs()
    } else {
        -magnitude.abs()
    }
}
fn pythag(left: f64, right: f64) -> f64 {
    let a: f64 = left.abs();
    let b: f64 = right.abs();
    if a > b {
        let ratio: f64 = b / a;
        return a * (1.0 + ratio * ratio).sqrt();
    }
    if b != 0.0 {
        let ratio: f64 = a / b;
        return b * (1.0 + ratio * ratio).sqrt();
    }
    0.0
}
