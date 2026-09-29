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
    /// Returns the product with left row and right column bounds; `left.multiplied_matrix(&right)?`.
    pub fn multiplied_matrix(&self, right: &Self) -> Result<Self, Failure> {
        if self.col_number() != right.row_number() {
            return Err(dimension(
                "Multiplied",
                "matrices have incompatible dimensions",
            ));
        }
        let mut result = Self::filled(
            self.lower_row(),
            self.upper_row(),
            right.lower_col(),
            right.upper_col(),
            0.0,
        )
        .unwrap();
        for row in self.lower_row()..=self.upper_row() {
            self.product_row(right, &mut result, row);
        }
        Ok(result)
    }
    fn product_row(&self, right: &Self, result: &mut Self, row: i32) {
        for offset in 0..self.col_number() {
            let left = self[(row, self.lower_col() + offset)];
            for col in right.lower_col()..=right.upper_col() {
                result[(row, col)] += left * right[(right.lower_row() + offset, col)];
            }
        }
    }
    /// Multiplies without rebasing or resizing the destination, preserving native partial writes; `left.multiply_matrix(&right)?`.
    pub fn multiply_matrix(&mut self, right: &Self) -> Result<(), Failure> {
        if self.col_number() != right.row_number() {
            return Err(dimension(
                "Multiply",
                "input matrix has incompatible dimensions",
            ));
        }
        let previous = self.clone();
        for row in self.lower_row()..=self.upper_row() {
            self.replace_product_row(&previous, right, row)?;
        }
        Ok(())
    }
    fn replace_product_row(&mut self, left: &Self, right: &Self, row: i32) -> Result<(), Failure> {
        for col in right.lower_col()..=right.upper_col() {
            let mut sum = 0.0;
            for offset in 0..left.col_number() {
                sum += left[(row, left.lower_col() + offset)]
                    * right[(right.lower_row() + offset, col)];
            }
            *self.change_value(row, col)? = sum;
        }
        Ok(())
    }
    /// Applies OCCT's self-aliasing product branch with a saved operand; `matrix.square()?`.
    pub fn square(&mut self) -> Result<(), Failure> {
        if self.col_number() != self.row_number() {
            return Err(dimension(
                "Multiply",
                "input matrix has incompatible dimensions",
            ));
        }
        let previous = self.clone();
        self.set_product(&previous, &previous)
    }
    /// Stores `left * right` into this matrix's bounds with the native i-k-j order, the
    /// two-operand `Multiply`; `result.set_product(&left,&right)?`.
    pub fn set_product(&mut self, left: &Matrix<'_>, right: &Matrix<'_>) -> Result<(), Failure> {
        if left.col_number() != right.row_number()
            || self.row_number() != left.row_number()
            || self.col_number() != right.col_number()
        {
            return Err(dimension(
                "Multiply",
                "matrices have incompatible dimensions",
            ));
        }
        self.init(0.0);
        for row in 0..self.row_number() {
            self.accumulate_product_row(left, right, row);
        }
        Ok(())
    }
    fn accumulate_product_row(&mut self, left: &Matrix<'_>, right: &Matrix<'_>, row: i32) {
        for inner in 0..left.col_number() {
            let value = left[(left.lower_row() + row, left.lower_col() + inner)];
            let target_row = self.lower_row() + row;
            for col in 0..self.col_number() {
                let target_col = self.lower_col() + col;
                self[(target_row, target_col)] +=
                    value * right[(right.lower_row() + inner, right.lower_col() + col)];
            }
        }
    }
    /// Returns `transpose(self) * right` bounded by this matrix's columns and the right
    /// columns, the const `TMultiply`; `left.t_multiply(&right)?`.
    pub fn t_multiply(&self, right: &Matrix<'_>) -> Result<Self, Failure> {
        if self.row_number() != right.row_number() {
            return Err(dimension(
                "TMultiply",
                "matrices have incompatible dimensions",
            ));
        }
        let mut result = Self::with_bounds(
            self.lower_col(),
            self.upper_col(),
            right.lower_col(),
            right.upper_col(),
        )
        .unwrap();
        result.store_transpose_product(self, right);
        Ok(result)
    }
    /// Stores `transpose(t_left) * right` into this matrix's bounds, the two-operand
    /// `TMultiply`; `result.set_transpose_product(&t_left,&right)?`.
    pub fn set_transpose_product(
        &mut self,
        t_left: &Matrix<'_>,
        right: &Matrix<'_>,
    ) -> Result<(), Failure> {
        if t_left.row_number() != right.row_number()
            || self.row_number() != t_left.col_number()
            || self.col_number() != right.col_number()
        {
            return Err(dimension(
                "TMultiply",
                "matrices have incompatible dimensions",
            ));
        }
        self.store_transpose_product(t_left, right);
        Ok(())
    }
    fn store_transpose_product(&mut self, t_left: &Matrix<'_>, right: &Matrix<'_>) {
        for row in 0..self.row_number() {
            for col in 0..self.col_number() {
                let target = (self.lower_row() + row, self.lower_col() + col);
                self[target] = transpose_dot(t_left, right, row, col);
            }
        }
    }
    /// Stores an outer product using declared indices; `matrix.set_outer_product(&left,&right)?`.
    pub fn set_outer_product(
        &mut self,
        left: &Vector<'_>,
        right: &Vector<'_>,
    ) -> Result<(), Failure> {
        if self.row_number() != left.length() || self.col_number() != right.length() {
            return Err(dimension(
                "Multiply",
                "input vectors have incompatible dimensions",
            ));
        }
        for row in self.lower_row()..=self.upper_row() {
            self.outer_row(left, right, row)?;
        }
        Ok(())
    }
    fn outer_row(
        &mut self,
        left: &Vector<'_>,
        right: &Vector<'_>,
        row: i32,
    ) -> Result<(), Failure> {
        for col in self.lower_col()..=self.upper_col() {
            self[(row, col)] = left.value(row)? * right.value(col)?;
        }
        Ok(())
    }
    /// Returns matrix times vector with matrix row bounds; `matrix.multiplied_vector(&vector)?`.
    pub fn multiplied_vector(&self, right: &Vector<'_>) -> Result<Vector<'static>, Failure> {
        if self.col_number() != right.length() {
            return Err(dimension(
                "Multiplied",
                "input vector has incompatible dimensions",
            ));
        }
        let mut result = Vector::with_bounds(self.lower_row(), self.upper_row())?;
        // Native Multiplied(math_Vector) indexes cell by cell, so an empty matrix or
        // vector gives an empty or zero result instead of the pointer walk's raise.
        result.store_product(self, right, false, false);
        Ok(result)
    }
}
fn dimension(operation: &str, message: &str) -> Failure {
    Failure::with_type(
        "Standard_DimensionError",
        &format!("math_Matrix::{operation}() - {message}"),
        "",
    )
}
impl std::ops::MulAssign<&Self> for Matrix<'_> {
    fn mul_assign(&mut self, right: &Self) {
        self.multiply_matrix(right).unwrap();
    }
}
impl std::ops::Mul<&Vector<'_>> for &Matrix<'_> {
    type Output = Vector<'static>;
    fn mul(self, right: &Vector<'_>) -> Self::Output {
        self.multiplied_vector(right).unwrap()
    }
}
/// Column `row` of `t_left` dotted with column `col` of `right`, by position.
fn transpose_dot(t_left: &Matrix<'_>, right: &Matrix<'_>, row: i32, col: i32) -> f64 {
    let mut sum = 0.0;
    for inner in 0..t_left.row_number() {
        // Native `Som = Som + a * b`; `+=` evaluates identically (no contraction).
        sum += t_left[(t_left.lower_row() + inner, t_left.lower_col() + row)]
            * right[(right.lower_row() + inner, right.lower_col() + col)];
    }
    sum
}
