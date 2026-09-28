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

use super::Matrix;
use crate::math::{Gauss, not_square, singular_matrix};
use rustycad_tkernel::standard::Failure;
impl Matrix<'_> {
    /// Returns zero when native LU regards the matrix as singular; `matrix.determinant()?`.
    pub fn determinant(&self) -> Result<f64, Failure> {
        let solver = Gauss::new(self)?;
        if solver.is_done() {
            solver.determinant()
        } else {
            Ok(0.0)
        }
    }
    /// Inverts via native-order LU, preserving the input on singular failure; `matrix.invert()?`.
    pub fn invert(&mut self) -> Result<(), Failure> {
        if self.row_number() != self.col_number() {
            return Err(not_square("math_Matrix::Invert() - matrix is not square"));
        }
        let solver = Gauss::new(self)?;
        if !solver.is_done() {
            return Err(singular_matrix(""));
        }
        solver.invert(self)
    }
    /// Copies before inversion; `matrix.inverse()?`.
    pub fn inverse(&self) -> Result<Self, Failure> {
        let mut result = self.clone();
        result.invert()?;
        Ok(result)
    }
}
