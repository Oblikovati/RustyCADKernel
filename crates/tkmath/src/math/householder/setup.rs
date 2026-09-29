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

use super::{Failure, Householder, Matrix};
impl Householder {
    pub(super) fn construct(
        matrix: &Matrix<'_>,
        rhs: &Matrix<'_>,
        bounds: [i32; 4],
        q_lower_col: i32,
        epsilon: f64,
    ) -> Result<Self, Failure> {
        let [rl, ru, cl, cu] = bounds;
        let (rows, cols) = (ru - rl + 1, cu - cl + 1);
        let mut result = Self {
            solution: allocate(1, cols, 1, rhs.col_number())?,
            q: allocate(1, rows, q_lower_col, q_lower_col + cols - 1)?,
            done: false,
        };
        let mut transformed = allocate(1, rows, 1, rhs.col_number())?;
        for row in 1..=rows {
            result.copy_input_row(matrix, rhs, &mut transformed, row, rl, cl)?;
        }
        result.reduce(&mut transformed, epsilon)?;
        Ok(result)
    }
    fn copy_input_row(
        &mut self,
        matrix: &Matrix<'_>,
        rhs: &Matrix<'_>,
        transformed: &mut Matrix<'_>,
        row: i32,
        rl: i32,
        cl: i32,
    ) -> Result<(), Failure> {
        for col in 1..=self.q.col_number() {
            *self.q.change_value(row, col)? = *matrix.value(row + rl - 1, col + cl - 1)?;
        }
        for col in 1..=rhs.col_number() {
            transformed[(row, col)] = *rhs.value(row + rhs.lower_row() - 1, col)?;
        }
        Ok(())
    }
}
fn allocate(rl: i32, ru: i32, cl: i32, cu: i32) -> Result<Matrix<'static>, Failure> {
    Matrix::with_bounds(rl, ru, cl, cu).map_err(|error| error.into_failure("math_Householder"))
}
