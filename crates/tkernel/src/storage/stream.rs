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

//! `Standard_IStream`/`Standard_OStream`/`FSD_FStream`: a model of libstdc++'s
//! `std::basic_iostream<char>` over a file buffer.
//!
//! The storage drivers observe stream state bits, `gcount()` and positions, and several
//! of their quirks follow from exactly when libstdc++ sets `eofbit` or `failbit`. This
//! type reproduces those rules (sentries, unformatted and formatted I/O) over an injected
//! byte device, so drivers never touch the process file system directly.
use std::io::{Read, Seek, SeekFrom, Write};
mod formatted;
mod number_text;
pub use number_text::format_general;

/// Byte device behind a stream: a file, an in-memory buffer or a named test fake.
pub trait StreamDevice: Read + Write + Seek {}
impl<T: Read + Write + Seek> StreamDevice for T {}

/// `std::ios_base::iostate` bits, as libstdc++ numbers them.
pub mod state {
    /// `badbit`: an irrecoverable write or read failure.
    pub const BAD: u8 = 1;
    /// `eofbit`: an input operation reached the end of the device.
    pub const EOF: u8 = 2;
    /// `failbit`: an input operation extracted nothing or the value was invalid.
    pub const FAIL: u8 = 4;
}

const READ_CHUNK: usize = 4096;

/// A `std::iostream` over an optional device with libstdc++ state semantics.
///
/// A default stream is a closed `std::fstream`: reads report end of file and writes
/// set `badbit`, as with a `basic_filebuf` that is not open.
///
/// ```
/// use rustycad_tkernel::storage::IoStream;
/// let mut stream = IoStream::from_bytes(b" 42 x".to_vec());
/// let mut value = 0;
/// assert!(stream.extract_int(&mut value));
/// assert_eq!(value, 42);
/// ```
pub struct IoStream {
    device: Option<Box<dyn StreamDevice>>,
    readable: bool,
    writable: bool,
    state: u8,
    gcount: i64,
    precision: usize,
    width: i64,
    pending: Vec<u8>,
    cursor: usize,
}

impl Default for IoStream {
    fn default() -> Self {
        Self {
            device: None,
            readable: false,
            writable: false,
            state: 0,
            gcount: 0,
            precision: 6,
            width: 0,
            pending: Vec::new(),
            cursor: 0,
        }
    }
}

impl IoStream {
    /// Wraps an open device with the given directions; `IoStream::with_device(Box::new(file), true, false)`.
    pub fn with_device(device: Box<dyn StreamDevice>, readable: bool, writable: bool) -> Self {
        let mut stream = Self::default();
        stream.open(device, readable, writable);
        stream
    }

