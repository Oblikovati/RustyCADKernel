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

//! UTF-16 legacy assertions, including aliasing and invalid pointer/length adapters.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::{
    ncollection::DataMap,
    tcollection::{AsciiString, ExtendedString},
};
fn text(string: &ExtendedString, expected: &str) {
    assert_eq!(
        AsciiString::from_extended(string, None).as_bytes(),
        expected.as_bytes()
    );
}
fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn contains(string: &ExtendedString, pattern: &str) -> bool {
    AsciiString::from_extended(string, None)
        .as_bytes()
        .windows(pattern.len())
        .any(|part| part == pattern.as_bytes())
}
include!("tcollection_extended_string/basic.rs");
include!("tcollection_extended_string/conversion.rs");
include!("tcollection_extended_string/adjustment.rs");
include!("tcollection_extended_string/queries.rs");
include!("tcollection_extended_string/boundaries.rs");
include!("tcollection_extended_string/aliasing.rs");
