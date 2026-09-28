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
use crate::math::Vector;
use rustycad_tkernel::standard::Failure;
impl Matrix<'_> {
    /// Reads a declared cell with native exception category; `matrix.value(1,1)?`.
    pub fn value(&self, row: i32, col: i32) -> Result<&f64, Failure> {
        self.table
            .value(row, col)
            .map_err(|_| Failure::out_of_range("NCollection_Array2::Value"))
    }
    /// Mutates a declared cell with native exception category; `*matrix.change_value(1,1)?=2.0`.
    pub fn change_value(&mut self, row: i32, col: i32) -> Result<&mut f64, Failure> {
        self.table
            .change_value(row, col)
            .map_err(|_| Failure::out_of_range("NCollection_Array2::Value"))
    }
    /// Copies a row using matrix column bounds; `matrix.row(1)?`.
    pub fn row(&self, row: i32) -> Result<Vector<'static>, Failure> {
        let mut result = Vector::with_bounds(self.lower_col(), self.upper_col())?;
        for col in self.lower_col()..=self.upper_col() {
            result[col] = *self.value(row, col)?;
        }
        Ok(result)
    }
    /// Copies a column using matrix row bounds; `matrix.col(1)?`.
    pub fn col(&self, col: i32) -> Result<Vector<'static>, Failure> {
        let mut result = Vector::with_bounds(self.lower_row(), self.upper_row())?;
        for row in self.lower_row()..=self.upper_row() {
            result[row] = *self.value(row, col)?;
        }
        Ok(result)
    }
    /// Assigns a row by vector position; `matrix.set_row(1,&vector)?`.
    pub fn set_row(&mut self, row: i32, vector: &Vector<'_>) -> Result<(), Failure> {
        if vector.length() != self.col_number() {
            return Err(dimension("SetRow"));
        }
        for offset in 0..vector.length() {
            *self.change_value(row, self.lower_col() + offset)? = vector[vector.lower() + offset];
        }
        Ok(())
    }
    /// Assigns a column by vector position; `matrix.set_col(1,&vector)?`.
    pub fn set_col(&mut self, col: i32, vector: &Vector<'_>) -> Result<(), Failure> {
        if vector.length() != self.row_number() {
            return Err(dimension("SetCol"));
        }
        for offset in 0..vector.length() {
            *self.change_value(self.lower_row() + offset, col)? = vector[vector.lower() + offset];
        }
        Ok(())
    }
    /// Swaps two rows after copying both; `matrix.swap_row(1,2)?`.
    pub fn swap_row(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        let left = self.row(first)?;
        let right = self.row(second)?;
        self.set_row(first, &right)?;
        self.set_row(second, &left)
    }
    /// Swaps two columns after copying both; `matrix.swap_col(1,2)?`.
    pub fn swap_col(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        let left = self.col(first)?;
        let right = self.col(second)?;
        self.set_col(first, &right)?;
        self.set_col(second, &left)
    }
    /// Sets matching declared row/column indices, as OCCT does; `matrix.set_diag(1.0)?`.
    pub fn set_diag(&mut self, value: f64) -> Result<(), Failure> {
        if self.row_number() != self.col_number() {
            return Err(Failure::with_type(
                "math_NotSquare",
                "math_Matrix::SetDiag() - matrix is not square",
                "",
            ));
        }
        for index in self.lower_row()..=self.upper_row() {
            *self.change_value(index, index)? = value;
        }
        Ok(())
    }
}
fn dimension(operation: &str) -> Failure {
    Failure::with_type(
        "Standard_DimensionError",
        &format!("math_Matrix::{operation}() - input vector has wrong dimensions"),
        "",
    )
}
