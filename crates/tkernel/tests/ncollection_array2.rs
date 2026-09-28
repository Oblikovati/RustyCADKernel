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

//! Translations of OCCT NCollection_Array2_Test.cxx.
// Preserve the upstream decimal test inputs exactly.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::ncollection::Array2;

fn filled(rows: (i32, i32), cols: (i32, i32), factor: i32) -> Array2<'static, i32> {
    let mut array = Array2::with_bounds(rows.0, rows.1, cols.0, cols.1).unwrap();
    for row in rows.0..=rows.1 {
        for col in cols.0..=cols.1 {
            array.set_value(row, col, row * factor + col).unwrap();
        }
    }
    array
}

fn shape(array: &Array2<'_, i32>, rows: usize, cols: usize, lower: (i32, i32)) {
    assert_eq!(array.len(), rows * cols);
    assert_eq!((array.nb_rows(), array.nb_columns()), (rows, cols));
    assert_eq!((array.lower_row(), array.lower_col()), lower);
    assert_eq!(
        (array.upper_row(), array.upper_col()),
        (lower.0 + rows as i32 - 1, lower.1 + cols as i32 - 1)
    );
}

include!("ncollection_array2/construction.rs");
include!("ncollection_array2/assignment.rs");
include!("ncollection_array2/resize.rs");
include!("ncollection_array2/access.rs");
