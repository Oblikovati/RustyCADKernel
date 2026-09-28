// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_CString.hxx/.cxx and
// Standard_CStringHasher.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Plain C string helpers: `IsEqual`, `Atof` and the `Standard_CStringHasher` policy.
//!
//! A C string is a byte slice read up to its first NUL (or its end), so embedded NULs
//! end the string exactly as `strlen`/`strcmp` do. `Strtod` is [`crate::standard::strtod`].

use super::hashing::{HashWord, fnv_hash, hash_bytes};
use crate::ncollection::HashPolicy;

/// The bytes `strlen` would count: everything before the first NUL.
fn c_str_prefix(text: &[u8]) -> &[u8] {
    let length: usize = text
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(text.len());
    &text[..length]
}

/// `IsEqual(const char*, const char*)`: `strcmp(a, b) == 0`, comparing up to the first NUL.
///
/// ```
/// assert!(rustycad_tkernel::standard::c_string::is_equal(b"abc\0x", b"abc"));
/// ```
pub fn is_equal(first: &[u8], second: &[u8]) -> bool {
    c_str_prefix(first) == c_str_prefix(second)
}

/// `Atof(const char*)`: `Strtod(text, nullptr)`, the C-locale numeric prefix (0 if none).
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::c_string::atof(" 2.5e1xyz"), 25.0);
/// ```
pub fn atof(text: &str) -> f64 {
    super::strtod(text).0
}

/// `Standard_CStringHasher`: the hash/equality policy for C-string keys.
///
/// Strings shorter than four bytes use FNV-1a over their bytes (so the empty string
/// hashes to the FNV offset basis); longer ones use `opencascade::hashBytes` (Murmur).
#[derive(Clone, Copy, Debug, Default)]
pub struct CStringHasher;

impl CStringHasher {
    /// `size_t operator()(const char*)`: the hash of the C string in `text`.
    ///
    /// ```
    /// use rustycad_tkernel::standard::c_string::CStringHasher;
    /// assert_eq!(CStringHasher.hash_code(b"abc\0tail"), CStringHasher.hash_code(b"abc"));
    /// ```
    pub fn hash_code(&self, text: &[u8]) -> usize {
        let text: &[u8] = c_str_prefix(text);
        if text.len() < 4 {
            return fnv_hash::hash_combine(text, text.len(), usize::FNV_SEED);
        }
        hash_bytes(text)
    }

    /// `bool operator()(const char*, const char*)`: `strcmp(a, b) == 0`.
    ///
    /// ```
    /// use rustycad_tkernel::standard::c_string::CStringHasher;
    /// assert!(CStringHasher.is_equal(b"abc", b"abc\0"));
    /// ```
    pub fn is_equal(&self, first: &[u8], second: &[u8]) -> bool {
        is_equal(first, second)
    }
}

impl<K: AsRef<[u8]>> HashPolicy<K> for CStringHasher {
    fn hash(&self, key: &K) -> usize {
        self.hash_code(key.as_ref())
    }
    fn equal(&self, first: &K, second: &K) -> bool {
        self.is_equal(first.as_ref(), second.as_ref())
    }
}
