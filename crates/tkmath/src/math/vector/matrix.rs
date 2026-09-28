// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2023 OPEN CASCADE SAS
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

use super::{Failure, VectorBase, VectorItem, failure};
use crate::math::Matrix;
impl<T: VectorItem> VectorBase<'_, T> {
    /// Stores matrix times column vector; `result.set_matrix_vector(&matrix,&vector)?`.
    pub fn set_matrix_vector(
        &mut self,
        matrix: &Matrix<'_>,
        vector: &VectorBase<'_, T>,
    ) -> Result<(), Failure> {
        self.matrix_product(matrix, vector, false, false, "Multiply")
    }
    /// Stores row vector times matrix; `result.set_vector_matrix(&vector,&matrix)?`.
    pub fn set_vector_matrix(
        &mut self,
        vector: &VectorBase<'_, T>,
        matrix: &Matrix<'_>,
    ) -> Result<(), Failure> {
        self.matrix_product(matrix, vector, true, true, "Multiply")
    }
    /// Stores transposed matrix times column vector; `result.set_transpose_matrix_vector(&matrix,&vector)?`.
    pub fn set_transpose_matrix_vector(
        &mut self,
        matrix: &Matrix<'_>,
        vector: &VectorBase<'_, T>,
    ) -> Result<(), Failure> {
        self.matrix_product(matrix, vector, true, false, "TMultiply")
    }
    /// Stores row vector times transposed matrix; `result.set_vector_transpose_matrix(&vector,&matrix)?`.
    pub fn set_vector_transpose_matrix(
        &mut self,
        vector: &VectorBase<'_, T>,
        matrix: &Matrix<'_>,
    ) -> Result<(), Failure> {
        self.matrix_product(matrix, vector, false, true, "TMultiply")
    }
    /// Returns a row product with matrix column bounds; `vector.multiplied_matrix(&matrix)?`.
    pub fn multiplied_matrix(&self, matrix: &Matrix<'_>) -> Result<Self, Failure> {
        if self.length() != matrix.row_number() {
            return Err(failure(
                "Standard_DimensionError",
                "math_VectorBase::Multiplied() - input matrix has wrong dimensions",
            ));
        }
        let mut result = Self::with_bounds(matrix.lower_col(), matrix.upper_col()).unwrap();
        result.set_vector_matrix(self, matrix)?;
        Ok(result)
    }
    fn matrix_product(
        &mut self,
        matrix: &Matrix<'_>,
        vector: &VectorBase<'_, T>,
        transpose: bool,
        vector_first: bool,
        operation: &str,
    ) -> Result<(), Failure> {
        let (outputs, inputs) = if transpose {
            (matrix.col_number(), matrix.row_number())
        } else {
            (matrix.row_number(), matrix.col_number())
        };
        if self.length() != outputs || vector.length() != inputs {
            return Err(failure(
                "Standard_DimensionError",
                &format!(
                    "math_VectorBase::{operation}() - input matrix and /or vector have wrong dimensions"
                ),
            ));
        }
        for output in 0..outputs as usize {
            self.values_mut()[output] = matrix_component(
                matrix,
                vector,
                output,
                inputs as usize,
                transpose,
                vector_first,
            );
        }
        Ok(())
    }
}
// Operand order preserves the native overloads, including NaN representation choices.
#[allow(clippy::if_same_then_else)]
fn matrix_component<T: VectorItem>(
    matrix: &Matrix<'_>,
    vector: &VectorBase<'_, T>,
    output: usize,
    inputs: usize,
    transpose: bool,
    vector_first: bool,
) -> T {
    // The native accumulator has the item type: each `aSum += double` is a double sum
    // converted back, which truncates every partial sum of an integer vector.
    let mut sum = T::ZERO;
    for input in 0..inputs {
        let coefficient = *if transpose {
            matrix.at(input, output)
        } else {
            matrix.at(output, input)
        }
        .unwrap();
        let value = vector.values()[input].to_real();
        let product = if vector_first {
            value * coefficient
        } else {
            coefficient * value
        };
        sum = T::from_real(sum.to_real() + product);
    }
    sum
}
