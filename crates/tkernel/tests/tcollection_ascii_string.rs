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

//! Original AsciiString declarations retain byte-level checks and one-based offsets.
#![allow(non_snake_case, clippy::approx_constant, useless_ptr_null_checks)]
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
fn text(actual: &AsciiString, expected: &str) {
    assert_eq!(actual.as_bytes(), expected.as_bytes());
}
include!("tcollection_ascii_string/basic.rs");
include!("tcollection_ascii_string/concatenation.rs");
include!("tcollection_ascii_string/insertion.rs");
include!("tcollection_ascii_string/mutation.rs");
include!("tcollection_ascii_string/comparison.rs");
include!("tcollection_ascii_string/views.rs");
include!("tcollection_ascii_string/regressions.rs");
include!("tcollection_ascii_string/useful_length.rs");
