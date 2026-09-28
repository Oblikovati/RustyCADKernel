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

//! Partial-pivoting LU decomposition matching math_Gauss and math_Recipes.
use super::{Matrix, Vector};
use rustycad_tkernel::standard::Failure;
mod decompose;
mod solve;
/// Reusable native-order Gaussian decomposition; `Gauss::new(&matrix)?`.
pub struct Gauss {
    lu: Matrix<'static>,
    pivots: Vec<i32>,
    sign: f64,
    done: bool,
}
impl Gauss {
    // Newton callbacks receive the in-place LU factors before writing the next Jacobian.
    pub(super) fn factor_matrix(&self) -> &Matrix<'static> {
        &self.lu
    }

    /// Uses OCCT's default minimum pivot of 1e-20; `Gauss::new(&matrix)?`.
    pub fn new(matrix: &Matrix<'_>) -> Result<Self, Failure> {
        Self::with_min_pivot(matrix, 1e-20)
    }
    /// Decomposes using the supplied absolute pivot threshold; `Gauss::with_min_pivot(&matrix,1e-12)?`.
    pub fn with_min_pivot(matrix: &Matrix<'_>, minimum: f64) -> Result<Self, Failure> {
        if matrix.row_number() != matrix.col_number() {
            return Err(super::not_square(" "));
        }
        let n = matrix.row_number();
        let mut lu = Matrix::with_bounds(1, n, 1, n).unwrap();
        lu.initialized(matrix)?;
        let mut result = Self {
            lu,
            pivots: vec![0; n as usize],
            sign: 1.0,
            done: false,
        };
        result.done = result.decompose(minimum);
        Ok(result)
    }
    /// Reports successful decomposition; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Multiplies the signed diagonal in order; `solver.determinant()?`.
    pub fn determinant(&self) -> Result<f64, Failure> {
        self.check_done()?;
        let mut result = self.sign;
        for index in 1..=self.lu.row_number() {
            result *= self.lu[(index, index)];
        }
        Ok(result)
    }
    /// Solves in place with an explicit size check; `solver.solve(&mut rhs)?`.
    pub fn solve(&self, rhs: &mut Vector<'_>) -> Result<(), Failure> {
        self.check_done()?;
        if rhs.length() != self.lu.row_number() {
            return Err(error("Standard_DimensionError", ""));
        }
        self.solve_lu(rhs)
    }
    /// Copies RHS to existing bounds then solves, preserving the overload's validation order; `solver.solve_into(&rhs,&mut solution)?`.
    pub fn solve_into(&self, rhs: &Vector<'_>, solution: &mut Vector<'_>) -> Result<(), Failure> {
        self.check_done()?;
        solution.initialized(rhs)?;
        self.solve_lu(solution)
    }
    /// Writes the inverse using one solve per column; `solver.invert(&mut matrix)?`.
    pub fn invert(&self, inverse: &mut Matrix<'_>) -> Result<(), Failure> {
        self.check_done()?;
        let n = self.lu.row_number();
        if inverse.row_number() != n || inverse.col_number() != n {
            return Err(error("Standard_DimensionError", " "));
        }
        let mut column = Vector::with_bounds(1, n).unwrap();
        for j in 1..=n {
            self.inverse_column(inverse, &mut column, j)?;
        }
        Ok(())
    }
    fn inverse_column(
        &self,
        inverse: &mut Matrix<'_>,
        column: &mut Vector<'_>,
        j: i32,
    ) -> Result<(), Failure> {
        column.init(0.0);
        column[j] = 1.0;
        self.solve_lu(column)?;
        let (lower_row, lower_col) = (inverse.lower_row(), inverse.lower_col());
        for i in 1..=self.lu.row_number() {
            inverse[(i + lower_row - 1, j + lower_col - 1)] = column[i];
        }
        Ok(())
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(error("StdFail_NotDone", " "));
        }
        Ok(())
    }
}
fn error(kind: &'static str, message: &str) -> Failure {
    Failure::with_type(kind, message, "")
}
