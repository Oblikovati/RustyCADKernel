// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/gp/gp_Mat2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Mat2d with native arithmetic; out-of-line index and determinant checks exist only
//! under `occt-debug-checks`, as gp_Mat2d.cxx compiles them only with OCCT_DEBUG.
use super::{Failure, Xy};
mod algebra;
mod operators;
/// A row-major 2-by-2 matrix, zero initialized; `Matrix2::default()`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Matrix2 {
    pub(super) coefficients: [[f64; 2]; 2],
}
impl Matrix2 {
    /// Stores four coefficients in row order; `Matrix2::new([[1.0,0.0],[0.0,1.0]])`.
    pub const fn new(coefficients: [[f64; 2]; 2]) -> Self {
        Self { coefficients }
    }
    /// Returns the identity matrix; `Matrix2::identity()`.
    pub const fn identity() -> Self {
        Self::new([[1.0, 0.0], [0.0, 1.0]])
    }
    /// Creates a matrix from two columns (gp_Mat2d(gp_XY,gp_XY)); `Matrix2::from_columns(a,b)`.
    pub const fn from_columns(first: Xy, second: Xy) -> Self {
        Self::new([[first.x(), second.x()], [first.y(), second.y()]])
    }
    /// Borrows row-major coefficients; `matrix.coefficients()`.
    pub const fn coefficients(&self) -> &[[f64; 2]; 2] {
        &self.coefficients
    }
    /// Replaces a column; a release build writes any index other than 1 to column 2;
    /// `matrix.set_column(1,xy)?`.
    pub fn set_column(&mut self, index: i32, value: Xy) -> Result<(), Failure> {
        let column = outline_index(index, "gp_Mat2d::SetCol() - invalid index")?;
        self.coefficients[0][column] = value.x();
        self.coefficients[1][column] = value.y();
        Ok(())
    }
    /// Replaces both columns (gp_Mat2d::SetCols); `matrix.set_columns(a,b)`.
    pub fn set_columns(&mut self, first: Xy, second: Xy) {
        *self = Self::from_columns(first, second);
    }
    /// Replaces a row; a release build writes any index other than 1 to row 2;
    /// `matrix.set_row(1,xy)?`.
    pub fn set_row(&mut self, index: i32, value: Xy) -> Result<(), Failure> {
        let row = outline_index(index, "gp_Mat2d::SetRow() - invalid index")?;
        self.coefficients[row] = *value.coordinates();
        Ok(())
    }
    /// Replaces both rows; `matrix.set_rows(a,b)`.
    pub fn set_rows(&mut self, first: Xy, second: Xy) {
        self.coefficients = [*first.coordinates(), *second.coordinates()];
    }
    /// Replaces only the diagonal; `matrix.set_diagonal(1.0,2.0)`.
    pub fn set_diagonal(&mut self, first: f64, second: f64) {
        self.coefficients[0][0] = first;
        self.coefficients[1][1] = second;
    }
    /// Replaces the matrix with identity; `matrix.set_identity()`.
    pub fn set_identity(&mut self) {
        *self = Self::identity();
    }
    /// Replaces the matrix with a counter-clockwise rotation; `matrix.set_rotation(0.5)`.
    pub fn set_rotation(&mut self, angle: f64) {
        let sine = angle.sin();
        let cosine = angle.cos();
        self.coefficients = [[cosine, -sine], [sine, cosine]];
    }
    /// Replaces the matrix with a uniform scale; `matrix.set_scale(2.0)`.
    pub fn set_scale(&mut self, scale: f64) {
        self.coefficients = [[scale, 0.0], [0.0, scale]];
    }
    /// Sets one coefficient with the header's range check; `matrix.set_value(1,2,3.0)?`.
    pub fn set_value(&mut self, row: i32, column: i32, value: f64) -> Result<(), Failure> {
        *self.change_value(row, column)? = value;
        Ok(())
    }
    /// Reads a one-based coefficient (also gp_Mat2d::operator()); `matrix.value(1,2)?`.
    pub fn value(&self, row: i32, column: i32) -> Result<f64, Failure> {
        let (row, column) = checked_cell(row, column)?;
        Ok(self.coefficients[row][column])
    }
    /// Borrows a writable one-based coefficient; `*matrix.change_value(1,2)?=3.0`.
    pub fn change_value(&mut self, row: i32, column: i32) -> Result<&mut f64, Failure> {
        let (row, column) = checked_cell(row, column)?;
        Ok(&mut self.coefficients[row][column])
    }
    /// Returns a column; a release build reads column 2 for any index other than 1;
    /// `matrix.column(1)?`.
    pub fn column(&self, index: i32) -> Result<Xy, Failure> {
        let column = outline_index(index, "gp_Mat2d::Column() - invalid index")?;
        Ok(Xy::new(
            self.coefficients[0][column],
            self.coefficients[1][column],
        ))
    }
    /// Returns a row; a release build reads row 2 for any index other than 1; `matrix.row(1)?`.
    pub fn row(&self, index: i32) -> Result<Xy, Failure> {
        let row = outline_index(index, "gp_Mat2d::Row() - invalid index")?;
        Ok(Xy::new(
            self.coefficients[row][0],
            self.coefficients[row][1],
        ))
    }
    /// Reads the principal diagonal; `matrix.diagonal()`.
    pub fn diagonal(&self) -> Xy {
        Xy::new(self.coefficients[0][0], self.coefficients[1][1])
    }
    /// Computes `a00*a11 - a10*a01`; `matrix.determinant()`.
    pub fn determinant(&self) -> f64 {
        let m = self.coefficients;
        m[0][0] * m[1][1] - m[1][0] * m[0][1]
    }
    /// Compares the absolute determinant with gp::Resolution; `matrix.is_singular()`.
    pub fn is_singular(&self) -> bool {
        self.determinant().abs() <= f64::MIN_POSITIVE
    }
    /// Exchanges the off-diagonal coefficients; `matrix.transposed()`.
    pub fn transposed(self) -> Self {
        let m = self.coefficients;
        Self::new([[m[0][0], m[1][0]], [m[0][1], m[1][1]]])
    }
    /// Transposes in place; `matrix.transpose()`.
    pub fn transpose(&mut self) {
        *self = self.transposed();
    }
}
fn checked_cell(row: i32, column: i32) -> Result<(usize, usize), Failure> {
    if !(1..=2).contains(&row) || !(1..=2).contains(&column) {
        return Err(Failure::out_of_range(&format!(
            "gp_Mat2d - cell ({row}, {column}) outside rows and columns [1, 2]"
        )));
    }
    Ok((row as usize - 1, column as usize - 1))
}
fn outline_index(index: i32, message: &str) -> Result<usize, Failure> {
    if cfg!(feature = "occt-debug-checks") && !(1..=2).contains(&index) {
        return Err(Failure::out_of_range(&format!("{message}: {index}")));
    }
    // Release gp_Mat2d.cxx tests only `index == 1` and treats every other index as 2.
    Ok(if index == 1 { 0 } else { 1 })
}
