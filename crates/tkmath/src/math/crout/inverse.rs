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

use super::{Crout, Matrix, Vector};
pub(super) fn invert_lower(lower: &mut Matrix<'_>) {
    // Native writes L(1, 1) unconditionally; for an empty matrix that cell is the first
    // slot of L's unused 64-value stack buffer, which nothing reads again (Done is true,
    // the inverse is empty), so skipping it is the only observable-equivalent port.
    if lower.row_number() == 0 {
        return;
    }
    lower[(1, 1)] = 1.0 / lower[(1, 1)];
    for row in 2..=lower.row_number() {
        invert_row(lower, row);
        lower[(row, row)] = 1.0 / lower[(row, row)];
    }
}
fn invert_row(lower: &mut Matrix<'_>, row: i32) {
    for k in 1..row {
        let mut scale = 0.0;
        for j in k..row {
            scale += lower[(row, j)] * lower[(j, k)];
        }
        lower[(row, k)] = -scale / lower[(row, row)];
    }
}
impl Crout {
    pub(super) fn build_inverse(&mut self, lower: &Matrix<'_>, diagonal: &Vector<'_>) {
        for col in 1..=lower.row_number() {
            self.inverse[(col, col)] = inverse_diagonal(lower, diagonal, col);
            for row in col + 1..=lower.row_number() {
                self.inverse[(row, col)] = inverse_lower_entry(lower, diagonal, row, col);
            }
        }
    }
}
fn inverse_diagonal(lower: &Matrix<'_>, diagonal: &Vector<'_>, col: i32) -> f64 {
    let mut scale = lower[(col, col)] * lower[(col, col)] / diagonal[col];
    for k in col + 1..=lower.row_number() {
        scale += lower[(k, col)] * lower[(k, col)] / diagonal[k];
    }
    scale
}
fn inverse_lower_entry(lower: &Matrix<'_>, diagonal: &Vector<'_>, row: i32, col: i32) -> f64 {
    let mut scale = lower[(row, col)] * lower[(row, row)] / diagonal[row];
    for k in row + 1..=lower.row_number() {
        scale += lower[(k, col)] * lower[(k, row)] / diagonal[k];
    }
    scale
}
