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
// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_ProfileMatrix.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The immutable profile description of `FEmTool_ProfileMatrix`: the `profile`
//! table (row heights and diagonal addresses) and `NextCoeff`.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// `profile(1, i)` (the number of stored entries left of the diagonal), `profile(2, i)`
/// (the 1-based address of the diagonal) and `NextCoeff(address)` (the next row
/// storing the column of `address`, 0 for none).
pub(super) struct ProfileLayout {
    heights: Vec<i32>,
    diagonals: Vec<i32>,
    next: Vec<i32>,
}

/// The 0-based slot of a 1-based `address` in storage of `size` values; the
/// addresses the native code reaches without a check (undefined out of range)
/// are Standard_OutOfRange here; `index(1, 4)? == 0`.
pub(super) fn index(address: i32, size: usize) -> Result<usize, Failure> {
    usize::try_from(address - 1)
        .ok()
        .filter(|slot| *slot < size)
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "FEmTool_ProfileMatrix: address {address} outside the profile storage 1..={size}"
            ))
        })
}

/// The value at 1-based `address`; `slot(&values, 1)?`.
pub(super) fn slot(values: &[f64], address: i32) -> Result<&f64, Failure> {
    Ok(&values[index(address, values.len())?])
}

/// The mutable value at 1-based `address`; `*slot_mut(&mut values, 1)? = 0.0`.
pub(super) fn slot_mut(values: &mut [f64], address: i32) -> Result<&mut f64, Failure> {
    let position: usize = index(address, values.len())?;
    Ok(&mut values[position])
}

// FirstIndexes(i), read from index 1 as natively (unchecked there).
fn first_index(first_indexes: &Array1<'_, i32>, i: i32) -> Result<i32, Failure> {
    first_indexes.value(i).copied().map_err(|error| {
        Failure::out_of_range(&format!(
            "FEmTool_ProfileMatrix: FirstIndexes({i}) ({error}); the native code reads indexes 1..=Length"
        ))
    })
}

impl ProfileLayout {
    /// The profile of `FirstIndexes`; `ProfileLayout::new(&first_indexes)?`.
    pub(super) fn new(first_indexes: &Array1<'_, i32>) -> Result<Self, Failure> {
        let rows: i32 = first_indexes.length();
        if rows < 1 {
            return Err(Failure::out_of_range(
                "FEmTool_ProfileMatrix: FirstIndexes is empty; expected at least one row",
            ));
        }
        let (mut heights, mut diagonals): (Vec<i32>, Vec<i32>) = (vec![0], vec![1]);
        for i in 2..=rows {
            let height: i32 = i - first_index(first_indexes, i)?;
            heights.push(height);
            diagonals.push(diagonals[diagonals.len() - 1] + height + 1);
        }
        let mut layout: Self = Self {
            heights,
            diagonals,
            next: Vec::new(),
        };
        layout.next = layout.next_coefficients(first_indexes)?;
        Ok(layout)
    }

    // NextCoeff: for each stored (i, j), the first row l > i with FirstIndexes(l) <= j.
    fn next_coefficients(&self, first_indexes: &Array1<'_, i32>) -> Result<Vec<i32>, Failure> {
        let (rows, size) = (self.rows(), self.size());
        let mut next: Vec<i32> = vec![0; size];
        let mut k: i32 = 1;
        for i in 1..=rows {
            for j in first_index(first_indexes, i)?..=i {
                let mut l: i32 = i + 1;
                while l <= rows && j < first_index(first_indexes, l)? {
                    l += 1;
                }
                next[index(k, size)?] = if l > rows { 0 } else { l };
                k += 1;
            }
        }
        Ok(next)
    }

    /// `RowNumber()`; `layout.rows()`.
    pub(super) fn rows(&self) -> i32 {
        self.heights.len() as i32
    }

    /// The number of stored coefficients, `profile(2, RowNumber())`; `layout.size()`.
    pub(super) fn size(&self) -> usize {
        usize::try_from(self.diagonals[self.diagonals.len() - 1]).unwrap_or(0)
    }

    /// `profile(1, i)`; `layout.height(2)?`.
    pub(super) fn height(&self, i: i32) -> Result<i32, Failure> {
        row_entry(&self.heights, i)
    }

    /// `profile(2, i)`; `layout.diagonal(2)?`.
    pub(super) fn diagonal(&self, i: i32) -> Result<i32, Failure> {
        row_entry(&self.diagonals, i)
    }

    /// `NextCoeff(address)`; `layout.next(1)?`.
    pub(super) fn next(&self, address: i32) -> Result<i32, Failure> {
        Ok(self.next[index(address, self.next.len())?])
    }
}

// Entry `i` (1-based) of a per-row table.
fn row_entry(table: &[i32], i: i32) -> Result<i32, Failure> {
    usize::try_from(i - 1)
        .ok()
        .and_then(|slot| table.get(slot).copied())
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "FEmTool_ProfileMatrix: row {i}; expected 1..={}",
                table.len()
            ))
        })
}
