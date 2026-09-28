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
use crate::standard::hashing::{HashWord, fnv_hash, hash_bytes};
impl AsciiString {
    /// `HashCode`: FNV-1a over signed `char` bytes below four bytes, Murmur otherwise, at the
    /// native pointer width; `AsciiString::new("abc").hash_code()`.
    pub fn hash_code(&self) -> usize {
        let bytes: &[u8] = self.as_bytes();
        if bytes.len() < 4 {
            // `FNVHash::hash_combine(*myString, myLength)`: the same engine as the C-string hasher.
            return fnv_hash::hash_combine(bytes, bytes.len(), usize::FNV_SEED);
        }
        hash_bytes(bytes)
    }
}