    /// A `std::stringstream` over `bytes`, readable and writable; `IoStream::from_bytes(vec![1, 2])`.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::with_device(Box::new(std::io::Cursor::new(bytes)), true, true)
    }

    /// `basic_fstream::open` success: attaches the device and clears the state bits.
    /// Example: `stream.open(Box::new(std::io::Cursor::new(Vec::new())), true, true)`.
    pub fn open(&mut self, device: Box<dyn StreamDevice>, readable: bool, writable: bool) {
        self.device = Some(device);
        self.readable = readable;
        self.writable = writable;
        self.pending.clear();
        self.cursor = 0;
        self.state = 0;
    }

    /// `basic_fstream::open` failure: the file buffer stays closed and `failbit` is set.
    /// Example: `stream.fail_open(); assert!(stream.fail())`.
    pub fn fail_open(&mut self) {
        self.set_state(state::FAIL);
    }

    /// `basic_fstream::close`: releases the device; closing a closed stream sets `failbit`.
    /// Example: `stream.close()`.
    pub fn close(&mut self) {
        match self.device.take() {
            Some(mut device) => {
                self.sync_device(&mut *device);
                if device.flush().is_err() {
                    self.set_state(state::FAIL);
                }
            }
            None => self.set_state(state::FAIL),
        }
        self.pending.clear();
        self.cursor = 0;
    }

    /// Reports whether a device is attached (`is_open()`); `stream.is_open()`.
    pub fn is_open(&self) -> bool {
        self.device.is_some()
    }

    /// `rdstate()`; combine with the [`state`] bits; `stream.rdstate() & state::EOF`.
    pub fn rdstate(&self) -> u8 {
        self.state
    }

    /// `good()`: no state bit is set; `stream.good()`.
    pub fn good(&self) -> bool {
        self.state == 0
    }

    /// `eof()`; `stream.eof()`.
    pub fn eof(&self) -> bool {
        self.state & state::EOF != 0
    }

    /// `fail()`: `failbit` or `badbit`; `stream.fail()`.
    pub fn fail(&self) -> bool {
        self.state & (state::FAIL | state::BAD) != 0
    }

    /// `bad()`; `stream.bad()`.
    pub fn bad(&self) -> bool {
        self.state & state::BAD != 0
    }

    /// `clear(state)`: replaces every bit; `stream.clear(0)`.
    pub fn clear(&mut self, bits: u8) {
        self.state = bits;
    }

    /// `setstate(bits)`: adds bits; `stream.set_state(state::FAIL)`.
    pub fn set_state(&mut self, bits: u8) {
        self.state |= bits;
    }

    /// `gcount()`: characters taken by the last unformatted input; `stream.gcount()`.
    pub fn gcount(&self) -> i64 {
        self.gcount
    }

    /// `precision()`; `stream.precision() == 6` for a new stream.
    pub fn precision(&self) -> usize {
        self.precision
    }

    /// `precision(n)`; the file drivers use 17; `stream.set_precision(17)`.
    pub fn set_precision(&mut self, precision: usize) {
        self.precision = precision;
    }

    /// `width()`; `stream.width()`.
    pub fn width(&self) -> i64 {
        self.width
    }

    /// `width(n)`, returning the previous width; `let old = stream.set_width(8190)`.
    pub fn set_width(&mut self, width: i64) -> i64 {
        std::mem::replace(&mut self.width, width)
    }

    /// Takes the device back, flushing pending reads; `let device = stream.into_device()`.
    pub fn into_device(mut self) -> Option<Box<dyn StreamDevice>> {
        let mut device = self.device.take()?;
        self.sync_device(&mut *device);
        Some(device)
    }
}

impl IoStream {
    /// `sgetc()`: the next byte without consuming it, `None` at end of input.
    pub(crate) fn peek_byte(&mut self) -> Option<u8> {
        if self.cursor == self.pending.len() && !self.refill() {
            return None;
        }
        Some(self.pending[self.cursor])
    }

    /// `sbumpc()`: consumes and returns the next byte.
    pub(crate) fn bump_byte(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.cursor += 1;
        Some(byte)
    }

    fn refill(&mut self) -> bool {
        if !self.readable {
            return false;
        }
        let Some(device) = self.device.as_mut() else {
            return false;
        };
        self.pending.resize(READ_CHUNK, 0);
        let count = device.read(&mut self.pending).unwrap_or(0);
        self.pending.truncate(count);
        self.cursor = 0;
        count > 0
    }

    /// Returns unread look-ahead to the device so writes and positions see the logical offset.
    fn sync_device(&mut self, device: &mut dyn StreamDevice) {
        let unread = (self.pending.len() - self.cursor) as i64;
        if unread > 0 {
            let _ = device.seek(SeekFrom::Current(-unread));
        }
        self.pending.clear();
        self.cursor = 0;
    }

    /// `sputn`: writes all bytes or reports how many were accepted.
    pub(crate) fn put_bytes(&mut self, bytes: &[u8]) -> usize {
        if !self.writable {
            return 0;
        }
        let Some(mut device) = self.device.take() else {
            return 0;
        };
        self.sync_device(&mut *device);
        let written = device.write_all(bytes).map_or(0, |()| bytes.len());
        self.device = Some(device);
        written
    }

    /// `pubseekoff(0, cur)`: the logical position, `-1` for a closed buffer.
    fn logical_position(&mut self) -> i64 {
        let Some(mut device) = self.device.take() else {
            return -1;
        };
        self.sync_device(&mut *device);
        let position = device.stream_position().map_or(-1, |p| p as i64);
        self.device = Some(device);
        position
    }
}

