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
// Rust translation of the NCollection_Array2<Extrema_POnSurfParams> members of
// Extrema_GenExtPS at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The grids of points on surface of Extrema_GenExtPS, with the storage rules of
//! NCollection_Array2: a resize without copy keeps the old cells when the flat size
//! does not change (only the bounds move), and an access outside the bounds raises
//! Standard_OutOfRange.
use crate::extrema::pon::POnSurfParams;
use crate::native_failure::failure;
use rustycad_tkernel::standard::Failure;

/// A row-major 2D array of points on surface with native bounds.
#[derive(Default)]
pub(super) struct ParamGrid {
    lower_row: i32,
    lower_col: i32,
    rows: usize,
    cols: usize,
    cells: Vec<POnSurfParams>,
}

impl ParamGrid {
    /// `Resize(theRowLower, theRowUpper, theColLower, theColUpper, false)`.
    pub fn resize(
        &mut self,
        (row_lower, row_upper): (i32, i32),
        (col_lower, col_upper): (i32, i32),
    ) -> Result<(), Failure> {
        if row_upper < row_lower || col_upper < col_lower {
            return Err(failure(
                "Standard_RangeError",
                &format!(
                    "NCollection_Array2::Resize - bounds ({row_lower}..{row_upper}, {col_lower}..{col_upper}); expected lower <= upper"
                ),
            ));
        }
        let rows: usize = (row_upper - row_lower + 1) as usize;
        let cols: usize = (col_upper - col_lower + 1) as usize;
        if rows * cols != self.cells.len() {
            self.cells = vec![POnSurfParams::default(); rows * cols];
        }
        (self.lower_row, self.lower_col, self.rows, self.cols) = (row_lower, col_lower, rows, cols);
        Ok(())
    }
    // The flat offset of `(row, col)`, Standard_OutOfRange outside the bounds.
    fn offset(&self, row: i32, col: i32) -> Result<usize, Failure> {
        let (dr, dc) = (
            i64::from(row) - i64::from(self.lower_row),
            i64::from(col) - i64::from(self.lower_col),
        );
        if dr < 0 || dc < 0 || dr as usize >= self.rows || dc as usize >= self.cols {
            return Err(failure(
                "Standard_OutOfRange",
                &format!(
                    "NCollection_Array2::Value - index ({row}, {col}) is outside ({}..{}, {}..{})",
                    self.lower_row,
                    self.lower_row + self.rows as i32 - 1,
                    self.lower_col,
                    self.lower_col + self.cols as i32 - 1
                ),
            ));
        }
        Ok(dr as usize * self.cols + dc as usize)
    }
    /// `Value(row, col)`.
    pub fn value(&self, row: i32, col: i32) -> Result<&POnSurfParams, Failure> {
        let offset: usize = self.offset(row, col)?;
        Ok(&self.cells[offset])
    }
    /// `ChangeValue(row, col)`.
    pub fn change_value(&mut self, row: i32, col: i32) -> Result<&mut POnSurfParams, Failure> {
        let offset: usize = self.offset(row, col)?;
        Ok(&mut self.cells[offset])
    }
    /// `SetValue(row, col, item)`.
    pub fn set_value(&mut self, row: i32, col: i32, item: POnSurfParams) -> Result<(), Failure> {
        *self.change_value(row, col)? = item;
        Ok(())
    }
    /// The square distance at `(row, col)`.
    pub fn sqr_distance(&self, row: i32, col: i32) -> Result<f64, Failure> {
        Ok(self.value(row, col)?.get_sqr_distance())
    }
}
