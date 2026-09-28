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

//! The single-byte code-page branches (CP125x, ISO 8859-x, CP850) of Resource_Unicode.
use super::{
    multibyte::{c_bytes, put},
    table_lookup,
};
use crate::{standard::Failure, tcollection::ExtendedString};

/// Bytes below 0x80 are ASCII; others read the code page, whose unused slots decode to '?'.
pub(super) fn decode(row: usize, from: &[u8]) -> ExtendedString {
    let units: Vec<u16> = c_bytes(from)
        .iter()
        .map(|&byte| match byte & 0x80 {
            0 => u16::from(byte),
            _ => match table_lookup::code_page(row, usize::from(byte & 0x7f)) {
                0 => u16::from(b'?'),
                unit => unit,
            },
        })
        .collect();
    ExtendedString::from_raw(&units)
}
/// The native encoding loop, quirks included.
///
/// * A buffer shorter than the text reports false without writing.
/// * The loop always fills `maxsize - 1` bytes, reading `Value(i + 1)` of the source; a
///   buffer longer than the text plus one therefore raises Standard_OutOfRange from
///   `TCollection_ExtendedString::Value` (after the bytes before it were written).
/// * Only the 128 code-page slots are searched: ASCII characters are absent from them and
///   become '?'. When a character occurs twice the last slot wins.
pub(super) fn encode(row: usize, from: &ExtendedString, to: &mut [u8]) -> Result<bool, Failure> {
    let units = from.as_utf16();
    let max_size = to.len() as i32;
    if max_size < units.len() as i32 {
        return Ok(false);
    }
    for index in 0..(max_size - 1).max(0) {
        let Some(&unit) = units.get(index as usize) else {
            return Err(Failure::out_of_range(
                "TCollection_ExtendedString::Value : parameter theWhere",
            ));
        };
        put(to, index, encode_unit(row, unit))?;
    }
    put(to, max_size - 1, 0)?;
    Ok(true)
}
fn encode_unit(row: usize, unit: u16) -> u8 {
    if unit == 0 {
        // Unused code-page slots hold zero, so zero must not be searched for.
        return 0;
    }
    // The native loop keeps overwriting, so the last matching slot wins.
    (0..128u8)
        .rev()
        .find(|&slot| table_lookup::code_page(row, usize::from(slot)) == unit)
        .map_or(b'?', |slot| slot | 0x80)
}
