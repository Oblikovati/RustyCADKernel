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

//! Column-major 4x4 transforms with upstream multiplication ordering.
use super::{Vec3, Vec4};
use std::ops::{Add, Mul};

/// A four-column matrix, initially identity; `Mat4::<f64>::default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4<T> {
    columns: [[T; 4]; 4],
}
impl<T: Copy + From<u8>> Default for Mat4<T> {
    fn default() -> Self {
        let mut columns = [[T::from(0); 4]; 4];
        for (index, column) in columns.iter_mut().enumerate() {
            column[index] = T::from(1);
        }
        Self { columns }
    }
}
impl<T: Copy + From<u8> + Add<Output = T> + Mul<Output = T>> Mat4<T> {
    /// Postmultiplies a translation matrix; `matrix.translate(Vec3::new(1.0,2.0,3.0))`.
    pub fn translate(&mut self, offset: Vec3<T>) {
        let mut translation = Self::default();
        translation.columns[3] = [offset.x, offset.y, offset.z, T::from(1)];
        *self = *self * translation;
    }
    fn row_times_vector(&self, row: usize, vector: Vec4<T>) -> T {
        self.columns[0][row] * vector.x
            + self.columns[1][row] * vector.y
            + self.columns[2][row] * vector.z
            + self.columns[3][row] * vector.w
    }
}
impl<T: Copy + From<u8> + Add<Output = T> + Mul<Output = T>> Mul<Vec4<T>> for Mat4<T> {
    type Output = Vec4<T>;
    fn mul(self, vector: Vec4<T>) -> Vec4<T> {
        Vec4::new(
            self.row_times_vector(0, vector),
            self.row_times_vector(1, vector),
            self.row_times_vector(2, vector),
            self.row_times_vector(3, vector),
        )
    }
}
impl<T: Copy + From<u8> + Add<Output = T> + Mul<Output = T>> Mul for Mat4<T> {
    type Output = Self;
    fn mul(self, right: Self) -> Self {
        let mut result = Self::default();
        for element in 0..16 {
            result.columns[element / 4][element % 4] =
                self.product_element(right, element % 4, element / 4);
        }
        result
    }
}
impl<T: Copy + From<u8> + Add<Output = T> + Mul<Output = T>> Mat4<T> {
    fn product_element(&self, right: Self, row: usize, column: usize) -> T {
        let mut result = T::from(0);
        for inner in 0..4 {
            result = result + self.columns[inner][row] * right.columns[column][inner];
        }
        result
    }
}
