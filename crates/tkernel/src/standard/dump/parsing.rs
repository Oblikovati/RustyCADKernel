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

use crate::{standard::Failure, tcollection::AsciiString};
/// Consumes the exact quoted key and optional comma-space; `process_stream_name(text,"gp_Pnt",&mut position)?`.
pub fn process_stream_name(text: &str, name: &str, position: &mut i32) -> Result<bool, Failure> {
    if text.is_empty() || (text.len() as i64) < i64::from(*position) {
        return Ok(false);
    }
    let mut suffix = tail(text, *position)?;
    if suffix.starts_with(b", ") {
        *position += 2;
        suffix = tail(text, *position)?;
    }
    let key = format!("\"{name}\": ");
    let matches = suffix.starts_with(key.as_bytes());
    if matches {
        *position += key.len() as i32;
    }
    Ok(matches)
}
fn tail(text: &str, position: i32) -> Result<&[u8], Failure> {
    if position < 1 || position as usize > text.len() {
        return Err(Failure::out_of_range(
            "TCollection_AsciiString::SubString() out of range",
        ));
    }
    Ok(&text.as_bytes()[position as usize - 1..])
}
/// Reads bracketed numbers, mutating each output as it is parsed; `init_real_values(text,&mut position,&mut xyz)?`.
pub fn init_real_values(
    text: &str,
    position: &mut i32,
    values: &mut [f64],
) -> Result<bool, Failure> {
    let content = tail(text, *position + 1)?;
    let close = content.iter().position(|byte| *byte == b']');
    let mut start = 0;
    for index in 0..values.len() {
        let end = if index + 1 == values.len() {
            close.ok_or_else(substring_error)?
        } else {
            find_separator(content, start)?
        };
        if end <= start {
            return Err(substring_error());
        }
        let token = AsciiString::from_bytes(content.get(start..end).ok_or_else(|| {
            Failure::out_of_range("TCollection_AsciiString::SubString() out of range")
        })?);
        if !token.is_real_value(false) {
            return Ok(false);
        }
        values[index] = token.real_value()?;
        start = end + 2;
    }
    *position = tail(text, *position)?
        .iter()
        .position(|byte| *byte == b']')
        .map_or(1, |index| *position + index as i32 + 1);
    Ok(true)
}
fn find_separator(content: &[u8], start: usize) -> Result<usize, Failure> {
    content
        .get(start..)
        .and_then(|tail| tail.windows(2).position(|bytes| bytes == b", "))
        .map(|index| index + start)
        .ok_or_else(|| Failure::out_of_range("TCollection_AsciiString::SubString() out of range"))
}
/// Reads a scalar fragment and consumes its delimiter; `init_value(text,&mut position)?`.
pub fn init_value(text: &str, position: &mut i32) -> Result<AsciiString, Failure> {
    let content = tail(text, *position)?;
    let comma = content.windows(2).position(|bytes| bytes == b", ");
    let close = content.iter().position(|byte| *byte == b'}');
    let delimiter = match (comma, close) {
        (Some(a), Some(b)) if b < a => Some((b, 1)),
        (Some(a), _) => Some((a, 2)),
        (None, Some(b)) => Some((b, 1)),
        _ => None,
    };
    let end = delimiter.map_or(content.len(), |(index, _)| index);
    if end == 0 {
        return Err(substring_error());
    }
    let value = AsciiString::from_bytes(&content[..end]);
    *position = delimiter.map_or(text.len() as i32, |(index, length)| {
        *position + (index + length) as i32
    });
    Ok(value)
}
/// Initializes a named vector while committing the cursor only on success; `init_vector(text,"Location",&mut p,&mut xyz)?`.
pub fn init_vector(
    text: &str,
    name: &str,
    position: &mut i32,
    values: &mut [f64],
) -> Result<bool, Failure> {
    let mut local = *position;
    if !process_stream_name(text, name, &mut local)? || !init_real_values(text, &mut local, values)?
    {
        return Ok(false);
    }
    *position = local;
    Ok(true)
}
/// Initializes a named real field; `init_real_field(text,"Gap",&mut p,&mut gap)?`.
pub fn init_real_field(
    text: &str,
    name: &str,
    position: &mut i32,
    value: &mut f64,
) -> Result<bool, Failure> {
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
/// Initializes a named integer field; `init_integer_field(text,"Flags",&mut p,&mut flags)?`.
pub fn init_integer_field(
    text: &str,
    name: &str,
    position: &mut i32,
    value: &mut i32,
) -> Result<bool, Failure> {
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

fn substring_error() -> Failure {
    Failure::out_of_range("TCollection_AsciiString::SubString() out of range")
}
