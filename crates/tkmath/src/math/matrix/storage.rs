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

use super::{DoubleTab, Matrix};
use rustycad_tkernel::standard::Failure;
impl<'a> Matrix<'a> {
    /// Initializes every matrix cell; `matrix.init(0.0)`.
    pub fn init(&mut self, value: f64) {
        self.table.init(value);
    }
    /// Copies matching dimensions while retaining destination bounds; `matrix.initialized(&source)?`.
    pub fn initialized(&mut self, source: &Matrix<'_>) -> Result<(), Failure> {
        source.table.copy_to(&mut self.table).map_err(|_| {
            Failure::with_type(
                "Standard_DimensionMismatch",
                "NCollection_Array2::CopyValues",
                "",
            )
        })
    }
    /// Moves the underlying bounded table; `Matrix::move_construct(&mut source)`.
    pub fn move_construct(source: &mut Self) -> Self {
        Self {
            table: DoubleTab::move_construct(&mut source.table),
        }
    }
    /// Moves equal owned shapes or copies values preserving destination bounds; `matrix.move_assign(&mut source)?`.
    pub fn move_assign(&mut self, source: &mut Self) -> Result<(), Failure> {
        let same_bounds =
            (self.lower_row(), self.lower_col()) == (source.lower_row(), source.lower_col());
        let same_shape =
            (self.row_number(), self.col_number()) == (source.row_number(), source.col_number());
        if self.table.is_deletable() && source.table.is_deletable() && same_bounds && same_shape {
            self.table.move_assign(&mut source.table);
            return Ok(());
        }
        self.initialized(source)
    }
    pub(super) fn empty_like(&self) -> Self {
        Self::with_bounds(
            self.lower_row(),
            self.upper_row(),
            self.lower_col(),
            self.upper_col(),
        )
        .unwrap()
    }
}
