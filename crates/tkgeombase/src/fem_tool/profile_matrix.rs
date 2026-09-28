// Created on: 1997-10-31
// Created by: Roman BORISOV
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
// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_ProfileMatrix.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FEmTool_ProfileMatrix`: a symmetric matrix stored by profile (skyline), with
//! its Cholesky factorization.
mod layout;
mod solve;

use super::sparse_matrix::{EntryRef, SparseMatrix};
use crate::native_failure::failure;
use layout::ProfileLayout;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::math::Vector;
use std::ops::{Deref, DerefMut};
use std::sync::RwLockWriteGuard;

// The mutable part of the matrix: the profile coefficients (`ProfileMatrix`), the
// Cholesky factor (`SMatrix`) and `IsDecomp`.
struct ProfileValues {
    matrix: Vec<f64>,
    factor: Vec<f64>,
    is_decomp: bool,
}

/// `FEmTool_ProfileMatrix`: row `i` stores the columns `FirstIndexes(i)..=i`.
///
/// ```
/// use rustycad_tkgeombase::fem_tool::{ProfileMatrix, SparseMatrix};
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::math::Vector;
/// let matrix = ProfileMatrix::new(&Array1::from_shared(&[1, 1], 1))?;
/// matrix.init(0.0);
/// *matrix.change_value(1, 1)? = 4.0;
/// *matrix.change_value(2, 1)? = 2.0;
/// *matrix.change_value(2, 2)? = 5.0;
/// assert!(matrix.decompose()?);
/// let (b, mut x) = (Vector::filled(1, 2, 2.0)?, Vector::filled(1, 2, 0.0)?);
/// matrix.solve(&b, &mut x)?;
/// assert_eq!((x[1], x[2]), (0.375, 0.25));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct ProfileMatrix {
    parent: StandardTransient,
    layout: ProfileLayout,
    values: SharedState<ProfileValues>,
}
impl_transient!(
    ProfileMatrix,
    "FEmTool_ProfileMatrix",
    dyn SparseMatrix,
    parent,
    interfaces: [dyn SparseMatrix]
);

impl ProfileMatrix {
    /// `FEmTool_ProfileMatrix(FirstIndexes)`: `FirstIndexes(i)` (read from index 1,
    /// whatever the array's lower bound) is the first stored column of row i. The
    /// coefficients are uninitialized natively; they start at 0 here.
    /// `ProfileMatrix::new(&first_indexes)?`.
    pub fn new(first_indexes: &Array1<'_, i32>) -> Result<Self, Failure> {
        let layout: ProfileLayout = ProfileLayout::new(first_indexes)?;
        let size: usize = layout.size();
        Ok(Self {
            parent: StandardTransient::default(),
            layout,
            values: SharedState::new(ProfileValues {
                matrix: vec![0.; size],
                factor: vec![0.; size],
                is_decomp: false,
            }),
        })
    }

    /// `IsInProfile(i, j)`: whether the entry (i, j) is stored;
    /// `matrix.is_in_profile(2, 1)?`.
    pub fn is_in_profile(&self, i: i32, j: i32) -> Result<bool, Failure> {
        if j <= i {
            return Ok(i - j <= self.layout.height(i)?);
        }
        Ok(j - i <= self.layout.height(j)?)
    }

    /// `OutM()`: prints the profile coefficients and NextCoeff on stdout, as the
    /// native member prints on std::cout; `matrix.out_m()?`.
    pub fn out_m(&self) -> Result<(), Failure> {
        print!(
            "{}",
            self.write_rows("Matrix A", |values| &values.matrix, true)?
        );
        Ok(())
    }

    /// `OutS()`: prints the Cholesky factor on stdout; `matrix.out_s()?`.
    pub fn out_s(&self) -> Result<(), Failure> {
        print!(
            "{}",
            self.write_rows("Matrix S", |values| &values.factor, false)?
        );
        Ok(())
    }

    /// The text `OutM()` (`with_next`) or `OutS()` prints; `matrix.out_text(true)?`.
    pub fn out_text(&self, matrix_a: bool) -> Result<String, Failure> {
        if matrix_a {
            return self.write_rows("Matrix A", |values| &values.matrix, true);
        }
        self.write_rows("Matrix S", |values| &values.factor, false)
    }

