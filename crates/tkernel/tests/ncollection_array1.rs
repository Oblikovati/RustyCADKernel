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

//! Faithful translations of NCollection_Array1_Test.cxx, grouped for review.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;

#[test]
fn NCollection_Array1Test__DefaultConstructor() {
    let array = Array1::<i32>::default();
    assert_eq!(0, array.len());
    assert!(array.is_empty());
    assert_eq!(1, array.lower());
    assert_eq!(0, array.upper());
}

include!("ncollection_array1/constructors.rs");
include!("ncollection_array1/assignment.rs");
include!("ncollection_array1/access.rs");
include!("ncollection_array1/size.rs");
include!("ncollection_array1/algorithms.rs");
