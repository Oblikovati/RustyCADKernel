// Copyright (c) 1996-1999 Matra Datavision
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

//! The protected line, word and tag readers of `FSD_File`.
use super::{TextDialect, TextFile};
use crate::standard::Failure;
use crate::storage::{StorageError, StreamException};
use crate::tcollection::{AsciiString, ExtendedString};

/// `getline` buffer size of the line readers (`char Buffer[8193]` read with 8192).
const LINE_BUFFER: i64 = 8192;
/// `ENDOFNORMALEXTENDEDSECTION`: an extended line that spells this tag is corrupt.
const END_OF_EXTENDED_SECTION: &[u8] = b"BEGIN_REF_SECTION";
/// `SIZEOFNORMALEXTENDEDSECTION`.
const EXTENDED_SECTION_SIZE: usize = 16;
/// `ReadWord` flushes its stack buffer every 8192 bytes.
const WORD_CHUNK: usize = 8192;

/// A C string view of `bytes`: everything before the first NUL (`buffer += Buffer`).
fn c_string(bytes: &[u8]) -> &[u8] {
    let length = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    &bytes[..length]
}

impl<D: TextDialect> TextFile<D> {
    /// `FSD_File::ReadLine`: one `getline` of at most 8191 bytes unless at end of file.
    pub(crate) fn fsd_read_line(&mut self) -> AsciiString {
        let mut buffer = AsciiString::default();
        if !self.stream.eof() {
            let mut line = Vec::new();
            self.stream.getline(&mut line, LINE_BUFFER, b'\n');
            buffer.assign_cat(c_string(&line));
        }
        buffer
    }

    /// `FSD_File::ReadString`: as `ReadLine`, dropping leading newlines and spaces.
    pub(crate) fn fsd_read_string(&mut self) -> AsciiString {
        let mut buffer = AsciiString::default();
        if !self.stream.eof() {
            let mut line = Vec::new();
            self.stream.getline(&mut line, LINE_BUFFER, b'\n');
            let text = c_string(&line);
            let start = text
                .iter()
                .position(|byte| *byte != b'\n' && *byte != b' ')
                .unwrap_or(text.len());
            buffer.assign_cat(&text[start..]);
        }
        buffer
    }

    /// `FSD_File::ReadWord`: skips spaces and newlines, then reads to the next one. The
    /// word is kept as C strings, so an embedded NUL cuts the chunk it is in.
    pub(crate) fn read_word(&mut self) -> AsciiString {
        let mut byte = 0_u8;
        while !self.stream.eof() {
            self.stream.get(&mut byte);
            if byte != b' ' && byte != b'\n' {
                break;
            }
        }
        let mut word = AsciiString::default();
        let mut chunk = Vec::with_capacity(WORD_CHUNK);
        while !self.stream.eof() {
            if chunk.len() == WORD_CHUNK {
                word.assign_cat(c_string(&chunk));
                chunk.clear();
            }
            chunk.push(byte);
            self.stream.get(&mut byte);
            if byte == b'\n' || byte == b' ' {
                break;
            }
        }
        word.assign_cat(c_string(&chunk));
        word
    }

    /// Protected `ReadChar`: `size` single `get`s; a failed `get` appends the previous byte.
    pub(crate) fn read_char(&mut self, size: usize) -> AsciiString {
        let mut buffer = AsciiString::default();
        let mut byte = 0_u8;
        let mut count = 0;
        while !self.stream.eof() && count < size {
            self.stream.get(&mut byte);
            buffer.assign_cat(byte);
            count += 1;
        }
        buffer
    }

    /// `FlushEndOfLine`: reads the rest of the line with the dialect's `ReadLine`.
    pub(crate) fn flush_end_of_line(&mut self) {
        D::read_line(self);
    }

    /// `FindTag`: reads strings until one equals `tag`. Reaching the end of the file, even
    /// on the tag line itself, is `SectionNotFound`, as natively.
    pub(crate) fn find_tag(&mut self, tag: &str) -> StorageError {
        let mut line = D::read_string(self);
        while line.as_bytes() != tag.as_bytes() && !self.stream.eof() {
            line = D::read_string(self);
        }
        if self.stream.eof() {
            StorageError::SectionNotFound
        } else {
            StorageError::Ok
        }
    }
}

impl<D: TextDialect> TextFile<D> {
    /// `FSD_File::WriteExtendedLine`: each unit as its high and low byte, then NUL and a
    /// newline. A unit 0x000A therefore ends the line early for the reader.
    pub(crate) fn fsd_write_extended_line(&mut self, text: &ExtendedString) {
        for unit in text.as_utf16() {
            self.stream.insert_bytes(&unit.to_be_bytes());
        }
        self.stream.insert_bytes(b"\0\n");
    }

    /// `FSD_File::ReadExtendedLine`: byte pairs until a NUL high byte followed by a newline.
    /// A pair that spells the first 16 characters of `BEGIN_REF_SECTION` is an
    /// ext-character parity error; zero units are dropped (`AssignCat` skips them).
    pub(crate) fn fsd_read_extended_line(&mut self, text: &mut ExtendedString) -> Result<(), Failure> {
        *text = ExtendedString::default();
        let (mut byte, mut matched, mut finished) = (0_u8, 0_usize, false);
        while !finished && !self.stream.eof() {
            self.stream.get(&mut byte);
            matched = tag_progress(byte, matched)?;
            let high = u16::from(byte);
            finished = byte == 0;
            self.stream.get(&mut byte);
            matched = tag_progress(byte, matched)?;
            if byte != b'\n' {
                finished = false;
                text.assign_cat((high << 8) | u16::from(byte));
            }
        }
        Ok(())
    }
}

/// Advances the match against `BEGIN_REF_SECTION`, failing when 16 characters match.
fn tag_progress(byte: u8, matched: usize) -> Result<usize, Failure> {
    let matched = if END_OF_EXTENDED_SECTION.get(matched) == Some(&byte) {
        matched + 1
    } else {
        0
    };
    if matched >= EXTENDED_SECTION_SIZE {
        return Err(StreamException::ExtCharParity.raise(
            "FSD_File::ReadExtendedLine: the line runs into BEGIN_REF_SECTION",
        ));
    }
    Ok(matched)
}
