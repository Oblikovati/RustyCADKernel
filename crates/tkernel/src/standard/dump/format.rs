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

//! `Standard_Dump::Text` and `FormatJson`.
use super::stream::DumpStream;
use crate::tcollection::AsciiString;

/// `Text`: the stream's content as `str().c_str()` sees it, so it ends at the first NUL;
/// `text(&stream)`.
pub fn text(stream: &DumpStream) -> AsciiString {
    let bytes = stream.bytes();
    AsciiString::from_bytes(bytes.split(|byte| *byte == 0).next().unwrap_or_default())
}

/// Indented output under construction.
struct Formatter {
    output: Vec<u8>,
    indent: Vec<u8>,
    depth: i32,
    in_array: bool,
}

impl Formatter {
    fn new_line(&mut self) {
        self.output.push(b'\n');
        for _ in 0..self.depth {
            self.output.extend_from_slice(&self.indent);
        }
    }
    fn open(&mut self) {
        self.depth += 1;
        self.output.push(b'{');
        self.new_line();
    }
    fn close(&mut self) {
        self.depth -= 1;
        self.new_line();
        self.output.push(b'}');
    }
}

/// `FormatJson`: puts `{`, `}` and top-level `,` on their own indented lines, adding the
/// outer braces when the text lacks them; `format_json(&stream, 3)`.
///
/// Native details kept: commas inside `[...]` stay inline, a space after a comma is skipped
/// only when it is not the last character, line feeds are dropped, an empty stream gives
/// an empty text and a negative indent means no indentation.
pub fn format_json(stream: &DumpStream, indent: i32) -> AsciiString {
    let source = text(stream);
    let bytes = source.as_bytes();
    let mut formatter = Formatter {
        output: Vec::new(),
        indent: vec![b' '; indent.max(0) as usize],
        depth: 0,
        in_array: false,
    };
    let length = bytes.len();
    let mut index = 1;
    while index <= length {
        let mut symbol = bytes[index - 1];
        if index == 1 && formatter.output.is_empty() && symbol != b'{' {
            // The missing opening brace is emitted first and this symbol is read again.
            symbol = b'{';
            index -= 1;
        }
        index += format_symbol(&mut formatter, bytes, index, symbol);
        if index == length && symbol != b'}' {
            formatter.close();
        }
        index += 1;
    }
    AsciiString::from_bytes(&formatter.output)
}

/// Emits one symbol; returns 1 when the space after a comma was consumed as well.
fn format_symbol(formatter: &mut Formatter, bytes: &[u8], index: usize, symbol: u8) -> usize {
    match symbol {
        b'{' => formatter.open(),
        b'}' => formatter.close(),
        b'[' | b']' => {
            formatter.in_array = symbol == b'[';
            formatter.output.push(symbol);
        }
        b',' if !formatter.in_array => {
            formatter.output.push(b',');
            formatter.new_line();
            let skip = index + 1 < bytes.len() && bytes[index] == b' ';
            return usize::from(skip);
        }
        // JSON has no multi-line values, so line feeds are dropped.
        b'\n' => {}
        other => formatter.output.push(other),
    }
    0
}
