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

//! `opencascade::hashBytes` and `Standard_CStringHasher` against values computed by an
//! independent MurmurHash64A / FNV-1a implementation of Standard_HashUtils.lxx.
use rustycad_tkernel::standard::c_string::CStringHasher;
use rustycad_tkernel::standard::hashing::hash_bytes;
use rustycad_tkernel::tcollection::AsciiString;

#[test]
#[cfg(target_pointer_width = "64")]
fn hash_bytes_is_murmur64a_with_the_optimal_seed() {
    assert_eq!(hash_bytes(&[]), 1518816612617952436);
    // A four-byte int goes through the tail path only.
    assert_eq!(hash_bytes(&3_i32.to_le_bytes()), 1372453362253495625);
    // 24 bytes: three full 8-byte blocks.
    assert_eq!(
        hash_bytes(b"Geom2dEval_SineWaveCurve"),
        12281767022877799982
    );
}

#[test]
#[cfg(target_pointer_width = "64")]
fn short_c_strings_use_fnv_over_signed_chars() {
    assert_eq!(CStringHasher.hash_code(b"ab"), 620445648566982762);
    assert_eq!(
        CStringHasher.hash_code(b"Geom2d_Line"),
        hash_bytes(b"Geom2d_Line")
    );
}

#[test]
fn ascii_string_hash_is_the_c_string_hash() {
    for text in ["", "a", "abc", "abcd", "Geom2dEval_TBezierCurve"] {
        let ascii = AsciiString::from(text);
        assert_eq!(
            ascii.hash_code(),
            CStringHasher.hash_code(text.as_bytes()),
            "{text:?}"
        );
    }
}
