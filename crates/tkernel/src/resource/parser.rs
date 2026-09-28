// Copyright (c) 2026 OPEN CASCADE SAS
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

use crate::tcollection::AsciiString;
fn horizontal_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t')
}
pub(super) fn resource_line(line: &[u8]) -> Option<(AsciiString, AsciiString)> {
    if line.first() == Some(&b'!') {
        return None;
    }
    // Native whitespace is only space/tab: CR and other ASCII controls remain significant.
    let start = line.iter().position(|byte| !horizontal_space(*byte))?;
    let content = &line[start..];
    let colon = content.iter().position(|byte| *byte == b':')?;
    if colon == 0 {
        return None;
    }
    let key_end = content[..colon]
        .iter()
        .rposition(|byte| !horizontal_space(*byte))?
        + 1;
    Some((
        AsciiString::from_bytes(&content[..key_end]),
        resource_value(&content[colon + 1..]),
    ))
}
fn resource_value(bytes: &[u8]) -> AsciiString {
    let start = bytes
        .iter()
        .position(|byte| !horizontal_space(*byte))
        .unwrap_or(bytes.len());
    let mut value = &bytes[start..];
    if value.first() == Some(&b'\\') && matches!(value.get(1), Some(b'\\' | b' ' | b'\t')) {
        value = &value[1..];
    }
    value = value.strip_suffix(b"\n").unwrap_or(value);
    value = value.strip_suffix(b"\r").unwrap_or(value);
    AsciiString::from_bytes(value)
}
pub(super) fn saved_line(key: &AsciiString, value: &AsciiString) -> Vec<u8> {
    let mut line = key.as_bytes().to_vec();
    line.extend_from_slice(b":\t");
    if matches!(value.as_bytes().first(), Some(b'\\' | b' ' | b'\t')) {
        line.push(b'\\');
    }
    line.extend_from_slice(value.as_bytes());
    line.push(b'\n');
    line
}
