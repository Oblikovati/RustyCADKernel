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

use super::{Crout, Matrix, Vector};
impl Crout {
    pub(super) fn factor(
        &mut self,
        matrix: &Matrix<'_>,
        lower: &mut Matrix<'_>,
        diagonal: &mut Vector<'_>,
        minimum: f64,
    ) -> bool {
        for row in 1..=matrix.row_number() {
            factor_row(matrix, lower, diagonal, row);
            let mut scale = 0.0;
            for k in 1..row {
                scale += lower[(row, k)] * lower[(row, k)] * diagonal[k];
            }
            diagonal[row] =
                matrix[(row + matrix.lower_row() - 1, row + matrix.lower_col() - 1)] - scale;
            self.determinant *= diagonal[row];
            if diagonal[row].abs() <= minimum {
                return false;
            }
            lower[(row, row)] = 1.0;
        }
        true
    }
}
fn factor_row(matrix: &Matrix<'_>, lower: &mut Matrix<'_>, diagonal: &Vector<'_>, row: i32) {
    for col in 1..row {
        let mut scale = 0.0;
        for k in 1..col {
            scale += lower[(row, k)] * lower[(col, k)] * diagonal[k];
        }
        lower[(row, col)] = (matrix[(row + matrix.lower_row() - 1, col + matrix.lower_col() - 1)]
            - scale)
            / diagonal[col];
    }
}
