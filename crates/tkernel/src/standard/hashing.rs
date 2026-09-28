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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_HashUtils.hxx/.lxx and Standard_CStringHasher.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `opencascade::hashBytes` and `Standard_CStringHasher`: OCCT's native-width
//! MurmurHash/FNV hashing, whose values decide the iteration order of hashed containers.

// `MurmurHash::optimalSeed()` of each native size_t width.
const MURMUR_SEED_64: u64 = 0xA329_F1D3_A586;
const MURMUR_SEED_32: u32 = 0xc70f_6907;

/// `opencascade::hashBytes(theKey, theLen)` (and `opencascade::hash(theValue)` over the
/// value's native bytes): MurmurHash64A on 64-bit targets, MurmurHash2A otherwise;
/// `hash_bytes(&42_i32.to_ne_bytes())`.
pub fn hash_bytes(bytes: &[u8]) -> usize {
    if usize::BITS == 64 {
        murmur64(bytes, MURMUR_SEED_64) as usize
    } else {
        murmur32(bytes, MURMUR_SEED_32) as usize
    }
}

/// `Standard_CStringHasher::operator()(const char*)`: FNV-1a over signed chars below four
/// bytes, [`hash_bytes`] otherwise; `c_string_hash(b"Geom2d_Line")`.
pub fn c_string_hash(text: &[u8]) -> usize {
    if text.len() < 4 {
        return short_fnv_hash(text);
    }
    hash_bytes(text)
}

// FNVHash64A/FNVHash1A with the optimal seeds; `char` is signed on the pinned profile,
// so each byte is sign-extended before the xor.
fn short_fnv_hash(bytes: &[u8]) -> usize {
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

fn mix64(value: u64) -> u64 {
    value ^ (value >> 47)
}
pub(crate) fn murmur64(bytes: &[u8], seed: u64) -> u64 {
    const MUL: u64 = 0xc6a4_a793_5bd1_e995;
    let mut hash = seed ^ (bytes.len() as u64).wrapping_mul(MUL);
    let (chunks, remainder) = bytes.as_chunks::<8>();
    for chunk in chunks {
        let value = u64::from_ne_bytes(*chunk).wrapping_mul(MUL);
        hash = (hash ^ mix64(value).wrapping_mul(MUL)).wrapping_mul(MUL);
    }
    if !remainder.is_empty() {
        let mut tail = [0; 8];
        tail[..remainder.len()].copy_from_slice(remainder);
        hash = (hash ^ u64::from_le_bytes(tail)).wrapping_mul(MUL);
    }
    mix64(mix64(hash).wrapping_mul(MUL))
}
pub(crate) fn murmur32(bytes: &[u8], seed: u32) -> u32 {
    const MUL: u32 = 0x5bd1_e995;
    let mut hash = seed ^ bytes.len() as u32;
    let (chunks, remainder) = bytes.as_chunks::<4>();
    for chunk in chunks {
        let mut key = u32::from_ne_bytes(*chunk).wrapping_mul(MUL);
        key ^= key >> 24;
        key = key.wrapping_mul(MUL);
        hash = hash.wrapping_mul(MUL) ^ key;
    }
    if !remainder.is_empty() {
        let mut tail = [0; 4];
        tail[..remainder.len()].copy_from_slice(remainder);
        hash = (hash ^ u32::from_le_bytes(tail)).wrapping_mul(MUL);
    }
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(MUL);
    hash ^ (hash >> 15)
}
