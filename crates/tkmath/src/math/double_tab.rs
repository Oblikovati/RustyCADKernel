// Created on: 1992-02-07
// Created by: Laurent PAINNOT
// Copyright (c) 1992-1999 Matra Datavision
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

//! math_DoubleTab's inline, owned, and external storage at pinned OCCT 3d097a0.
use rustycad_tkernel::ncollection::ArrayError;
mod movement;
mod storage;
use storage::{TableShape, TableStorage};

/// Bounded row-major doubles with OCCT's 64-value inline buffer; `DoubleTab::with_size(3,3)`.
pub struct DoubleTab<'a> {
    shape: TableShape,
    storage: TableStorage<'a>,
}
impl<'a> DoubleTab<'a> {
    /// Constructs zero-based dimensions; `DoubleTab::with_size(2,3)`.
    pub fn with_size(rows: usize, columns: usize) -> Self {
        let shape = TableShape::new([0, 0], [rows, columns]);
        Self {
            shape,
            storage: TableStorage::new(shape.len(), rows != 0),
        }
    }
    /// Constructs inclusive bounds; `DoubleTab::with_bounds(-1,1,1,3)?`.
    pub fn with_bounds(rl: i32, ru: i32, cl: i32, cu: i32) -> Result<Self, ArrayError> {
        let shape = TableShape::with_bounds(rl, ru, cl, cu)?;
        Ok(Self {
            shape,
            storage: TableStorage::new(shape.len(), true),
        })
    }
    /// Borrows the requested prefix, retaining external writes; `DoubleTab::from_buffer(&mut values,1,2,1,3)?`.
    pub fn from_buffer(
        buffer: &'a mut [f64],
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
    ) -> Result<Self, ArrayError> {
        let shape = TableShape::with_bounds(rl, ru, cl, cu)?;
        if buffer.len() < shape.len() {
            return Err(ArrayError::DimensionMismatch {
                expected: shape.len(),
                actual: buffer.len(),
            });
        }
        Ok(Self {
            shape,
            storage: TableStorage::Borrowed(&mut buffer[..shape.len()]),
        })
    }
    /// Initializes every cell; `table.init(1.0)`.
    pub fn init(&mut self, value: f64) {
        let length = self.shape.len();
        self.storage.slice_mut()[..length].fill(value);
    }
    /// Reports native buffer ownership; `!DoubleTab::with_size(8,8).is_deletable()`.
    pub fn is_deletable(&self) -> bool {
        matches!(&self.storage, TableStorage::Owned(values) if !values.is_empty())
    }
    /// Rebases rows without moving cells; `table.set_lower_row(-2)`.
    pub fn set_lower_row(&mut self, lower: i32) {
        self.shape.lower[0] = lower;
    }
    /// Rebases columns without moving cells; `table.set_lower_col(5)`.
    pub fn set_lower_col(&mut self, lower: i32) {
        self.shape.lower[1] = lower;
    }
    /// Returns the first declared row; `table.lower_row()`.
    pub fn lower_row(&self) -> i32 {
        self.shape.lower[0]
    }
    /// Returns the first declared column; `table.lower_col()`.
    pub fn lower_col(&self) -> i32 {
        self.shape.lower[1]
    }
    /// Returns the final declared row; `table.upper_row()`.
    pub fn upper_row(&self) -> i32 {
        self.shape.upper(0)
    }
    /// Returns the final declared column; `table.upper_col()`.
    pub fn upper_col(&self) -> i32 {
        self.shape.upper(1)
    }
    /// Counts rows without narrowing to native int; `table.row_size()`.
    pub fn row_size(&self) -> usize {
        self.shape.size[0]
    }
    /// Counts columns without narrowing to native int; `table.col_size()`.
    pub fn col_size(&self) -> usize {
        self.shape.size[1]
    }
    /// Returns the native signed row count; `table.nb_rows()`.
    pub fn nb_rows(&self) -> i32 {
        self.row_size() as i32
    }
    /// Returns the native signed column count; `table.nb_columns()`.
    pub fn nb_columns(&self) -> i32 {
        self.col_size() as i32
    }
    /// Reads zero-based offsets, independently of bounds; `table.at(0,0)?`.
    pub fn at(&self, row: usize, col: usize) -> Result<&f64, ArrayError> {
        Ok(&self.storage.slice()[self.shape.offset(row, col)?])
    }
    /// Mutates zero-based offsets; `*table.change_at(0,0)? = 2.0`.
    pub fn change_at(&mut self, row: usize, col: usize) -> Result<&mut f64, ArrayError> {
        let offset = self.shape.offset(row, col)?;
        Ok(&mut self.storage.slice_mut()[offset])
    }
    /// Reads declared indices; `table.value(1,1)?`.
    pub fn value(&self, row: i32, col: i32) -> Result<&f64, ArrayError> {
        let [r, c] = self.shape.indices(row, col)?;
        self.at(r, c)
    }
    /// Mutates declared indices; `*table.change_value(1,1)? = 2.0`.
    pub fn change_value(&mut self, row: i32, col: i32) -> Result<&mut f64, ArrayError> {
        let [r, c] = self.shape.indices(row, col)?;
        self.change_at(r, c)
    }
    /// Copies equal dimensions while preserving destination bounds; `source.copy_to(&mut destination)?`.
    pub fn copy_to(&self, destination: &mut DoubleTab<'_>) -> Result<(), ArrayError> {
        if self.shape.size != destination.shape.size {
            return Err(ArrayError::ShapeMismatch {
                expected: destination.shape.dimensions(),
                actual: self.shape.dimensions(),
            });
        }
        destination.storage.slice_mut()[..self.shape.len()].copy_from_slice(self.values());
        Ok(())
    }
    /// Copies bounds and values, retaining equal-length destination storage; `destination.assign(&source)`.
    pub fn assign(&mut self, source: &DoubleTab<'_>) {
        if self.shape.len() != source.shape.len() {
            self.storage = TableStorage::Owned(source.values().to_vec());
        } else {
            self.storage.slice_mut()[..source.shape.len()].copy_from_slice(source.values());
        }
        self.shape = source.shape;
    }
    fn values(&self) -> &[f64] {
        &self.storage.slice()[..self.shape.len()]
    }
    /// Borrows the row-major cells for native pointer-walking algorithms;
    /// `table.cells_mut()[0] = 1.0`.
    pub(crate) fn cells_mut(&mut self) -> &mut [f64] {
        let length: usize = self.shape.len();
        &mut self.storage.slice_mut()[..length]
    }
    /// Reads the row-major cells; `table.cells()[0]`.
    pub(crate) fn cells(&self) -> &[f64] {
        self.values()
    }
}
impl Clone for DoubleTab<'_> {
    fn clone(&self) -> Self {
        Self {
            shape: self.shape,
            storage: TableStorage::Owned(self.values().to_vec()),
        }
    }
}
impl std::ops::Index<(i32, i32)> for DoubleTab<'_> {
    type Output = f64;
    fn index(&self, (row, col): (i32, i32)) -> &f64 {
        self.value(row, col)
            .unwrap_or_else(|error| panic!("{error}"))
    }
}
impl std::ops::IndexMut<(i32, i32)> for DoubleTab<'_> {
    fn index_mut(&mut self, (row, col): (i32, i32)) -> &mut f64 {
        self.change_value(row, col)
            .unwrap_or_else(|error| panic!("{error}"))
    }
}
