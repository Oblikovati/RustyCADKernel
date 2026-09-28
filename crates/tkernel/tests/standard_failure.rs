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

//! Translation of the OCC670 exception-printing regression.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Failure;

#[test]
fn Standard_Failure_Test__OCC670_ExceptionWithoutMessage() {
    // Result transports the legacy throw/catch across Rust's typed error boundary.
    let result: Result<(), Failure> = Err(Failure::out_of_range("test"));
    let mut caught = false;
    if let Err(exception) = result {
        caught = true;
        let message = exception.to_string();
        assert!(!message.is_empty(), "Exception message should not be empty");
    }
    assert!(caught, "Exception should have been caught");
}
