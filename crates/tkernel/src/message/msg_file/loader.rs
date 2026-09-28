// Created on: 2001-04-26
// Created by: OCC Team
// Copyright (c) 2001-2014 OPEN CASCADE SAS
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

//! The native message-file parser (`getString` and `loadFile`) over 8- and 16-bit units.
//!
//! The native parser splits a NUL-terminated buffer in place; here a line is a range of
//! a slice and the end of the slice (or its first zero unit) is the terminator.
use super::MsgFile;
use crate::standard::Failure;
use crate::tcollection::{AsciiString, ExtendedString};

/// A code unit of a message buffer: `char` or `char16_t`.
trait Unit: Copy {
    fn code(self) -> u32;
    fn text(units: &[Self]) -> ExtendedString;
}
impl Unit for u8 {
    fn code(self) -> u32 {
        u32::from(self)
    }
    /// `TCollection_AsciiString(ptr)` converted to extended text as UTF-8 when valid.
    fn text(units: &[Self]) -> ExtendedString {
        ExtendedString::from_ascii(&AsciiString::from_bytes(units))
    }
}
impl Unit for u16 {
    fn code(self) -> u32 {
        u32::from(self)
    }
    fn text(units: &[Self]) -> ExtendedString {
        ExtendedString::from_utf16(units)
    }
}

/// One significant line: its text and the indentation counted before it.
struct Line {
    text: ExtendedString,
    left_spaces: i32,
}

fn unit_at<T: Unit>(buffer: &[T], index: usize) -> u32 {
    buffer.get(index).map_or(0, |unit| unit.code())
}

/// Skips blanks and line breaks; a tab counts 8 spaces, a break resets the count.
fn skip_blanks<T: Unit>(buffer: &[T], mut index: usize) -> (usize, i32) {
    let mut spaces = 0;
    loop {
        match unit_at(buffer, index) {
            0x20 => spaces += 1,
            0x09 => spaces += 8,
            0x0d | 0x0a => spaces = 0,
            _ => return (index, spaces),
        }
        index += 1;
    }
}

/// End of the line starting at `start`: the `\n`, or the `\r` of a `\r\n`, or the terminator.
fn line_end<T: Unit>(buffer: &[T], start: usize) -> usize {
    let mut end = start;
    while unit_at(buffer, end) != 0 {
        if unit_at(buffer, end) == 0x0a {
            if unit_at(buffer, end - 1) == 0x0d {
                end -= 1;
            }
            break;
        }
        end += 1;
    }
    end
}

/// `getString`: the next line that is not a `!` comment; advances `cursor` past it.
fn next_line<T: Unit>(buffer: &[T], cursor: &mut usize) -> Option<Line> {
    let mut end = *cursor;
    let (start, spaces) = loop {
        let (start, spaces) = skip_blanks(buffer, end);
        end = line_end(buffer, start);
        if unit_at(buffer, start) != u32::from(b'!') {
            break (start, spaces);
        }
    };
    if start == end {
        return None;
    }
    // The native parser overwrites the terminating unit with NUL and steps past it.
    *cursor = if unit_at(buffer, end) != 0 {
        end + 1
    } else {
        end
    };
    Some(Line {
        text: T::text(&buffer[start..end]),
        left_spaces: spaces,
    })
}

/// `LoadingState` of the native parser.
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Keyword,
    Message,
    MoreMessage,
}

/// Parser state carried between lines.
struct Parse<'a> {
    catalog: &'a MsgFile,
    state: State,
    keyword: AsciiString,
    message: ExtendedString,
    first_left_spaces: i32,
}
impl Parse<'_> {
    /// One line through the native `switch` with its fall-through cases.
    fn accept(&mut self, line: Line) -> Result<(), Failure> {
        let is_keyword = line.text.as_utf16().first() == Some(&u16::from(b'.'));
        if self.state == State::MoreMessage {
            if !is_keyword {
                self.continue_message(&line);
                return Ok(());
            }
            self.catalog.add_msg(&self.keyword, &self.message);
        }
        if self.state != State::Keyword && !is_keyword {
            self.message = line.text;
            self.first_left_spaces = line.left_spaces;
            self.state = State::MoreMessage;
            return Ok(());
        }
        if is_keyword {
            self.keyword = keyword_of(&line.text)?;
            self.state = State::Message;
        }
        Ok(())
    }
    /// Appends a continuation line, re-indented relative to the message's first line.
    fn continue_message(&mut self, line: &Line) {
        self.message.assign_cat(u16::from(b'\n'));
        let indent = line.left_spaces - self.first_left_spaces;
        if indent > 0 {
            let spaces = vec![u16::from(b' '); indent as usize];
            self.message.assign_cat(ExtendedString::from_raw(&spaces));
        }
        self.message.assign_cat(&line.text);
    }
}

/// `TCollection_AsciiString(line.Split(1))`, trimmed of C spaces on both sides.
fn keyword_of(line: &ExtendedString) -> Result<AsciiString, Failure> {
    if line.len() <= 1 {
        return Err(Failure::with_type(
            "Standard_OutOfRange",
            &format!(
                "TCollection_ExtendedString::Split index 1; keyword line length {}, expected > 1",
                line.len()
            ),
            "",
        ));
    }
    let name = AsciiString::from(&ExtendedString::from_raw(&line.as_utf16()[1..]));
    let bytes = name.as_bytes();
    let is_space = |byte: &u8| matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c);
    let start = bytes
        .iter()
        .position(|byte| !is_space(byte))
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !is_space(byte))
        .map_or(start, |last| last + 1);
    Ok(AsciiString::from_raw(&bytes[start..end]))
}

/// `loadFile<CharType>`: parses every line of a buffer into the catalog.
fn load_units<T: Unit>(catalog: &MsgFile, buffer: &[T]) -> Result<bool, Failure> {
    let mut parse = Parse {
        catalog,
        state: State::Keyword,
        keyword: AsciiString::default(),
        message: ExtendedString::default(),
        first_left_spaces: 0,
    };
    let mut cursor = 0;
    while let Some(line) = next_line(buffer, &mut cursor) {
        parse.accept(line)?;
    }
    if parse.state == State::MoreMessage {
        catalog.add_msg(&parse.keyword, &parse.message);
    }
    Ok(true)
}

/// Parses 8-bit content (`LoadFromString`, or a file without a BOM).
pub(super) fn load_bytes(catalog: &MsgFile, content: &[u8]) -> Result<bool, Failure> {
    load_units(catalog, content)
}

/// `LoadFile` after reading: a `FF FE`/`FE FF` BOM selects UTF-16 little/big endian.
///
/// For an odd byte count the native code reads one unit past its buffer (undefined
/// behavior); here the dangling byte forms a last unit with a zero high/low byte.
pub(super) fn load_buffer(catalog: &MsgFile, content: &[u8]) -> Result<bool, Failure> {
    let big_endian = content.starts_with(&[0xfe, 0xff]);
    if !content.starts_with(&[0xff, 0xfe]) && !big_endian {
        return load_bytes(catalog, content);
    }
    let units: Vec<u16> = content[2..]
        .chunks(2)
        .map(|pair| {
            let bytes = [pair[0], pair.get(1).copied().unwrap_or(0)];
            if big_endian {
                u16::from_be_bytes(bytes)
            } else {
                u16::from_le_bytes(bytes)
            }
        })
        .collect();
    load_units(catalog, &units)
}
