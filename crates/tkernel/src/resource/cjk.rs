// Created on: 2017-11-17
// Created by: Artem NOVIKOV
// Copyright (c) 2013-2017 OPEN CASCADE SAS
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

//! ConvertGBKToUnicode and ConvertBig5ToUnicode, state machines included.
use super::{multibyte::c_bytes, table_lookup};
use crate::{standard::Failure, tcollection::ExtendedString};

/// Big5 pointers the native code expands to a base letter plus a combining mark.
const BIG5_COMPOSED: [(u32, [u16; 2]); 4] = [
    (1133, [0x00CA, 0x0304]),
    (1135, [0x00CA, 0x030C]),
    (1164, [0x00EA, 0x0304]),
    (1166, [0x00EA, 0x030C]),
];
/// The native recovery `tostr.Insert(0, ...)`: position 0 is out of range, so every such
/// path raises Standard_OutOfRange with the text converted so far left in place.
fn insert_at_zero() -> Failure {
    Failure::out_of_range("TCollection_ExtendedString::Insert : Parameter theWhere is out of range")
}
fn is_digit(c: u32) -> bool {
    (0x30..=0x39).contains(&c)
}
/// Runs a decoder into `to`, which keeps the partial text when the decoder raises.
pub(super) fn decode_into(
    from: &[u8],
    to: &mut ExtendedString,
    decoder: fn(&[u8], &mut Vec<u16>) -> Result<bool, Failure>,
) -> Result<bool, Failure> {
    let mut units = Vec::new();
    let result = decoder(c_bytes(from), &mut units);
    *to = ExtendedString::from_raw(&units);
    result
}
/// Native GBK state machine over `(gb1, gb2, gb3)`.
///
/// Quirks preserved: a four-byte sequence indexes the two-byte table and succeeds only below
/// 23940; its state is never reset, so every following byte must be another digit that
/// completes the same prefix. Invalid trails of ASCII range raise (see `insert_at_zero`).
pub(super) fn gbk_units(bytes: &[u8], units: &mut Vec<u16>) -> Result<bool, Failure> {
    let (mut gb1, mut gb2, mut gb3) = (0u32, 0u32, 0u32);
    for &byte in bytes {
        let c = u32::from(byte);
        if gb3 != 0 {
            if !gbk_four_byte((gb1, gb2, gb3), c, units)? {
                return Ok(false);
            }
        } else if gb2 != 0 {
            if !(0x81..=0xFE).contains(&c) {
                return Err(insert_at_zero());
            }
            gb3 = c;
        } else if gb1 != 0 {
            if is_digit(c) {
                gb2 = c;
                continue;
            }
            let lead = std::mem::take(&mut gb1);
            if !gbk_pair(lead, c, units)? {
                return Ok(false);
            }
        } else if !gbk_single(c, &mut gb1, units) {
            return Ok(false);
        }
    }
    Ok(true)
}
/// The fourth byte of a four-byte sequence; false for the native `return false`.
fn gbk_four_byte(
    (gb1, gb2, gb3): (u32, u32, u32),
    c: u32,
    units: &mut Vec<u16>,
) -> Result<bool, Failure> {
    if !is_digit(c) {
        return Err(insert_at_zero());
    }
    let code = (gb1 - 0x81) * 12600 + (gb2 - 0x30) * 1260 + (gb3 - 0x81) * 10 + c - 0x30;
    if code >= 23940 {
        return Ok(false);
    }
    push_nonzero(units, table_lookup::gbkuni(code as usize));
    Ok(true)
}
/// The two-byte branch after a lead; false for the native `return false`.
fn gbk_pair(lead: u32, c: u32, units: &mut Vec<u16>) -> Result<bool, Failure> {
    let offset = if c < 0x7F { 0x40 } else { 0x41 };
    if (0x40..=0x7E).contains(&c) || (0x80..=0xFE).contains(&c) {
        let pointer = (lead - 0x81) * 190 + (c - offset);
        if pointer < 23940 {
            push_nonzero(units, table_lookup::gbkuni(pointer as usize));
            return Ok(true);
        }
    }
    if c <= 0x7F {
        return Err(insert_at_zero());
    }
    Ok(false)
}
/// A byte outside a sequence: ASCII, the euro sign 0x80, or a lead; false for 0xFF.
fn gbk_single(c: u32, gb1: &mut u32, units: &mut Vec<u16>) -> bool {
    match c {
        0..=0x7F => units.push(c as u16),
        0x80 => units.push(0x20AC),
        0x81..=0xFE => *gb1 = c,
        _ => return false,
    }
    true
}
/// `AssignCat(TCollection_ExtendedString(unit))`: a zero unit builds an empty string.
fn push_nonzero(units: &mut Vec<u16>, unit: u16) {
    if unit != 0 {
        units.push(unit);
    }
}
/// Native Big5 state machine with a single pending lead byte.
///
/// Quirks preserved: `Insert(Length()+1, unit)` appends table zeros verbatim, so unmapped
/// pairs leave U+0000 units inside the text; invalid ASCII-range trails raise.
pub(super) fn big5_units(bytes: &[u8], units: &mut Vec<u16>) -> Result<bool, Failure> {
    let mut pending = 0u32;
    for &byte in bytes {
        let c = u32::from(byte);
        if pending == 0 {
            match c {
                0..=0x7F => units.push(c as u16),
                0x81..=0xFE => pending = c,
                _ => return Ok(false),
            }
            continue;
        }
        let lead = std::mem::take(&mut pending);
        match big5_pair(lead, c, units) {
            Some(true) => continue,
            Some(false) => return Ok(false),
            None => {}
        }
        if c <= 0x7F {
            return Err(insert_at_zero());
        }
        return Ok(false);
    }
    Ok(true)
}
/// Appends the character of a lead/trail pair; `None` when the trail is not a Big5 trail,
/// `Some(false)` for the native `return false`.
///
/// Native quirk preserved: a table value above U+FFFF is put in a one-element `char32_t`
/// array without terminator and read as a NUL-terminated NCollection_UtfString, so the read
/// runs past the array and the UTF-16 copy is not exactly one surrogate pair (4 bytes).
/// The native code then returns false; OCCT never appends an astral Big5 character.
fn big5_pair(lead: u32, c: u32, units: &mut Vec<u16>) -> Option<bool> {
    if !(0x40..=0x7E).contains(&c) && !(0xA1..=0xFE).contains(&c) {
        return None;
    }
    let offset = if c < 0x7F { 0x40 } else { 0x62 };
    let pointer = (lead - 0x81) * 157 + (c - offset);
    if let Some((_, pair)) = BIG5_COMPOSED.iter().find(|(p, _)| *p == pointer) {
        units.extend_from_slice(pair);
        return Some(true);
    }
    // Every lead/trail pair maps below 19782, so the native `pointer < 19782` always holds.
    let Ok(unit) = u16::try_from(table_lookup::big5uni(pointer as usize)) else {
        return Some(false);
    };
    units.push(unit);
    Some(true)
}
