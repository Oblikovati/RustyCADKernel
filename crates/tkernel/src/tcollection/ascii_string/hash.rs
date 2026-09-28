// Copyright (c) 2023 OPEN CASCADE SAS
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

use super::AsciiString;
impl AsciiString {
    /// Computes OCCT's native-width hash with signed-char short-input compatibility
    /// (the Standard_CStringHasher algorithm); `text.hash_code()`.
    pub fn hash_code(&self) -> usize {
        crate::standard::hashing::c_string_hash(self.as_bytes())
    }
}
