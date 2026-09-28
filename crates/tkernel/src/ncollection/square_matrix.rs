// Copyright (c) 2013-2014 OPEN CASCADE SAS
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

//! The members NCollection_Mat3 and NCollection_Mat4 share. Both store `N*N` elements in
//! column-major order, so `columns[c][r]` is native `myMat[c * N + r]`.
use std::ops::{Add, Div, Mul, Sub};
/// Element types with the arithmetic the matrices use (`float`, `double`); `f64: MatrixScalar`.
pub trait MatrixScalar:
    Copy
    + From<u8>
    + PartialEq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
}
impl MatrixScalar for f32 {}
impl MatrixScalar for f64 {}

/// Access, identity/zero and comparison members.
macro_rules! square_matrix_access {
    ($matrix:ident, $n:literal, $nn:literal, $vector:ident) => {
        impl<T: Copy + From<u8>> Default for $matrix<T> {
            /// The native default constructor builds the identity.
            fn default() -> Self {
                let mut columns = [[T::from(0); $n]; $n];
                for (index, column) in columns.iter_mut().enumerate() {
                    column[index] = T::from(1);
                }
                Self { columns }
            }
        }
        impl<T: Copy + From<u8>> $matrix<T> {
            /// Returns the identity, as `Identity`; `Mat4::<f64>::identity().is_identity()`.
            pub fn identity() -> Self {
                Self::default()
            }
            /// Returns the zero matrix, as `Zero`; `Mat4::<f64>::zero().is_zero()`.
            pub fn zero() -> Self {
                Self {
                    columns: [[T::from(0); $n]; $n],
                }
            }
            /// Overwrites every element with zero, as `InitZero`; `matrix.init_zero()`.
            pub fn init_zero(&mut self) {
                *self = Self::zero();
            }
            /// Overwrites the matrix with identity, as `InitIdentity`; `matrix.init_identity()`.
            pub fn init_identity(&mut self) {
                *self = Self::default();
            }
        }
        impl<T: Copy + From<u8> + PartialEq> $matrix<T> {
            /// Compares every element with `!=` against zero, as `IsZero`; `matrix.is_zero()`.
            pub fn is_zero(&self) -> bool {
                *self == Self::zero()
            }
            /// Compares every element with `!=` against identity, as `IsIdentity`; `matrix.is_identity()`.
            pub fn is_identity(&self) -> bool {
                *self == Self::default()
            }
            /// Compares every element with `!=`, as `IsEqual` (NaN is never equal); `a.is_equal(&b)`.
            pub fn is_equal(&self, other: &Self) -> bool {
                self == other
            }
        }
        impl<T> $matrix<T> {
            /// Borrows the column-major storage writably, as `ChangeData`;
            /// `matrix.change_data()[3][0] = 1.0` sets the element at row 0, column 3.
            pub fn change_data(&mut self) -> &mut [[T; $n]; $n] {
                &mut self.columns
            }
            /// Borrows one element writably, as `ChangeValue`/`operator()`; `*matrix.change_value(0, 3) = 1.0`.
            pub fn change_value(&mut self, row: usize, column: usize) -> &mut T {
                &mut self.columns[column][row]
            }
        }
        impl<T: Copy> $matrix<T> {
            /// Builds a matrix from column-major values, as `Map`; `Mat4::from_data([0.0; 16])`.
            pub fn from_data(data: [T; $nn]) -> Self {
                let columns = std::array::from_fn(|column| {
                    std::array::from_fn(|row| data[column * $n + row])
                });
                Self { columns }
            }
            /// Copies the column-major storage, as `GetData`; `matrix.data()`.
            pub fn data(&self) -> [T; $nn] {
                std::array::from_fn(|index| self.columns[index / $n][index % $n])
            }
            /// Reads one zero-based element, as `GetValue`/const `operator()`; `matrix.get_value(0, 3)`.
            pub fn get_value(&self, row: usize, column: usize) -> T {
                self.columns[column][row]
            }
            /// Writes one element, as `SetValue`; `matrix.set_value(0, 0, 2.0)`.
            pub fn set_value(&mut self, row: usize, column: usize, value: T) {
                self.columns[column][row] = value;
            }
            /// Reads a row, as `GetRow`; `matrix.get_row(0)`.
            pub fn get_row(&self, row: usize) -> $vector<T> {
                $vector::from_array(std::array::from_fn(|column| self.columns[column][row]))
            }
            /// Replaces a row, as `SetRow`; `matrix.set_row(0, v)`.
            pub fn set_row(&mut self, row: usize, values: $vector<T>) {
                for (column, value) in values.data().into_iter().enumerate() {
                    self.columns[column][row] = value;
                }
            }
            /// Reads a column, as `GetColumn`; `matrix.get_column(0)`.
            pub fn get_column(&self, column: usize) -> $vector<T> {
                $vector::from_array(self.columns[column])
            }
            /// Replaces a column, as `SetColumn`; `matrix.set_column(0, v)`.
            pub fn set_column(&mut self, column: usize, values: $vector<T>) {
                self.columns[column] = values.data();
            }
            /// Reads the diagonal, as `GetDiagonal`; `matrix.get_diagonal()`.
            pub fn get_diagonal(&self) -> $vector<T> {
                $vector::from_array(std::array::from_fn(|index| self.columns[index][index]))
            }
            /// Replaces the diagonal, as `SetDiagonal`; `matrix.set_diagonal(v)`.
            pub fn set_diagonal(&mut self, values: $vector<T>) {
                for (index, value) in values.data().into_iter().enumerate() {
                    self.columns[index][index] = value;
                }
            }
            /// Swaps rows and columns, as `Transposed`; `matrix.transposed()`.
            pub fn transposed(&self) -> Self {
                let columns = std::array::from_fn(|column| {
                    std::array::from_fn(|row| self.columns[row][column])
                });
                Self { columns }
            }
            /// Transposes in place, as `Transpose`; `matrix.transpose()`.
            pub fn transpose(&mut self) {
                *self = self.transposed();
            }
            /// Converts every element with `static_cast`, as `ConvertFrom`/`Convert`;
            /// `matrix.convert_from(&other)`.
            pub fn convert_from<U: Copy + crate::ncollection::ElementCast<T>>(
                &mut self,
                other: &$matrix<U>,
            ) {
                self.columns = other
                    .columns
                    .map(|column| column.map(crate::ncollection::ElementCast::cast));
            }
            /// The templated converting constructor; `Mat4::<f32>::from_other(&double_matrix)`.
            pub fn from_other<U: Copy + crate::ncollection::ElementCast<T>>(
                other: &$matrix<U>,
            ) -> Self {
                Self {
                    columns: other
                        .columns
                        .map(|column| column.map(crate::ncollection::ElementCast::cast)),
                }
            }
        }
    };
}
/// Products, scalar operations, sums and negation.
macro_rules! square_matrix_arithmetic {
    ($matrix:ident, $n:literal, $vector:ident) => {
        impl<T: $crate::ncollection::MatrixScalar> $matrix<T> {
            /// The static `Multiply(A, B)`: each element starts at 0 and adds `A(r,k)*B(k,c)`
            /// for increasing k; `Mat4::product(&a, &b)`.
            pub fn product(left: &Self, right: &Self) -> Self {
                let columns = std::array::from_fn(|column| {
                    std::array::from_fn(|row| {
                        (0..$n).fold(T::from(0), |sum, k| {
                            sum + left.columns[k][row] * right.columns[column][k]
                        })
                    })
                });
                Self { columns }
            }
            /// Right-multiplies in place, as `Multiply(Mat)`; `matrix.multiply(&other)`.
            pub fn multiply(&mut self, other: &Self) {
                *self = Self::product(self, other);
            }
            /// Returns `self * other`, as `Multiplied(Mat)`; `matrix.multiplied(&other)`.
            pub fn multiplied(&self, other: &Self) -> Self {
                Self::product(self, other)
            }
            /// Multiplies every element, as `Multiply(Element_t)`; `matrix.multiply_scalar(2.0)`.
            pub fn multiply_scalar(&mut self, factor: T) {
                self.columns = self.columns.map(|column| column.map(|value| value * factor));
            }
            /// Returns a scaled copy, as `Multiplied(Element_t)`; `matrix.multiplied_scalar(2.0)`.
            pub fn multiplied_scalar(&self, factor: T) -> Self {
                let mut copy: Self = *self;
                copy.multiply_scalar(factor);
                copy
            }
            /// Divides every element (no reciprocal), as `Divide`; `matrix.divide(2.0)`.
            pub fn divide(&mut self, divisor: T) {
                self.columns = self.columns.map(|column| column.map(|value| value / divisor));
            }
            /// Returns a divided copy, as `Divided`; `matrix.divided(2.0)`.
            pub fn divided(&self, divisor: T) -> Self {
                let mut copy: Self = *self;
                copy.divide(divisor);
                copy
            }
            /// Adds element-wise, as `Add`; `matrix.add(&other)`.
            pub fn add(&mut self, other: &Self) {
                self.zip_assign(other, |a, b| a + b);
            }
            /// Returns the element-wise sum, as `Added`; `matrix.added(&other)`.
            pub fn added(&self, other: &Self) -> Self {
                let mut copy: Self = *self;
                copy.add(other);
                copy
            }
            /// Subtracts element-wise, as `Subtract`; `matrix.subtract(&other)`.
            pub fn subtract(&mut self, other: &Self) {
                self.zip_assign(other, |a, b| a - b);
            }
            /// Returns the element-wise difference, as `Subtracted`; `matrix.subtracted(&other)`.
            pub fn subtracted(&self, other: &Self) -> Self {
                let mut copy: Self = *self;
                copy.subtract(other);
                copy
            }
            fn zip_assign(&mut self, other: &Self, combine: fn(T, T) -> T) {
                for (column, other_column) in self.columns.iter_mut().zip(&other.columns) {
                    for (value, &other_value) in column.iter_mut().zip(other_column) {
                        *value = combine(*value, other_value);
                    }
                }
            }
            /// Applies the matrix to a column vector, summing row products left to right.
            fn apply(&self, vector: $vector<T>) -> $vector<T> {
                let values = vector.data();
                $vector::from_array(std::array::from_fn(|row| {
                    (1..$n).fold(self.columns[0][row] * values[0], |sum, k| {
                        sum + self.columns[k][row] * values[k]
                    })
                }))
            }
        }
        impl<T: $crate::ncollection::MatrixScalar + std::ops::Neg<Output = T>> $matrix<T> {
            /// Negates every element, as `Negated`/unary `operator-`; `matrix.negated()`.
            pub fn negated(&self) -> Self {
                Self { columns: self.columns.map(|column| column.map(|value| -value)) }
            }
        }
        impl<T: $crate::ncollection::MatrixScalar + std::ops::Neg<Output = T>> std::ops::Neg for $matrix<T> {
            type Output = Self;
            fn neg(self) -> Self {
                self.negated()
            }
        }
        impl<T: $crate::ncollection::MatrixScalar> std::ops::Mul<$vector<T>> for $matrix<T> {
            type Output = $vector<T>;
            fn mul(self, vector: $vector<T>) -> $vector<T> {
                self.apply(vector)
            }
        }
        impl<T: $crate::ncollection::MatrixScalar> std::ops::Mul for $matrix<T> {
            type Output = Self;
            fn mul(self, right: Self) -> Self {
                Self::product(&self, &right)
            }
        }
        impl<T: $crate::ncollection::MatrixScalar> std::ops::MulAssign for $matrix<T> {
            fn mul_assign(&mut self, right: Self) {
                self.multiply(&right);
            }
        }
        impl<T: $crate::ncollection::MatrixScalar> std::ops::Add for $matrix<T> {
            type Output = Self;
            fn add(self, right: Self) -> Self {
                self.added(&right)
            }
        }
        impl<T: $crate::ncollection::MatrixScalar> std::ops::AddAssign for $matrix<T> {
            fn add_assign(&mut self, right: Self) {
                $matrix::add(self, &right);
            }
        }
        impl<T: $crate::ncollection::MatrixScalar> std::ops::Sub for $matrix<T> {
            type Output = Self;
            fn sub(self, right: Self) -> Self {
                self.subtracted(&right)
            }
        }
        impl<T: $crate::ncollection::MatrixScalar> std::ops::SubAssign for $matrix<T> {
            fn sub_assign(&mut self, right: Self) {
                self.subtract(&right);
            }
        }
        square_matrix_arithmetic!(@scalar $matrix, f32);
        square_matrix_arithmetic!(@scalar $matrix, f64);
    };
    (@scalar $matrix:ident, $scalar:ty) => {
        impl std::ops::Mul<$scalar> for $matrix<$scalar> {
            type Output = Self;
            fn mul(self, factor: $scalar) -> Self {
                self.multiplied_scalar(factor)
            }
        }
        impl std::ops::MulAssign<$scalar> for $matrix<$scalar> {
            fn mul_assign(&mut self, factor: $scalar) {
                self.multiply_scalar(factor);
            }
        }
        impl std::ops::Div<$scalar> for $matrix<$scalar> {
            type Output = Self;
            fn div(self, divisor: $scalar) -> Self {
                self.divided(divisor)
            }
        }
        impl std::ops::DivAssign<$scalar> for $matrix<$scalar> {
            fn div_assign(&mut self, divisor: $scalar) {
                self.divide(divisor);
            }
        }
    };
}
/// `DumpJson`: the elements in row order under the class name, read as doubles.
macro_rules! square_matrix_dump {
    ($matrix:ident, $n:literal, $name:literal) => {
        square_matrix_dump!(@one $matrix, $n, $name, f32);
        square_matrix_dump!(@one $matrix, $n, $name, f64);
    };
    (@one $matrix:ident, $n:literal, $name:literal, $scalar:ty) => {
        impl $matrix<$scalar> {
            #[doc = concat!("Writes `\"", $name, "\": [...]` in row order; `matrix.dump_json(&mut stream)?`.")]
            pub fn dump_json(
                &self,
                output: &mut impl $crate::standard::dump::DumpOutput,
            ) -> std::io::Result<()> {
                let values: [f64; $n * $n] =
                    std::array::from_fn(|index| f64::from(self.get_value(index / $n, index % $n)));
                $crate::standard::dump::dump_vector(output, $name, &values)
            }
        }
    };
}
pub(crate) use {square_matrix_access, square_matrix_arithmetic, square_matrix_dump};
