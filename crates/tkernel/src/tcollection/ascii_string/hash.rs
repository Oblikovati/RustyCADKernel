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
    /// `HashCode`: FNV-1a over signed `char` bytes below four bytes, Murmur otherwise, at the
    /// native pointer width; `AsciiString::new("abc").hash_code()`.
    pub fn hash_code(&self) -> usize {
        if self.len() < 4 {
            return short_hash(self.as_bytes());
        }
        crate::standard::hash_bytes(self.as_bytes())
    }
}
/// `FNVHash::hash_combine(*myString, myLength)`: OCCT feeds sign-extended `char` values.
fn short_hash(bytes: &[u8]) -> usize {
    let (mut hash, prime) = if usize::BITS == 64 {
        (14695981039346656037_u64, 1099511628211_u64)
    } else {
        (2166136261, 16777619)
    };
    for &byte in bytes {
        hash = (hash ^ (byte as i8 as i64 as u64)).wrapping_mul(prime);
    }
    hash as usize
}