    // One line per row: "0 " for each column before the profile, then the stored
    // entries; OutM adds the NextCoeff table.
    fn write_rows(
        &self,
        title: &str,
        pick: impl Fn(&ProfileValues) -> &Vec<f64>,
        with_next: bool,
    ) -> Result<String, Failure> {
        let values = self.values.read();
        let mut text: String = format!("{title}\n");
        for i in 1..=self.layout.rows() {
            let (height, diagonal) = (self.layout.height(i)?, self.layout.diagonal(i)?);
            (1..i - height).for_each(|_| text.push_str("0 "));
            for address in diagonal - height..=diagonal {
                text.push_str(&format!(
                    "{} ",
                    stream_real(*layout::slot(pick(&values), address)?)
                ));
            }
            text.push('\n');
        }
        if with_next {
            text.push_str("NextCoeff\n");
            for address in 1..=self.layout.diagonal(self.layout.rows())? {
                text.push_str(&format!("{} ", self.layout.next(address)?));
            }
            text.push('\n');
        }
        Ok(text)
    }
}

// `Standard_OStream << double` with default flags (C's `%g`).
fn stream_real(value: f64) -> String {
    String::from_utf8_lossy(rustycad_tkernel::tcollection::AsciiString::from_real(value).as_bytes())
        .into_owned()
}

// The locked matrix and the address of one of its coefficients.
struct ProfileEntry<'a> {
    values: RwLockWriteGuard<'a, ProfileValues>,
    index: usize,
}

impl Deref for ProfileEntry<'_> {
    type Target = f64;
    fn deref(&self) -> &f64 {
        &self.values.matrix[self.index]
    }
}

impl DerefMut for ProfileEntry<'_> {
    fn deref_mut(&mut self) -> &mut f64 {
        &mut self.values.matrix[self.index]
    }
}

impl SparseMatrix for ProfileMatrix {
    fn init(&self, value: f64) {
        let mut values = self.values.write();
        values.matrix.fill(value);
        values.is_decomp = false;
    }

    // The native range checks are compiled out: an entry outside the profile but
    // inside the storage aliases another coefficient, as natively; an address
    // outside the storage (undefined natively) is Standard_OutOfRange.
    fn change_value(&self, i: i32, j: i32) -> Result<EntryRef<'_>, Failure> {
        let address: i32 = if i < j {
            self.layout.diagonal(j)? - (j - i)
        } else {
            self.layout.diagonal(i)? - (i - j)
        };
        let index: usize = layout::index(address, self.layout.size())?;
        Ok(EntryRef::new(ProfileEntry {
            values: self.values.write(),
            index,
        }))
    }

    fn decompose(&self) -> Result<bool, Failure> {
        let mut values = self.values.write();
        let ProfileValues {
            matrix,
            factor,
            is_decomp,
        } = &mut *values;
        *is_decomp = false;
        *is_decomp = solve::decompose(&self.layout, matrix, factor)?;
        Ok(*is_decomp)
    }

    fn solve(&self, b: &Vector<'_>, x: &mut Vector<'_>) -> Result<(), Failure> {
        let values = self.values.read();
        if !values.is_decomp {
            return Err(failure("StdFail_NotDone", "Decomposition must be done"));
        }
        solve::solve(&self.layout, &values.factor, b, x)
    }

    fn prepare(&self) -> Result<bool, Failure> {
        Err(failure(
            "Standard_NotImplemented",
            "FEmTool_ProfileMatrix::Prepare",
        ))
    }

    fn solve_iterative(
        &self,
        _: &Vector<'_>,
        _: &Vector<'_>,
        _: &mut Vector<'_>,
        _: &mut Vector<'_>,
        _: f64,
        _: i32,
    ) -> Result<(), Failure> {
        Err(failure(
            "Standard_NotImplemented",
            "FEmTool_ProfileMatrix::Solve",
        ))
    }

    fn multiplied(&self, x: &Vector<'_>, mx: &mut Vector<'_>) -> Result<(), Failure> {
        solve::multiplied(&self.layout, &self.values.read().matrix, x, mx)
    }

    fn row_number(&self) -> i32 {
        self.layout.rows()
    }

    fn col_number(&self) -> i32 {
        self.layout.rows()
    }
}
