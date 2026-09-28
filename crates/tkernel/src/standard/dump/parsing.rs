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

//! Reading dump text back: stream names, field names, scalar values and real vectors.
//! Positions are one-based, as in `TCollection_AsciiString`.
use super::keys::{JsonKey, json_key_length, json_key_to_string};
use super::text::{length, location, sub_string};
use super::values::dump_field_to_name;
use crate::{standard::Failure, tcollection::AsciiString};

/// `ProcessStreamName`: consumes an optional `, ` and the exact `"name": ` at `position`;
/// false (position untouched unless a `, ` was consumed) when the key differs, the text is
/// empty or `position` is past its end; `process_stream_name(text, "gp_Pnt", &mut position)?`.
pub fn process_stream_name(
    text: impl AsRef<[u8]>,
    name: impl AsRef<[u8]>,
    position: &mut i32,
) -> Result<bool, Failure> {
    let text = text.as_ref();
    if text.is_empty() || length(text) < *position {
        return Ok(false);
    }
    match_key(text, name.as_ref(), position)
}

/// `ProcessFieldName`: like [`process_stream_name`] for a C++ field expression, whose key is
/// `DumpFieldToName` of it (up to its first NUL); unlike it, a position past the end raises
/// Standard_OutOfRange; `process_field_name(text, "myGap", &mut position)?`.
pub fn process_field_name(
    text: impl AsRef<[u8]>,
    name: impl AsRef<[u8]>,
    position: &mut i32,
) -> Result<bool, Failure> {
    let text = text.as_ref();
    if text.is_empty() {
        return Ok(false);
    }
    let name = name
        .as_ref()
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default();
    match_key(text, dump_field_to_name(name)?.as_bytes(), position)
}

fn match_key(text: &[u8], name: &[u8], position: &mut i32) -> Result<bool, Failure> {
    let mut rest = sub_string(text, *position, length(text))?;
    let separator = json_key_to_string(JsonKey::SeparatorValueToValue).as_bytes();
    if rest.starts_with(separator) {
        *position += json_key_length(JsonKey::SeparatorValueToValue);
        rest = sub_string(text, *position, length(text))?;
    }
    let key = [b"\"", name, b"\": "].concat();
    let matches = rest.starts_with(&key);
    if matches {
        *position += key.len() as i32;
    }
    Ok(matches)
}

/// `InitRealValues`: parses `[v1, v2, ...]` after `position` into `values`, storing each as
/// it is read, then moves `position` past the text's first `]` from `position` (to 1 when
/// there is none); false at the first non-numeric value; `init_real_values(t, &mut p, &mut xyz)?`.
pub fn init_real_values(
    text: impl AsRef<[u8]>,
    position: &mut i32,
    values: &mut [f64],
) -> Result<bool, Failure> {
    let text = text.as_ref();
    let content = sub_string(
        text,
        *position + json_key_length(JsonKey::OpenContainer),
        length(text),
    )?;
    let close = json_key_to_string(JsonKey::CloseContainer).as_bytes();
    let separator = json_key_to_string(JsonKey::SeparatorValueToValue).as_bytes();
    let close_at = location(content, close, 1, length(content))?;
    let count = values.len();
    let mut start = 1;
    for (index, value) in values.iter_mut().enumerate() {
        let end = if index + 1 < count {
            location(content, separator, start, length(content))?
        } else {
            close_at
        };
        let token = AsciiString::from_bytes(sub_string(content, start, end - 1)?);
        if !token.is_real_value(false) {
            return Ok(false);
        }
        *value = token.real_value()?;
        start = end + json_key_length(JsonKey::SeparatorValueToValue);
    }
    *position =
        location(text, close, *position, length(text))? + json_key_length(JsonKey::CloseContainer);
    Ok(true)
}

/// `InitValue`: the scalar text at `position`, up to the first `, ` or an earlier `}` (the
/// whole rest when neither follows); `position` moves past that delimiter, or to the text
/// length; `init_value(text, &mut position)?`.
pub fn init_value(text: impl AsRef<[u8]>, position: &mut i32) -> Result<AsciiString, Failure> {
    let text = text.as_ref();
    let rest = sub_string(text, *position, length(text))?;
    let comma = location(
        rest,
        json_key_to_string(JsonKey::SeparatorValueToValue).as_bytes(),
        1,
        length(rest),
    )?;
    let brace = location(
        rest,
        json_key_to_string(JsonKey::CloseChild).as_bytes(),
        1,
        length(rest),
    )?;
    let (end, key) = if (comma > 0 && brace > 0 && brace < comma) || comma == 0 {
        (brace, JsonKey::CloseChild)
    } else {
        (comma, JsonKey::SeparatorValueToValue)
    };
    if end == 0 {
        *position = length(text);
        return Ok(AsciiString::from_bytes(rest));
    }
    let value = AsciiString::from_bytes(sub_string(rest, 1, end - 1)?);
    *position += end - 1 + json_key_length(key);
    Ok(value)
}

/// `OCCT_INIT_VECTOR_CLASS`: a named real vector, committing the position only on success;
/// `init_vector(text, "Location", &mut position, &mut xyz)?`.
pub fn init_vector(
    text: impl AsRef<[u8]>,
    name: &str,
    position: &mut i32,
    values: &mut [f64],
) -> Result<bool, Failure> {
    let text = text.as_ref();
    let mut local = *position;
    if !process_stream_name(text, name, &mut local)? || !init_real_values(text, &mut local, values)?
    {
        return Ok(false);
    }
    *position = local;
    Ok(true)
}

/// `OCCT_INIT_FIELD_VALUE_REAL` for an already converted key; `init_real_field(t, "Gap", &mut p, &mut gap)?`.
pub fn init_real_field(
    text: impl AsRef<[u8]>,
    name: &str,
    position: &mut i32,
    value: &mut f64,
) -> Result<bool, Failure> {
    let text = text.as_ref();
    let mut local = *position;
    if !process_stream_name(text, name, &mut local)? {
        return Ok(false);
    }
    let token = init_value(text, &mut local)?;
    if !token.is_real_value(false) {
        return Ok(false);
    }
    *value = token.real_value()?;
    *position = local;
    Ok(true)
}

/// `OCCT_INIT_FIELD_VALUE_INTEGER` for an already converted key; `init_integer_field(t, "Flags", &mut p, &mut f)?`.
pub fn init_integer_field(
    text: impl AsRef<[u8]>,
    name: &str,
    position: &mut i32,
    value: &mut i32,
) -> Result<bool, Failure> {
    let text = text.as_ref();
    let mut local = *position;
    if !process_stream_name(text, name, &mut local)? {
        return Ok(false);
    }
    let token = init_value(text, &mut local)?;
    if !token.is_integer_value() {
        return Ok(false);
    }
    *value = token.integer_value()?;
    *position = local;
    Ok(true)
}
