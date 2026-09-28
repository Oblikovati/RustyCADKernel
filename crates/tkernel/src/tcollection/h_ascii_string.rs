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

//! Object form of AsciiString, suitable for shared handles.
use super::AsciiString;
/// A string object with owned byte storage; `Arc::new(HAsciiString::default())`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HAsciiString {
    value: AsciiString,
}
impl HAsciiString {
    /// Borrows the object's terminated string; `object.as_bytes_with_nul()`.
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        self.value.as_bytes_with_nul()
    }
}
