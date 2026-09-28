// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Element, row, column and diagonal access of NCollection_Mat4 (row, column order).
use super::Mat4;
use crate::ncollection::{Vec3, Vec4};

impl<T: Copy + From<u8>> Mat4<T> {
    /// Returns the identity matrix, as `Identity`; `Mat4::<f64>::identity().is_identity()`.
    pub fn identity() -> Self {
        Self::default()
    }
    /// Returns the zero matrix, as `Zero`; `Mat4::<f64>::zero().is_zero()`.
    pub fn zero() -> Self {
        let mut matrix = Self::default();
        matrix.init_zero();
        matrix
    }
    /// Overwrites every element with zero, as `InitZero`; `matrix.init_zero()`.
    pub fn init_zero(&mut self) {
        self.columns = [[T::from(0); 4]; 4];
    }
    /// Overwrites the matrix with identity, as `InitIdentity`; `matrix.init_identity()`.
    pub fn init_identity(&mut self) {
        *self = Self::default();
    }
    /// Builds a matrix from 16 column-major values, as `Map`; `Mat4::from_data([0.0; 16])`.
    pub fn from_data(data: [T; 16]) -> Self {
        let columns =
            std::array::from_fn(|column| std::array::from_fn(|row| data[column * 4 + row]));
        Self { columns }
    }
}
impl<T: Copy + From<u8> + PartialEq> Mat4<T> {
    /// Compares against zero without tolerance, as `IsZero`; `Mat4::<f64>::zero().is_zero()`.
    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
    /// Compares against identity without tolerance, as `IsIdentity`; `matrix.is_identity()`.
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }
}
impl<T: Copy> Mat4<T> {
    /// Reads one element; row and column are zero-based; `matrix.get_value(0, 3)`.
    pub fn get_value(&self, row: usize, column: usize) -> T {
        self.columns[column][row]
    }
    /// Borrows one element mutably, as `ChangeValue`; `*matrix.change_value(0, 3) = 1.0`.
    pub fn change_value(&mut self, row: usize, column: usize) -> &mut T {
        &mut self.columns[column][row]
    }
    /// Writes one element, as `SetValue`; `matrix.set_value(0, 0, 2.0)`.
    pub fn set_value(&mut self, row: usize, column: usize, value: T) {
        self.columns[column][row] = value;
    }
    /// Copies the raw column-major storage, as `GetData`; `matrix.data()[12]` is X translation.
    pub fn data(&self) -> [T; 16] {
        std::array::from_fn(|index| self.columns[index / 4][index % 4])
    }
    /// Reads a row, as `GetRow`; `matrix.get_row(0)`.
    pub fn get_row(&self, row: usize) -> Vec4<T> {
        Vec4::new(
            self.get_value(row, 0),
            self.get_value(row, 1),
            self.get_value(row, 2),
            self.get_value(row, 3),
        )
    }
    /// Changes the first three row values, as `SetRow(Vec3)`; `matrix.set_row3(0, v)`.
    pub fn set_row3(&mut self, row: usize, values: Vec3<T>) {
        self.set_value(row, 0, values.x);
        self.set_value(row, 1, values.y);
        self.set_value(row, 2, values.z);
    }
    /// Changes a whole row, as `SetRow(Vec4)`; `matrix.set_row(0, v)`.
    pub fn set_row(&mut self, row: usize, values: Vec4<T>) {
        self.set_row3(row, values.xyz());
        self.set_value(row, 3, values.w);
    }
    /// Reads a column, as `GetColumn`; `matrix.get_column(3)` is the translation.
    pub fn get_column(&self, column: usize) -> Vec4<T> {
        let [x, y, z, w] = self.columns[column];
        Vec4::new(x, y, z, w)
    }
    /// Changes the first three column values, as `SetColumn(Vec3)`; `matrix.set_column3(3, t)`.
    pub fn set_column3(&mut self, column: usize, values: Vec3<T>) {
        self.set_value(0, column, values.x);
        self.set_value(1, column, values.y);
        self.set_value(2, column, values.z);
    }
    /// Changes a whole column, as `SetColumn(Vec4)`; `matrix.set_column(3, v)`.
    pub fn set_column(&mut self, column: usize, values: Vec4<T>) {
        self.set_column3(column, values.xyz());
        self.set_value(3, column, values.w);
    }
    /// Reads the diagonal, as `GetDiagonal`; `Mat4::<f64>::identity().get_diagonal().w == 1.0`.
    pub fn get_diagonal(&self) -> Vec4<T> {
        Vec4::new(
            self.get_value(0, 0),
            self.get_value(1, 1),
            self.get_value(2, 2),
            self.get_value(3, 3),
        )
    }
    /// Changes the first three diagonal values, as `SetDiagonal(Vec3)`; `matrix.set_diagonal3(s)`.
    pub fn set_diagonal3(&mut self, values: Vec3<T>) {
        self.set_value(0, 0, values.x);
        self.set_value(1, 1, values.y);
        self.set_value(2, 2, values.z);
    }
    /// Changes the whole diagonal, as `SetDiagonal(Vec4)`; `matrix.set_diagonal(s)`.
    pub fn set_diagonal(&mut self, values: Vec4<T>) {
        self.set_diagonal3(values.xyz());
        self.set_value(3, 3, values.w);
    }
    /// Swaps rows and columns, as `Transposed`; `matrix.transposed()`.
    pub fn transposed(&self) -> Self {
        let columns =
            std::array::from_fn(|column| std::array::from_fn(|row| self.columns[row][column]));
        Self { columns }
    }
}
