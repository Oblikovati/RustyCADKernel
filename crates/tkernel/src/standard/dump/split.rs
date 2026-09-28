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

//! `Standard_Dump::SplitJson` and the key scanning it is built on.
use super::keys::{JsonKey, json_key_length, json_key_to_string};
use super::text::{length, location, search, sub_string};
use crate::ncollection::IndexedDataMap;
use crate::standard::Failure;
use crate::tcollection::AsciiString;
mod value;

/// A split value and the one-based position of its first byte in the whole text
/// (`Standard_DumpValue`); `DumpValue::new(AsciiString::new("1"), 9)`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DumpValue {
    /// The value text (`myValue`).
    pub value: AsciiString,
    /// Position of the value's first byte in the stream (`myStartPosition`).
    pub start_position: i32,
}

impl DumpValue {
    /// Pairs a value with its position; `DumpValue::new(AsciiString::new("x"), 3)`.
    pub fn new(value: AsciiString, start_position: i32) -> Self {
        Self {
            value,
            start_position,
        }
    }
}

/// Split values keyed by their JSON key, in text order.
pub type DumpValues = IndexedDataMap<AsciiString, DumpValue>;

/// `SplitJson`: adds every `"key": value` pair of one level of `text` to `values`, renaming
/// repeated keys `key_1`, `key_2`, ...; returns false on text it cannot split;
/// `split_json("\"a\": 1", &mut values)?`.
///
/// Native quirk kept: a `{` at the level being split always raises Standard_OutOfRange,
/// because the nested text is cut with `SubString(start + 1, start - 2)`.
pub fn split_json(text: impl AsRef<[u8]>, values: &mut DumpValues) -> Result<bool, Failure> {
    let text = text.as_ref();
    let mut next = 1;
    while next < length(text) {
        let Some((key, after)) = json_key(text, next)? else {
            return Ok(false);
        };
        next = after;
        let processed = match key {
            JsonKey::Quote => value::split_key_to_value(text, next, &mut next, values)?,
            JsonKey::OpenChild => split_child(text, &mut next, values)?,
            JsonKey::SeparatorValueToValue => continue,
            _ => false,
        };
        if !processed {
            return Ok(false);
        }
    }
    Ok(true)
}

fn split_child(text: &[u8], next: &mut i32, values: &mut DumpValues) -> Result<bool, Failure> {
    let start = *next;
    let close = next_close_position(text, start, JsonKey::OpenChild, JsonKey::CloseChild)?;
    if close == 0 {
        return Ok(false);
    }
    let child = sub_string(text, start + json_key_length(JsonKey::OpenChild), *next - 2)?;
    if !split_json(child, values)? {
        return Ok(false);
    }
    *next = close + json_key_length(JsonKey::CloseChild);
    Ok(true)
}

/// `HierarchicalValueIndices`: one-based indices of the values holding a `: ` pair (the
/// NCollection_List becomes a Vec in index order); `hierarchical_value_indices(&values)?`.
pub fn hierarchical_value_indices(
    values: &IndexedDataMap<AsciiString, AsciiString>,
) -> Result<Vec<i32>, Failure> {
    let mut indices = Vec::new();
    for index in 1..=values.extent() {
        if has_child_key(values.find_from_index(index)?) {
            indices.push(index as i32);
        }
    }
    Ok(indices)
}

/// `HasChildKey`: true when the value contains `: `; `has_child_key("{\"a\": 1}")`.
pub fn has_child_key(value: impl AsRef<[u8]>) -> bool {
    search(
        value.as_ref(),
        json_key_to_string(JsonKey::SeparatorKeyToValue).as_bytes(),
    ) >= 0
}

/// `jsonKey`: the key starting at `start` and the position after it, `None` when the
/// text there starts with no key; Standard_OutOfRange when `start` is past the end.
fn json_key(text: &[u8], start: i32) -> Result<Option<(JsonKey, i32)>, Failure> {
    let rest = sub_string(text, start, length(text))?;
    Ok(JsonKey::PARSED
        .into_iter()
        .find(|key| rest.starts_with(json_key_to_string(*key).as_bytes()))
        .map(|key| (key, start + json_key_length(key))))
}

/// `nextClosePosition`: the first `close` after `start` not matched by an `open` in
/// between, or the text length when there is none (0 only for an empty text).
fn next_close_position(
    text: &[u8],
    start: i32,
    open: JsonKey,
    close: JsonKey,
) -> Result<i32, Failure> {
    let (open, close) = (
        json_key_to_string(open).as_bytes(),
        json_key_to_string(close).as_bytes(),
    );
    let (mut position, mut depth) = (start, 0);
    while position < length(text) {
        let open_at = location(text, open, position, length(text))?;
        let close_at = location(text, close, position, length(text))?;
        if close_at == 0 {
            break;
        }
        if open_at != 0 && open_at <= close_at {
            depth += 1;
            position = open_at + 1;
        } else if depth == 0 {
            return Ok(close_at);
        } else {
            depth -= 1;
            position = close_at + 1;
        }
    }
    Ok(length(text))
}
