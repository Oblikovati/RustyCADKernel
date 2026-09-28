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
// Rust translation of the hash expressions shared by ModelingData/TKG2d/Geom2dHash/*.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The three hash expressions every Geom2dHash hasher is built from, with the
//! native bit patterns (the values decide the iteration order of hashed containers).
use rustycad_tkernel::standard::hashing::hash_bytes;

// 2^63: the first double that no longer fits in an int64_t.
const TWO_POW_63: f64 = 9_223_372_036_854_775_808.0;

/// `static_cast<int64_t>(std::round(value))` on the pinned x86-64 profile.
///
/// The conversion compiles to `cvttsd2si`, which gives the "integer indefinite"
/// value `INT64_MIN` for NaN and for values out of range (C++ leaves them undefined;
/// Rust's `as` would saturate instead); `native_rounded_i64(f64::NAN) == i64::MIN`.
pub(crate) fn native_rounded_i64(value: f64) -> i64 {
    let rounded: f64 = value.round();
    if rounded.is_nan() || !(-TWO_POW_63..TWO_POW_63).contains(&rounded) {
        return i64::MIN;
    }
    rounded as i64
}

/// `opencascade::hash(static_cast<int64_t>(std::round(value * factor)))`, where
/// `factor = 1.0 / HashTolerance`; `rounded_hash(0.5, 1e7)`.
pub(crate) fn rounded_hash(value: f64, factor: f64) -> usize {
    hash_bytes(&native_rounded_i64(value * factor).to_ne_bytes())
}

/// `opencascade::hash(int)` over the four native bytes; `int_hash(3)`.
pub(crate) fn int_hash(value: i32) -> usize {
    hash_bytes(&value.to_ne_bytes())
}

/// `opencascade::hashBytes(aHashes, sizeof(aHashes))` of a `size_t` array;
/// `combined_hash(&[1, 2])`.
pub(crate) fn combined_hash(hashes: &[usize]) -> usize {
    let bytes: Vec<u8> = hashes.iter().flat_map(|hash| hash.to_ne_bytes()).collect();
    hash_bytes(&bytes)
}
