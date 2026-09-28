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

//! Normal-equation least squares, matching math_GaussLeastSquare's arithmetic.
use super::recipes::{STATUS_OK, lu_decompose, lu_solve};
use super::{IntegerVector, Matrix, Vector};
use rustycad_tkernel::standard::Failure;
use std::fmt;
/// Reusable normal-equation decomposition; `GaussLeastSquares::new(&matrix)?`.
pub struct GaussLeastSquares {
    lu: Matrix<'static>,
    transpose: Matrix<'static>,
    index: IntegerVector<'static>,
    done: bool,
}
impl GaussLeastSquares {
    /// Uses OCCT's default pivot threshold; `GaussLeastSquares::new(&matrix)?`.
    pub fn new(matrix: &Matrix<'_>) -> Result<Self, Failure> {
        Self::with_min_pivot(matrix, 1e-20)
    }
    /// Forms A-transpose times A and decomposes it in native order; `GaussLeastSquares::with_min_pivot(&matrix,1e-15)?`.
    pub fn with_min_pivot(matrix: &Matrix<'_>, minimum: f64) -> Result<Self, Failure> {
        let (rows, cols) = (matrix.row_number(), matrix.col_number());
        let mut transpose = Matrix::with_bounds(1, cols, 1, rows).unwrap();
        transpose.initialized(&matrix.transposed())?;
        let mut lu = Matrix::with_bounds(1, cols, 1, cols).unwrap();
        lu.set_product(&transpose, matrix)?;
        let mut index = IntegerVector::with_bounds(1, cols)?;
        let mut sign = 0.0;
        let done = lu_decompose(&mut lu, &mut index, &mut sign, minimum)? == STATUS_OK;
        Ok(Self {
            lu,
            transpose,
            index,
            done,
        })
    }
    /// Reports a successful normal-equation decomposition; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Solves using the existing output bounds; `solver.solve_into(&rhs,&mut solution)?`.
    pub fn solve_into(&self, rhs: &Vector<'_>, solution: &mut Vector<'_>) -> Result<(), Failure> {
        self.check_done()?;
        // Native out-of-line dimension macros are disabled; larger buffers retain their tail.
        for row in 0..self.transpose.row_number() as usize {
            *solution.change_at(row)? = self.rhs_component(rhs, row)?;
        }
        lu_solve(&self.lu, &self.index, solution)
    }
    /// Retains sequential RHS/output aliasing during the transposed product; `solver.solve_in_place(&mut rhs)?`.
    pub fn solve_in_place(&self, rhs: &mut Vector<'_>) -> Result<(), Failure> {
        self.check_done()?;
        for row in 0..self.transpose.row_number() as usize {
            let value = self.rhs_component(rhs, row)?;
            *rhs.change_at(row)? = value;
        }
        lu_solve(&self.lu, &self.index, rhs)
    }
    fn rhs_component(&self, rhs: &Vector<'_>, row: usize) -> Result<f64, Failure> {
        let mut sum = 0.0;
        for col in 0..self.transpose.col_number() as usize {
            sum += self.transpose.at(row, col).unwrap() * rhs.at(col)?;
        }
        Ok(sum)
    }
    /// Writes the native status text, including its asymmetric spacing;
    /// `solver.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        write!(stream, "math_GaussLeastSquare ")?;
        if self.done {
            writeln!(stream, " Status = Done ")
        } else {
            writeln!(stream, "Status = not Done ")
        }
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.is_done() {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
