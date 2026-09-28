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

//! math_SVD: a stored `SVD_Decompose` factorization with truncated solves.
use super::recipes::{STATUS_OK, svd_decompose, svd_solve};
use super::{Matrix, Vector};
use rustycad_tkernel::standard::Failure;
use std::fmt;
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
        let mut u = Matrix::filled(1, rows.max(cols), 1, cols, 0.0).unwrap();
        u.set(1, rows, 1, cols, matrix)?;
        let mut v = Matrix::with_bounds(1, cols, 1, cols).unwrap();
        let mut diagonal = Vector::with_bounds(1, cols)?;
        let done = svd_decompose(&mut u, &mut diagonal, &mut v)? == STATUS_OK;
        Ok(Self {
            u,
            v,
            diagonal,
            rows,
            done,
        })
    }
    /// Reports successful convergence; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    pub(crate) fn into_factors(self) -> (bool, Matrix<'static>, Vector<'static>, Matrix<'static>) {
        (self.done, self.u, self.diagonal, self.v)
    }
    /// Zeroes singular values below `epsilon` times the largest, then solves; the cutoff
    /// persists for later calls, as natively. `solver.solve(&rhs,&mut output,1e-6)?`.
    pub fn solve(
        &mut self,
        rhs: &Vector<'_>,
        output: &mut Vector<'_>,
        epsilon: f64,
    ) -> Result<(), Failure> {
        self.check_done()?;
        let mut padded: Vector<'static> = Vector::filled(1, self.u.row_number(), 0.0)?;
        padded.set(1, rhs.length(), rhs)?;
        self.truncate(epsilon);
        if output.lower() == 1 {
            return svd_solve(&self.u, &self.diagonal, &self.v, &padded, output);
        }
        // Native code solves into a 1-based view sharing `output`'s storage.
        let mut view: Vector<'static> = Vector::with_bounds(1, output.length())?;
        view.initialized(output)?;
        svd_solve(&self.u, &self.diagonal, &self.v, &padded, &mut view)?;
        output.initialized(&view)
    }
    /// Writes the one-based pseudoinverse and retains the cutoff; `solver.pseudo_inverse(&mut inverse,1e-6)?`.
    pub fn pseudo_inverse(
        &mut self,
        inverse: &mut Matrix<'_>,
        epsilon: f64,
    ) -> Result<(), Failure> {
        self.check_done()?;
        self.truncate(epsilon);
        let mut basis: Vector<'static> = Vector::with_bounds(1, self.u.row_number())?;
        let mut column: Vector<'static> = Vector::with_bounds(1, self.diagonal.length())?;
        for col in 1..=self.rows {
            basis.init(0.0);
            basis[col] = 1.0;
            svd_solve(&self.u, &self.diagonal, &self.v, &basis, &mut column)?;
            for row in 1..=self.diagonal.length() {
                *inverse.change_value(row, col)? = column[row];
            }
        }
        Ok(())
    }
    fn truncate(&mut self, epsilon: f64) {
        let minimum: f64 = epsilon * self.diagonal[self.diagonal.max_index()];
        for index in 1..=self.diagonal.length() {
            if self.diagonal[index] < minimum {
                self.diagonal[index] = 0.0;
            }
        }
    }
    /// Writes the native status text; `solver.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        write!(stream, "math_SVD")?;
        if self.done {
            writeln!(stream, " Status = Done ")
        } else {
            writeln!(stream, " Status = not Done ")
        }
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
display_via_dump!(Svd);
