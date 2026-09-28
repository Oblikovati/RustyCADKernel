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

//! OCC22595: the default matrix initializes every coefficient.
#![allow(non_snake_case)]
use rustycad_tkmath::gp::Matrix3;
#[test]
fn gp_MatTest__OCC22595_DefaultConstructor() {
    let matrix = Matrix3::default();
    for row in 1..=3 {
        for column in 1..=3 {
            assert_eq!(0.0, matrix.value(row, column).unwrap());
        }
    }
}
