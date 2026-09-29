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
// Rust translation helpers for the NCollection_Array2 accesses of
// ModelingData/TKGeomBase/GeomConvert and GeomLib at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `NCollection_Array1` and one-based `NCollection_Array2` accesses of the
//! GeomConvert and GeomLib ports: allocation and checked element access with the
//! native exceptions (Standard_RangeError for bad bounds, Standard_OutOfRange for a
//! bad index).
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Matrix;

/// `array(index)` of a cloneable element; `element(&curves, 0, "Curves")?`.
pub(crate) fn element<T: Clone>(
    array: &Array1<'_, T>,
    index: i32,
    context: &str,
) -> Result<T, Failure> {
    array
        .value(index)
        .cloned()
        .map_err(|error| array_failure(error, context))
}

/// `array(index) = value`; `store(&mut curves, 0, curve, "Curves")?`.
pub(crate) fn store<T>(
    array: &mut Array1<'_, T>,
    index: i32,
    value: T,
    context: &str,
) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|error| array_failure(error, context))
}

/// `NCollection_Array1<T>(0, count - 1)`; `zero_based::<f64>(3)?`.
pub(crate) fn zero_based<T: Default>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(0, count - 1).map_err(|error| array_failure(error, "NCollection_Array1"))
}

/// `math_Matrix(RowLower, RowUpper, ColLower, ColUpper, InitialValue)`;
/// `matrix(1, 3, 1, 3, 0.0)?`.
pub(crate) fn matrix(
    rl: i32,
    ru: i32,
    cl: i32,
    cu: i32,
    value: f64,
) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(rl, ru, cl, cu, value).map_err(|error| array_failure(error, "math_Matrix"))
}

/// `NCollection_Array2<T>(1, rows, 1, cols)`; `grid::<f64>(2, 3)?`.
pub(crate) fn grid<T: Default>(rows: i32, cols: i32) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, rows, 1, cols)
        .map_err(|error| array_failure(error, "NCollection_Array2"))
}

/// `array(row, col)` of a cloneable element; `at(&poles, 1, 1, "Poles")?`.
pub(crate) fn at<T: Clone>(
    array: &Array2<'_, T>,
    row: i32,
    col: i32,
    context: &str,
) -> Result<T, Failure> {
    array
        .value(row, col)
        .cloned()
        .map_err(|error| array_failure(error, context))
}

/// `array(row, col) = value`; `put(&mut poles, 1, 1, point, "Poles")?`.
pub(crate) fn put<T>(
    array: &mut Array2<'_, T>,
    row: i32,
    col: i32,
    value: T,
    context: &str,
) -> Result<(), Failure> {
    array
        .set_value(row, col, value)
        .map_err(|error| array_failure(error, context))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_access_uses_one_based_native_bounds() {
        let mut values: Array2<'static, f64> = grid(2, 3).unwrap();
        put(&mut values, 2, 3, 4.5, "test").unwrap();
        assert_eq!(at(&values, 2, 3, "test").unwrap(), 4.5);
        let error = at(&values, 3, 1, "test").unwrap_err();
        assert_eq!(error.exception_type(), "Standard_OutOfRange");
    }
}
