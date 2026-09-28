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

//! Householder transformation and simultaneous RHS reduction from pinned OCCT 3d097a0.
use super::{Matrix, Vector};
use rustycad_tkernel::standard::Failure;
mod reduction;
mod setup;
/// Direct least-squares solver retaining OCCT's reflection sign rule; `Householder::from_vector(&a,&b,1e-20)?`.
pub struct Householder {
    solution: Matrix<'static>,
    q: Matrix<'static>,
    done: bool,
}
impl Householder {
    /// Solves a vector RHS with the supplied native epsilon; `Householder::from_vector(&a,&b,1e-20)?`.
    pub fn from_vector(
        matrix: &Matrix<'_>,
        rhs: &Vector<'_>,
        epsilon: f64,
    ) -> Result<Self, Failure> {
        let mut column = Matrix::with_bounds(1, rhs.length(), 1, 1).unwrap();
        column.set_col(1, rhs)?;
        Self::construct(
            matrix,
            &column,
            [
                matrix.lower_row(),
                matrix.upper_row(),
                matrix.lower_col(),
                matrix.upper_col(),
            ],
            1,
            epsilon,
        )
    }
    /// Solves multiple RHS columns, retaining the native Q column bounds; `Householder::from_matrix(&a,&b,1e-20)?`.
    pub fn from_matrix(
        matrix: &Matrix<'_>,
        rhs: &Matrix<'_>,
        epsilon: f64,
    ) -> Result<Self, Failure> {
        Self::construct(
            matrix,
            rhs,
            [
                matrix.lower_row(),
                matrix.upper_row(),
                matrix.lower_col(),
                matrix.upper_col(),
            ],
            matrix.lower_col(),
            epsilon,
        )
    }
    /// Selects an inclusive A submatrix and the first matching RHS rows; `Householder::from_range(&a,&b,[1,3,1,2],1e-20)?`.
    pub fn from_range(
        matrix: &Matrix<'_>,
        rhs: &Matrix<'_>,
        bounds: [i32; 4],
        epsilon: f64,
    ) -> Result<Self, Failure> {
        Self::construct(matrix, rhs, bounds, 1, epsilon)
    }
    /// Reports successful reduction; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Copies a selected one-based solution column into existing vector bounds; `solver.value(&mut solution,1)?`.
    pub fn value(&self, solution: &mut Vector<'_>, index: i32) -> Result<(), Failure> {
        self.check_done()?;
        if index < 1 || index > self.solution.col_number() {
            return Err(Failure::out_of_range(" "));
        }
        solution.initialized(&self.solution.col(index)?)
    }
    /// Borrows the complete one-based solution matrix; `solver.all_values()?`.
    pub fn all_values(&self) -> Result<&Matrix<'static>, Failure> {
        self.check_done()?;
        Ok(&self.solution)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