impl IoStream {
    /// The `istream::sentry`: skips C-locale spaces unless `noskipws`, and fails a stream
    /// that is not good or that reaches end of file while skipping.
    pub(crate) fn input_sentry(&mut self, noskipws: bool) -> bool {
        let mut error = 0;
        if self.good() && !noskipws {
            while self.peek_byte().is_some_and(is_c_space) {
                self.cursor += 1;
            }
            if self.peek_byte().is_none() {
                error |= state::EOF;
            }
        }
        if self.good() && error == 0 {
            return true;
        }
        self.set_state(error | state::FAIL);
        false
    }

    /// The `ostream::sentry`: succeeds on a good stream; a bad stream also gains `failbit`.
    pub(crate) fn output_sentry(&mut self) -> bool {
        if self.good() {
            return true;
        }
        if self.bad() {
            self.set_state(state::FAIL);
        }
        false
    }

    /// `get(char&)`: one byte, or `eofbit|failbit` leaving `value` unchanged; `stream.get(&mut c)`.
    pub fn get(&mut self, value: &mut u8) -> &mut Self {
        self.gcount = 0;
        let mut error = 0;
        if self.input_sentry(true) {
            match self.bump_byte() {
                Some(byte) => {
                    self.gcount = 1;
                    *value = byte;
                }
                None => error |= state::EOF,
            }
        }
        if self.gcount == 0 {
            error |= state::FAIL;
        }
        self.set_state(error);
        self
    }

    /// `getline(buffer, size, delimiter)`: stores at most `size - 1` bytes, consumes the
    /// delimiter, and fails on a full buffer or an empty extraction. The native buffer
    /// always receives its terminator, so `buffer` is cleared first.
    /// Example: `stream.getline(&mut line, 8192, b'\n')`.
    pub fn getline(&mut self, buffer: &mut Vec<u8>, size: i64, delimiter: u8) -> &mut Self {
        self.gcount = 0;
        buffer.clear();
        let mut error = 0;
        if self.input_sentry(true) {
            error = self.getline_body(buffer, size, delimiter);
        }
        if self.gcount == 0 {
            error |= state::FAIL;
        }
        self.set_state(error);
        self
    }

    fn getline_body(&mut self, buffer: &mut Vec<u8>, size: i64, delimiter: u8) -> u8 {
        loop {
            let next = self.peek_byte();
            match next {
                None => return state::EOF,
                Some(byte) if byte == delimiter => {
                    self.gcount += 1;
                    self.cursor += 1;
                    return 0;
                }
                Some(_) if self.gcount + 1 >= size => return state::FAIL,
                Some(byte) => {
                    buffer.push(byte);
                    self.cursor += 1;
                    self.gcount += 1;
                }
            }
        }
    }

    /// `read(buffer, n)`: fills the prefix it can; a short read sets `eofbit|failbit`.
    /// Example: `stream.read(&mut bytes[..4])`.
    pub fn read(&mut self, buffer: &mut [u8]) -> &mut Self {
        self.gcount = 0;
        if !self.input_sentry(true) {
            return self;
        }
        for slot in buffer.iter_mut() {
            let Some(byte) = self.bump_byte() else {
                self.set_state(state::EOF | state::FAIL);
                return self;
            };
            *slot = byte;
            self.gcount += 1;
        }
        self
    }

    /// `write(bytes, n)`: a short write sets `badbit`; `stream.write(b"BINFILE")`.
    pub fn write(&mut self, bytes: &[u8]) -> &mut Self {
        if self.output_sentry() && self.put_bytes(bytes) != bytes.len() {
            self.set_state(state::BAD);
        }
        self
    }

    /// `tellg()`: the input position, or -1 when the sentry or the stream fails.
    /// Unlike `tellp`, a stream at end of file gains `failbit` here; `stream.tellg()`.
    pub fn tellg(&mut self) -> i64 {
        if !self.input_sentry(true) || self.fail() {
            return -1;
        }
        self.logical_position()
    }

    /// `tellp()`: the output position, or -1 when `fail()`; end of file does not matter.
    /// Example: `stream.tellp()`.
    pub fn tellp(&mut self) -> i64 {
        self.output_sentry();
        if self.fail() {
            return -1;
        }
        self.logical_position()
    }
}

/// `isspace` in the "C" locale.
pub(crate) fn is_c_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
}
