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

//! Byte-order primitives from FSD_BinaryFile; storage-driver I/O remains pending.

/// Binary persistence byte-order operations, independent of host endianness.
pub struct BinaryFile;

impl BinaryFile {
    /// Reverse every byte of a signed integer, including sign/payload bits.
    /// Example: `BinaryFile::inverse_int(0 as i32)` returns zero.
    pub const fn inverse_int(value: i32) -> i32 {
        value.swap_bytes()
    }

    /// Reverse every byte of a UTF-16 code unit, including sign/payload bits.
    /// Example: `BinaryFile::inverse_ext_char(0 as u16)` returns zero.
    pub const fn inverse_ext_char(value: u16) -> u16 {
        value.swap_bytes()
    }

    /// Reverse every byte of a unsigned 64-bit integer, including sign/payload bits.
    /// Example: `BinaryFile::inverse_uint64(0 as u64)` returns zero.
    pub const fn inverse_uint64(value: u64) -> u64 {
        value.swap_bytes()
    }

    /// Reverse every byte of a native-sized integer, including sign/payload bits.
    /// Example: `BinaryFile::inverse_size(0 as usize)` returns zero.
    pub const fn inverse_size(value: usize) -> usize {
        value.swap_bytes()
    }

    /// Reverse every byte of a binary64 representation, including sign/payload bits.
    /// Example: `BinaryFile::inverse_real(0 as f64)` returns zero.
    pub const fn inverse_real(value: f64) -> f64 {
        f64::from_bits(value.to_bits().swap_bytes())
    }

    /// Reverse every byte of a binary32 representation, including sign/payload bits.
    /// Example: `BinaryFile::inverse_short_real(0 as f32)` returns zero.
    pub const fn inverse_short_real(value: f32) -> f32 {
        f32::from_bits(value.to_bits().swap_bytes())
    }
}
