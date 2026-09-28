// Copyright (c) 2016 OPEN CASCADE SAS
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

//! Borrowed memory streams preserving OCCT's bounded seek behavior.
use std::io::{self, BufRead, Read, Seek, SeekFrom};

/// A non-owning readable buffer; `ArrayStreamBuffer::new(b"Hello")`.
pub struct ArrayStreamBuffer<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ArrayStreamBuffer<'a> {
    /// Borrows memory without copying; `ArrayStreamBuffer::new(b"ABC")`.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    /// Maps a null, zero-length C++ buffer to an empty slice; `ArrayStreamBuffer::from_optional(None)`.
    pub fn from_optional(bytes: Option<&'a [u8]>) -> Self {
        Self::new(bytes.unwrap_or_default())
    }
    /// Replaces the borrowed buffer and resets its position; `buffer.init(b"replacement")`.
    pub fn init(&mut self, bytes: &'a [u8]) {
        self.bytes = bytes;
        self.position = 0;
    }
    /// Returns unread bytes; `ArrayStreamBuffer::new(b"ABC").available() == 3`.
    pub const fn available(&self) -> usize {
        self.bytes.len() - self.position
    }
    /// Peeks without consuming; `ArrayStreamBuffer::new(b"ABC").peek() == Some(b'A')`.
    pub fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }
    /// Backs up only for EOF or the matching previous byte; `buffer.put_back(Some(b'A'))`.
    pub fn put_back(&mut self, byte: Option<u8>) -> Option<u8> {
        let previous = self.position.checked_sub(1)?;
        let actual = self.bytes[previous];
        if byte.is_some_and(|requested| requested != actual) {
            return None;
        }
        self.position = previous;
        Some(actual)
    }
    /// Matches OCCT seekoff, whose end offset is subtracted; `buffer.seek_occt(SeekFrom::End(3))?`.
    /// Rust's `Seek` trait uses its conventional signed end-relative offset instead.
    pub fn seek_occt(&mut self, offset: SeekFrom) -> io::Result<u64> {
        match offset {
            SeekFrom::End(distance) => {
                self.set_position((self.bytes.len() as i128 - i128::from(distance)).max(0))
            }
            other => self.seek(other),
        }
    }
    fn set_position(&mut self, position: i128) -> io::Result<u64> {
        if position < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("position {position}: expected 0..={}", self.bytes.len()),
            ));
        }
        self.position = position.min(self.bytes.len() as i128) as usize;
        Ok(self.position as u64)
    }
}

impl Read for ArrayStreamBuffer<'_> {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        let count = destination.len().min(self.available());
        destination[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}
impl BufRead for ArrayStreamBuffer<'_> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(&self.bytes[self.position..])
    }
    fn consume(&mut self, amount: usize) {
        self.position += amount.min(self.available());
    }
}
impl Seek for ArrayStreamBuffer<'_> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        let position = match offset {
            SeekFrom::Start(position) => i128::from(position),
            SeekFrom::Current(delta) => self.position as i128 + i128::from(delta),
            SeekFrom::End(delta) => self.bytes.len() as i128 + i128::from(delta),
        };
        self.set_position(position)
    }
}
