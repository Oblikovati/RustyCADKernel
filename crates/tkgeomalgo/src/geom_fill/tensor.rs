// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Tensor.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Tensor`: a 1-based three-index array of reals that multiplies a vector
//! into a matrix.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};

/// `GeomFill_Tensor`: `nbrow x nbcol x nbmat` reals in one array, the matrix index
/// varying fastest; an index outside the array raises Standard_OutOfRange, as the
/// native NCollection_Array1 does.
///
/// ```ignore
/// let mut t = GeomFillTensor::new(2, 2, 3)?;
/// t.init(0.0);
/// *t.change_value(1, 2, 3)? = 4.0;
/// ```
#[derive(Clone, Debug)]
pub struct GeomFillTensor {
    tab: Array1<'static, f64>,
    nbrow: i32,
    nbcol: i32,
    nbmat: i32,
    nbmtcl: i32,
}

fn out_of_range(detail: String) -> Failure {
    Failure::with_type("Standard_OutOfRange", &detail, "")
}

impl GeomFillTensor {
    /// `GeomFill_Tensor(NbRow, NbCol, NbMat)`: the array is not initialized natively;
    /// here it starts at zero. A non-positive size raises as the native array.
    pub fn new(nb_row: i32, nb_col: i32, nb_mat: i32) -> Result<Self, Failure> {
        let size = nb_row * nb_mat * nb_col;
        let tab = Array1::with_bounds(1, size).map_err(|error| {
            out_of_range(format!(
                "GeomFill_Tensor({nb_row}, {nb_col}, {nb_mat}): {error}"
            ))
        })?;
        Ok(Self {
            tab,
            nbrow: nb_row,
            nbcol: nb_col,
            nbmat: nb_mat,
            nbmtcl: nb_mat * nb_col,
        })
    }

    /// `Init(InitialValue)`: every entry; `tensor.init(0.0)`.
    pub fn init(&mut self, initial_value: f64) {
        self.tab.init(initial_value);
    }

    // `Mat + nbmat * (Col - 1) + nbmtcl * (Row - 1)`, checked against the array.
    fn index(&self, row: i32, col: i32, mat: i32) -> Result<i32, Failure> {
        let index = mat + self.nbmat * (col - 1) + self.nbmtcl * (row - 1);
        if index < self.tab.lower() || index > self.tab.upper() {
            return Err(out_of_range(format!(
                "GeomFill_Tensor::Value({row}, {col}, {mat}): flat index {index} outside 1..{}",
                self.tab.upper()
            )));
        }
        Ok(index)
    }

    /// `Value(Row, Col, Mat)`; `tensor.value(1, 1, 1)?`.
    pub fn value(&self, row: i32, col: i32, mat: i32) -> Result<f64, Failure> {
        let index = self.index(row, col, mat)?;
        Ok(self.tab[index])
    }

    /// `ChangeValue(Row, Col, Mat)`; `*tensor.change_value(1, 1, 1)? = 2.0`.
    pub fn change_value(&mut self, row: i32, col: i32, mat: i32) -> Result<&mut f64, Failure> {
        let index = self.index(row, col, mat)?;
        Ok(&mut self.tab[index])
    }

    /// `Multiply(Right, Product)`: `Product(i, j) = sum_k T(i, j, k) * Right(k)`;
    /// `tensor.multiply(&right, &mut product)?`.
    pub fn multiply(&self, right: &Vector<'_>, product: &mut Matrix<'_>) -> Result<(), Failure> {
        for i in 1..=self.nbrow {
            for j in 1..=self.nbcol {
                *product.change_value(i, j)? = self.row_sum(right, i, j)?;
            }
        }
        Ok(())
    }

    // The native `Somme += Value(i, j, k) * Right(k)` loop.
    fn row_sum(&self, right: &Vector<'_>, i: i32, j: i32) -> Result<f64, Failure> {
        let mut somme = 0.0;
        for k in 1..=self.nbmat {
            somme += self.value(i, j, k)? * *right.value(k)?;
        }
        Ok(somme)
    }
}
