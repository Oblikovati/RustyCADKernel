// Copyright (c) 2019 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_Dump.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_Dump::splitKeyToValue`: one `"key": value` pair.
use super::{DumpValue, DumpValues, json_key, next_close_position};
use crate::standard::Failure;
use crate::standard::dump::keys::JsonKey;
use crate::standard::dump::text::{length, sub_string};
use crate::tcollection::AsciiString;

/// Splits the pair whose key starts at `start` (just after its opening quote), stores it
/// and sets `next` past the value; false when the text has no closing quote, no key
/// separator or an unexpected value key.
pub(super) fn split_key_to_value(
    text: &[u8],
    start: i32,
    next: &mut i32,
    values: &mut DumpValues,
) -> Result<bool, Failure> {
    let close = next_close_position(text, start + 1, JsonKey::None, JsonKey::Quote)?;
    if close == 0 {
        return Ok(false);
    }
    let key = sub_string(text, start, close - 1)?;
    // Any key is accepted between the key and the value, not only ": ".
    let Some((_, after_separator)) = json_key(text, close + 1)? else {
        return Ok(false);
    };
    let (value_key, value_start) =
        json_key(text, after_separator)?.unwrap_or((JsonKey::None, after_separator));
    let Some((value, after)) = split_value(text, value_key, value_start)? else {
        return Ok(false);
    };
    *next = after;
    let key = unique_key(AsciiString::from_bytes(key), values);
    values.add(
        key,
        DumpValue::new(AsciiString::from_bytes(value), value_start),
    );
    Ok(true)
}

/// The value text and the position after it, by the key the value starts with.
fn split_value(text: &[u8], key: JsonKey, start: i32) -> Result<Option<(&[u8], i32)>, Failure> {
    let split = match key {
        JsonKey::OpenChild => {
            let close = next_close_position(text, start, JsonKey::OpenChild, JsonKey::CloseChild)?;
            (
                optional_range(text, start, close, close > start)?,
                close + 1,
            )
        }
        JsonKey::OpenContainer => {
            let close =
                next_close_position(text, start, JsonKey::OpenContainer, JsonKey::CloseContainer)?;
            (
                optional_range(text, start, close - 1, close > start)?,
                close + 1,
            )
        }
        JsonKey::Quote => quoted_value(text, start)?,
        JsonKey::None => plain_value(text, start)?,
        _ => return Ok(None),
    };
    Ok(Some(split))
}

fn optional_range(text: &[u8], from: i32, to: i32, present: bool) -> Result<&[u8], Failure> {
    if present {
        sub_string(text, from, to)
    } else {
        Ok(&[])
    }
}

/// A quoted value: `""` is empty, otherwise the text up to the next quote after `start`.
fn quoted_value(text: &[u8], start: i32) -> Result<(&[u8], i32), Failure> {
    if let Some((JsonKey::Quote, after)) = json_key(text, start)? {
        return Ok((&[], after));
    }
    let close = next_close_position(text, start + 1, JsonKey::None, JsonKey::Quote)?;
    Ok((sub_string(text, start, close - 1)?, close + 1))
}

/// An unquoted value: up to the byte before the next `}` or `, `; the last byte of the text
/// when the value starts there.
fn plain_value(text: &[u8], start: i32) -> Result<(&[u8], i32), Failure> {
    if start == length(text) {
        return Ok((sub_string(text, start, start)?, start + 1));
    }
    let child = next_close_position(text, start, JsonKey::None, JsonKey::CloseChild)? - 1;
    let separator =
        next_close_position(text, start, JsonKey::None, JsonKey::SeparatorValueToValue)? - 1;
    let close = child.min(separator);
    Ok((
        optional_range(text, start, close, start <= close)?,
        close + 1,
    ))
}

/// Appends `_1`, `_2`, ... until the key is new in `values`.
fn unique_key(key: AsciiString, values: &DumpValues) -> AsciiString {
    if !values.contains(&key) {
        return key;
    }
    let mut index = 1;
    loop {
        let candidate = key.cat(format!("_{index}").as_str());
        if !values.contains(&candidate) {
            return candidate;
        }
        index += 1;
    }
}
