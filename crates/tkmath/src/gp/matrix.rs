// Copyright (c) 1995-1999 Matra Datavision
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

//! gp_Mat with native arithmetic and optional OCCT_DEBUG out-of-line checks.
use super::{Failure, Xyz, construction, dot};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::io;
mod algebra;
mod operations;
/// A row-major 3-by-3 matrix, zero initialized (OCC22595); `Matrix3::default()`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Matrix3 {
    coefficients: [[f64; 3]; 3],
}
impl Matrix3 {
    /// Stores all nine coefficients in row order; `Matrix3::new([[1.0;3];3])`.
    pub const fn new(coefficients: [[f64; 3]; 3]) -> Self {
        Self { coefficients }
    }
    /// Creates a matrix from three columns; `Matrix3::from_columns(x,y,z)`.
    pub fn from_columns(first: Xyz, second: Xyz, third: Xyz) -> Self {
        Self::new([
            *first.coordinates(),
            *second.coordinates(),
            *third.coordinates(),
        ])
        .transposed()
    }
    /// Borrows row-major coefficients; `matrix.coefficients()`.
    pub const fn coefficients(&self) -> &[[f64; 3]; 3] {
        &self.coefficients
    }
    /// Reads a one-based coefficient with native header-level checks; `matrix.value(1,1)?`.
    pub fn value(&self, row: i32, column: i32) -> Result<f64, Failure> {
        Ok(self.coefficients[checked_index(row, " ")?][checked_index(column, " ")?])
    }
    /// Borrows a writable one-based coefficient; `*matrix.change_value(1,1)?=2.0`.
    pub fn change_value(&mut self, row: i32, column: i32) -> Result<&mut f64, Failure> {
        Ok(&mut self.coefficients[checked_index(row, " ")?][checked_index(column, " ")?])
    }
    /// Sets one coefficient; `matrix.set_value(1,1,2.0)?`.
    pub fn set_value(&mut self, row: i32, column: i32, value: f64) -> Result<(), Failure> {
        *self.change_value(row, column)? = value;
        Ok(())
    }
    /// Returns a row using the native compiled-check profile; `matrix.row(1)?`.
    pub fn row(&self, index: i32) -> Result<Xyz, Failure> {
        let row = self.coefficients[outline_index(index, "gp_Mat::Row() - wrong index")?];
        Ok(Xyz::new(row[0], row[1], row[2]))
    }
    /// Returns a column using the native compiled-check profile; `matrix.column(1)?`.
    pub fn column(&self, index: i32) -> Result<Xyz, Failure> {
        let index = outline_index(index, "gp_Mat::Column() - wrong index")?;
        Ok(Xyz::new(
            self.coefficients[0][index],
            self.coefficients[1][index],
            self.coefficients[2][index],
        ))
    }
    /// Replaces a row; `matrix.set_row(1,xyz)?`.
    pub fn set_row(&mut self, index: i32, value: Xyz) -> Result<(), Failure> {
        self.coefficients[outline_index(index, " ")?] = *value.coordinates();
        Ok(())
    }
    /// Replaces a column; `matrix.set_column(1,xyz)?`.
    pub fn set_column(&mut self, index: i32, value: Xyz) -> Result<(), Failure> {
        let column = outline_index(index, " ")?;
        for row in 0..3 {
            self.coefficients[row][column] = value.coordinates()[row];
        }
        Ok(())
    }
    /// Replaces all rows; `matrix.set_rows(x,y,z)`.
    pub fn set_rows(&mut self, first: Xyz, second: Xyz, third: Xyz) {
        self.coefficients = [
            *first.coordinates(),
            *second.coordinates(),
            *third.coordinates(),
        ];
    }
    /// Replaces all columns; `matrix.set_columns(x,y,z)`.
    pub fn set_columns(&mut self, first: Xyz, second: Xyz, third: Xyz) {
        *self = Self::from_columns(first, second, third);
    }
    /// Reads the principal diagonal; `matrix.diagonal()`.
    pub fn diagonal(&self) -> Xyz {
        Xyz::new(
            self.coefficients[0][0],
            self.coefficients[1][1],
            self.coefficients[2][2],
        )
    }
    /// Replaces only diagonal entries; `matrix.set_diagonal(1.0,2.0,3.0)`.
    pub fn set_diagonal(&mut self, x: f64, y: f64, z: f64) {
        self.coefficients[0][0] = x;
        self.coefficients[1][1] = y;
        self.coefficients[2][2] = z;
    }
    /// Clears off-diagonal values and sets a uniform diagonal; `matrix.set_scale(2.0)`.
    pub fn set_scale(&mut self, scale: f64) {
        self.coefficients = [[scale, 0.0, 0.0], [0.0, scale, 0.0], [0.0, 0.0, scale]];
    }
    /// Replaces the matrix with identity; `matrix.set_identity()`.
    pub fn set_identity(&mut self) {
        self.set_scale(1.0);
    }
    /// Constructs the antisymmetric cross-product matrix; `matrix.set_cross(xyz)`.
    pub fn set_cross(&mut self, vector: Xyz) {
        let [x, y, z] = *vector.coordinates();
        self.coefficients = [[0.0, -z, y], [z, 0.0, -x], [-y, x, 0.0]];
    }
    /// Constructs the symmetric outer-product matrix; `matrix.set_dot(xyz)`.
    pub fn set_dot(&mut self, vector: Xyz) {
        let [x, y, z] = *vector.coordinates();
        self.coefficients = [
            [x * x, x * y, x * z],
            [x * y, y * y, y * z],
            [x * z, y * z, z * z],
        ];
    }
    /// Applies the matrix with native row evaluation order; `matrix.apply(xyz)`.
    pub fn apply(&self, vector: Xyz) -> Xyz {
        let v = *vector.coordinates();
        Xyz::new(
            dot(self.coefficients[0], v),
            dot(self.coefficients[1], v),
            dot(self.coefficients[2], v),
        )
    }
    /// Exchanges rows and columns; `matrix.transposed()`.
    pub fn transposed(self) -> Self {
        Self::new(std::array::from_fn(|row| {
            std::array::from_fn(|column| self.coefficients[column][row])
        }))
    }
    /// Transposes in place; `matrix.transpose()`.
    pub fn transpose(&mut self) {
        *self = self.transposed();
    }
    /// Emits the native nine-value fragment; `matrix.dump_json(&mut stream)?`.
    pub fn dump_json(&self, stream: &mut impl DumpOutput) -> io::Result<()> {
        let values: [f64; 9] = std::array::from_fn(|i| self.coefficients[i / 3][i % 3]);
        dump::dump_vector(stream, "gp_Mat", &values)
    }
}
fn checked_index(index: i32, message: &str) -> Result<usize, Failure> {
    if !(1..=3).contains(&index) {
        return Err(Failure::out_of_range(message));
    }
    Ok(index as usize - 1)
}
fn outline_index(index: i32, _message: &str) -> Result<usize, Failure> {
    if cfg!(feature = "occt-debug-checks") {
        return checked_index(index, _message);
    }
    Ok(match index {
        1 => 0,
        2 => 1,
        _ => 2,
    })
}
