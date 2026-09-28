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

//! Resource_Manager.cxx `GetLine` and `WhatKindOfLine` over the bytes of a resource file.
use crate::{standard::Failure, tcollection::AsciiString};

/// `GetLine` asks OSD_File::ReadLine for `BufSize = 10`, i.e. `fgets` of at most 9 bytes.
const FGETS_CHUNK: usize = 9;

/// Native `Resource_KindOfLine` without `Resource_KOL_End`, which is `None` from `next_line`.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum LineKind {
    Empty,
    Comment,
    Resource(AsciiString, AsciiString),
    Error,
}

/// Emulates the Unix OSD_File line reading of `GetLine`, including its quirks.
pub(super) struct LineReader<'a> {
    bytes: &'a [u8],
    position: usize,
    at_end: bool,
}
impl<'a> LineReader<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            at_end: false,
        }
    }
    /// One `fgets(buffer, 10, file)` followed by `strlen`: a NUL byte hides the rest of the
    /// chunk, even its newline. `None` is the NULL return that sets the EOF state.
    fn read_chunk(&mut self) -> Option<&'a [u8]> {
        let rest = &self.bytes[self.position..];
        if rest.is_empty() {
            self.at_end = true;
            return None;
        }
        let limit = rest.len().min(FGETS_CHUNK);
        let length = rest[..limit]
            .iter()
            .position(|&byte| byte == b'\n')
            .map_or(limit, |newline| newline + 1);
        self.position += length;
        let chunk = &rest[..length];
        Some(&chunk[..chunk.iter().position(|&b| b == 0).unwrap_or(length)])
    }
    /// Native `GetLine`: `None` at end of file, otherwise a line that always ends in `\n`.
    ///
    /// Quirk preserved: a chunk that starts with a NUL byte leaves the line empty before end
    /// of file, and the native `aLine.Value(aLine.Length())` then raises Standard_OutOfRange.
    pub(super) fn next_line(&mut self) -> Result<Option<Vec<u8>>, Failure> {
        let mut line = Vec::new();
        loop {
            if let Some(chunk) = self.read_chunk() {
                line.extend_from_slice(chunk);
            }
            if self.at_end {
                if line.is_empty() {
                    return Ok(None);
                }
                line.push(b'\n');
            }
            match line.last() {
                Some(b'\n') => return Ok(Some(line)),
                Some(_) => {}
                None => return Err(value_out_of_range()),
            }
        }
    }
}
fn value_out_of_range() -> Failure {
    Failure::out_of_range("TCollection_AsciiString::Value : parameter theWhere")
}
fn is_blank(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t')
}
/// Native `WhatKindOfLine` on a line ending in `\n`; `trace` receives the `ResourceDebug` text.
pub(super) fn line_kind(line: &[u8], trace: &mut dyn FnMut(&[u8])) -> LineKind {
    if line[0] == b'!' {
        return LineKind::Comment;
    }
    // The terminating newline is never blank, so both searches always find a position.
    let key_start = line.iter().position(|&b| !is_blank(b)).unwrap_or(0);
    if line[key_start] == b'\n' {
        return LineKind::Empty;
    }
    let Some(colon) = line[key_start..].iter().position(|&b| b == b':') else {
        return LineKind::Error;
    };
    if colon == 0 {
        return LineKind::Error;
    }
    let colon = key_start + colon;
    let key_end = (key_start..colon)
        .rev()
        .find(|&i| !is_blank(line[i]))
        .unwrap_or(key_start);
    let key = AsciiString::from_raw(&line[key_start..=key_end]);
    trace(&[b"Key = '".as_slice(), key.as_bytes()].concat());
    let value = AsciiString::from_raw(&resource_value(line, colon));
    trace(&[b"'\t Value = '".as_slice(), value.as_bytes(), b"'\n"].concat());
    LineKind::Resource(key, value)
}
/// The value part of `WhatKindOfLine`: blanks skipped, one escaping backslash dropped, then a
/// `\r` directly before the final `\n` and the `\n` itself removed.
fn resource_value(line: &[u8], colon: usize) -> Vec<u8> {
    let mut start = (colon + 1..line.len())
        .find(|&i| !is_blank(line[i]))
        .unwrap_or(line.len() - 1);
    if line[start] == b'\\' && matches!(line.get(start + 1), Some(b'\\' | b' ' | b'\t')) {
        start += 1;
    }
    if start == line.len() - 1 {
        return Vec::new();
    }
    let mut value = line[start..].to_vec();
    if value.len() >= 2 && value[value.len() - 2] == b'\r' {
        value.remove(value.len() - 2);
    }
    value.pop();
    value
}
/// Native Save line: `Key:\tValue\n`, the value escaped when it starts with `\`, space or tab.
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
