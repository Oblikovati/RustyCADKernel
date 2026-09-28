// Copyright (c) 2013-2014 OPEN CASCADE SAS
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

//! Code-unit dispatch replacing the native `CharTypeChooser` size switch.
use super::encode;
use std::fmt::Debug;

/// The wide character unit: `wchar_t` is UTF-32 on Linux/macOS and UTF-16 on Windows.
#[cfg(not(windows))]
pub type WideChar = u32;
/// The wide character unit: `wchar_t` is UTF-32 on Linux/macOS and UTF-16 on Windows.
#[cfg(windows)]
pub type WideChar = u16;

/// A Unicode code unit (`u8` UTF-8, `u16` UTF-16, `u32` UTF-32), chosen by size as
/// `CharTypeChooser` does; implemented only for those three types.
///
/// ```
/// use rustycad_tkernel::ncollection::UtfUnit;
/// assert_eq!(<u16 as UtfUnit>::advance_bytes(0x1F600), 4);
/// ```
pub trait UtfUnit: Copy + Default + Eq + Debug + Into<u32> + 'static {
    /// Decodes the symbol at `position`; returns it and the next symbol's position.
    fn read_symbol(buffer: &[Self], position: usize) -> (u32, usize);
    /// Bytes needed to store `symbol` in this encoding (`AdvanceBytesUtf<Self>`).
    fn advance_bytes(symbol: u32) -> usize;
    /// Encodes `symbol` into `out`, which holds exactly `advance_bytes / size_of::<Self>()` units.
    fn write_symbol(symbol: u32, out: &mut [Self]);
    /// Reinterprets a raw 32-bit value as this unit, truncating like a native store.
    fn from_bits(value: u32) -> Self;
}

impl UtfUnit for u8 {
    fn read_symbol(buffer: &[Self], position: usize) -> (u32, usize) {
        encode::read_utf8(buffer, position)
    }
    fn advance_bytes(symbol: u32) -> usize {
        encode::utf8_length(symbol)
    }
    fn write_symbol(symbol: u32, out: &mut [Self]) {
        encode::write_utf8(symbol, out);
    }
    fn from_bits(value: u32) -> Self {
        value as u8
    }
}

impl UtfUnit for u16 {
    fn read_symbol(buffer: &[Self], position: usize) -> (u32, usize) {
        encode::read_utf16(buffer, position)
    }
    fn advance_bytes(symbol: u32) -> usize {
        encode::utf16_length(symbol) * 2
    }
    fn write_symbol(symbol: u32, out: &mut [Self]) {
        encode::write_utf16(symbol, out);
    }
    fn from_bits(value: u32) -> Self {
        value as u16
    }
}

impl UtfUnit for u32 {
    fn read_symbol(buffer: &[Self], position: usize) -> (u32, usize) {
        (encode::unit_at(buffer, position), position + 1)
    }
    // UTF-32 always takes one unit, even for surrogates or values above U+10FFFF.
    fn advance_bytes(_symbol: u32) -> usize {
        4
    }
    fn write_symbol(symbol: u32, out: &mut [Self]) {
        out[0] = symbol;
    }
    fn from_bits(value: u32) -> Self {
        value
    }
}
