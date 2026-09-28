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

impl<T: Default + Clone> Array2<'_, T> {
    /// Resizes using OCCT's packed-copy rule; `array.resize_bounds(1,3,1,4,true)?`.
    pub fn resize_bounds(
        &mut self,
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        copy: bool,
    ) -> Result<(), ArrayError> {
        self.reshape(rl, ru, cl, cu, copy, false)
    }
    /// Resizes preserving common row/column positions; `array.resize_trim_bounds(1,3,1,4,true)?`.
    pub fn resize_trim_bounds(
        &mut self,
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        copy: bool,
    ) -> Result<(), ArrayError> {
        self.reshape(rl, ru, cl, cu, copy, true)
    }
    /// Resizes counts while retaining origins; `array.resize(3,4,true)?`.
    pub fn resize(&mut self, rows: usize, cols: usize, copy: bool) -> Result<(), ArrayError> {
        self.resize_counts(rows, cols, copy, false)
    }
    /// Resizes counts preserving the common rectangle; `array.resize_with_trim(3,4,true)?`.
    pub fn resize_with_trim(
        &mut self,
        rows: usize,
        cols: usize,
        copy: bool,
    ) -> Result<(), ArrayError> {
        self.resize_counts(rows, cols, copy, true)
    }
    fn resize_counts(
        &mut self,
        rows: usize,
        cols: usize,
        copy: bool,
        trim: bool,
    ) -> Result<(), ArrayError> {
        if rows == 0 || cols == 0 {
            self.elements.resize(0, false);
            self.rows = rows;
            self.cols = cols;
            return Ok(());
        }
        self.reshape(
            self.row_lower,
            upper(self.row_lower, rows),
            self.col_lower,
            upper(self.col_lower, cols),
            copy,
            trim,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn reshape(
        &mut self,
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
        copy: bool,
        trim: bool,
    ) -> Result<(), ArrayError> {
        let (rows, cols) = (dimension(rl, ru)?, dimension(cl, cu)?);
        if !copy || (rows, cols) == (self.rows, self.cols) || self.is_empty() {
            self.elements.resize(rows * cols, false);
            self.rows = rows;
            self.cols = cols;
            self.rebase(rl, cl);
            return Ok(());
        }
        let mut replacement = Self::with_bounds(rl, ru, cl, cu)?;
        self.copy_resized_region(&mut replacement, trim);
        *self = replacement;
        Ok(())
    }
    fn copy_resized_region(&self, replacement: &mut Self, trim: bool) {
        let columns = self.cols.min(replacement.cols);
        let stride = if trim { self.cols } else { columns };
        for row in 0..self.rows.min(replacement.rows) {
            let dest = row * replacement.cols;
            let source = row * stride;
            replacement.as_mut_slice()[dest..dest + columns]
                .clone_from_slice(&self.as_slice()[source..source + columns]);
        }
    }
}
