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

//! OCC24533 pointer observation for null and live handles.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::{Handle, StandardTransient};
#[test]
fn Standard_Handle_Test__OCC24533_IsNullAndPointerChecks() {
    let mut handle = Handle::<StandardTransient>::default();
    assert!(handle.is_null());
    let pointer = handle.as_ptr();
    assert!(pointer.is_null());
    assert_eq!(std::ptr::null(), pointer);
    handle = Handle::new(StandardTransient::default());
    assert!(!handle.is_null());
    let pointer = handle.as_ptr();
    assert!(!pointer.is_null());
    assert_ne!(std::ptr::null(), pointer);
}
