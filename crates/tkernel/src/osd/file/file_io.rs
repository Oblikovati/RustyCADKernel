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

//! `OSD_File` reading, writing and positioning of the non-`_WIN32` build: `Read`/`Write`/
//! `Seek` use the descriptor, `ReadLine`/`Rewind` the buffered `FILE*` beside it.
use super::{
    calls::{Errno, SeekOrigin},
    osd_file::{File, IAM, program_error},
};
use crate::{
    osd::{FromWhere, KindFile, OpenMode},
    standard::Failure,
    tcollection::AsciiString,
};

fn out_of_range(message: &str) -> Failure {
    Failure::with_type("Standard_OutOfRange", message, "")
}

/// The bytes a C string built from `bytes` holds: up to the first NUL.
fn c_text(bytes: &[u8]) -> &[u8] {
    &bytes[..bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len())]
}

impl File {
    /// The common preconditions of `Read`/`ReadLine`: not a directory, open, no pending
    /// error (re-raised by `Perror`), readable and a positive count.
    fn check_readable(&mut self, member: &str, count: i32) -> Result<(), Failure> {
        if self.kind_of_file() == KindFile::Directory {
            return Err(program_error(&format!(
                "OSD_File::{member}(): it is a directory"
            )));
        }
        self.check_open(member)?;
        self.raise_pending()?;
        if self.mode == OpenMode::WriteOnly {
            return Err(program_error(&format!(
                "OSD_File::{member}(): file is Write only"
            )));
        }
        if count <= 0 {
            return Err(program_error(&format!(
                "OSD_File::{member}(): theNbBytes is 0"
            )));
        }
        Ok(())
    }
    /// One descriptor `read` of `count` bytes: a failure records errno and reads nothing,
    /// a short read marks the end of file.
    fn read_channel(&mut self, buffer: &mut [u8]) -> usize {
        match self.node.calls.read(self.channel, buffer) {
            Ok(count) => {
                self.at_end |= count < buffer.len();
                count
            }
            Err(errno) => {
                self.node.record(errno, IAM, "Read");
                0
            }
        }
    }
    /// `Read(buffer, nbyte)`: reads up to `nbyte` bytes into `buffer`, which keeps the
    /// text before the first NUL (empty when nothing was read); `file.read(&mut text, 80)?`.
    pub fn read(&mut self, buffer: &mut AsciiString, count: i32) -> Result<(), Failure> {
        self.check_readable("Read", count)?;
        let mut bytes: Vec<u8> = vec![0; count as usize];
        let read: usize = self.read_channel(&mut bytes);
        *buffer = AsciiString::from_raw(c_text(&bytes[..read]));
        Ok(())
    }
    /// `Read(void*, nbyte, readbyte)`: reads up to `count` bytes into `buffer`, returning
    /// the byte count; `file.read_bytes(&mut raw, 16)?`.
    ///
    /// `count` above the buffer length is refused with `Standard_OutOfRange` (natively a
    /// buffer overrun).
    pub fn read_bytes(&mut self, buffer: &mut [u8], count: i32) -> Result<i32, Failure> {
        self.check_readable("Read", count)?;
        let Some(target) = buffer.get_mut(..count as usize) else {
            let message: String = format!(
                "OSD_File::Read(): {count} bytes into a buffer of {}",
                buffer.len()
            );
            return Err(out_of_range(&message));
        };
        Ok(self.read_channel(target) as i32)
    }
    /// `ReadLine(buffer, nbyte, nbyteRead)`: `fgets` of at most `count - 1` bytes through
    /// the next newline (kept), returning the text length up to its first NUL;
    /// `file.read_line(&mut line, 1024)?`.
    ///
    /// At end of file the buffer is emptied, 0 is returned and `IsAtEnd` becomes true. A
    /// read error records errno and natively leaves both outputs untouched: the buffer is
    /// kept and 0 is returned.
    pub fn read_line(&mut self, buffer: &mut AsciiString, count: i32) -> Result<i32, Failure> {
        self.check_readable("ReadLine", count)?;
        let calls = std::sync::Arc::clone(&self.node.calls);
        let Some(stream) = self.stream.as_mut() else {
            // fdopen failed: natively fgets dereferences NULL.
            self.node.record(Errno(ebadf()), IAM, "ReadLine");
            return Ok(0);
        };
        let Some(line) = stream.fgets(calls.as_ref(), count) else {
            if !stream.eof() {
                let errno: i32 = stream.last_errno();
                self.node.record(Errno(errno), IAM, "ReadLine");
                return Ok(0);
            }
            self.at_end = true;
            *buffer = AsciiString::default();
            return Ok(0);
        };
        let text: &[u8] = c_text(&line);
        *buffer = AsciiString::from_raw(text);
        Ok(text.len() as i32)
    }
    /// `Write(buffer, nbyte)` of a string: writes its first `count` bytes, the terminating
    /// NUL included when `count` is its length plus one; `file.write(&text, text.length())?`.
    pub fn write(&mut self, buffer: &AsciiString, count: i32) -> Result<(), Failure> {
        self.write_bytes(buffer.as_bytes_with_nul(), count)
    }
    /// `Write(void*, nbyte)`: one descriptor `write`; a short write marks the end of file;
    /// `file.write_bytes(b"data", 4)?`. `count` above the buffer length is refused with
    /// `Standard_OutOfRange` (natively an over-read).
    pub fn write_bytes(&mut self, buffer: &[u8], count: i32) -> Result<(), Failure> {
        self.check_open("Write")?;
        self.raise_pending()?;
        if self.mode == OpenMode::ReadOnly {
            return Err(program_error("OSD_File::Write(): file is Read only"));
        }
        if count <= 0 {
            return Err(program_error("OSD_File::Write(): theNbBytes is null"));
        }
        let Some(source) = buffer.get(..count as usize) else {
            let message: String = format!(
                "OSD_File::Write(): {count} bytes from a buffer of {}",
                buffer.len()
            );
            return Err(out_of_range(&message));
        };
        match self.node.calls.write(self.channel, source) {
            Ok(written) => self.at_end |= written < source.len(),
            Err(errno) => self.node.record(errno, IAM, "Write"),
        }
        Ok(())
    }
    /// `Seek`: `lseek` of the descriptor (the stream buffer is not told); the end-of-file
    /// state is kept; `file.seek(0, FromWhere::FromBeginning)?`.
    pub fn seek(&mut self, offset: i32, whence: FromWhere) -> Result<(), Failure> {
        self.check_open("Seek")?;
        self.raise_pending()?;
        let origin: SeekOrigin = match whence {
            FromWhere::FromBeginning => SeekOrigin::Start,
            FromWhere::FromHere => SeekOrigin::Current,
            FromWhere::FromEnd => SeekOrigin::End,
        };
        if let Err(errno) = self
            .node
            .calls
            .lseek(self.channel, i64::from(offset), origin)
        {
            self.node.record(errno, IAM, "Seek");
        }
        Ok(())
    }
    /// `Rewind`: `rewind` of the stream, which may only move inside its buffer; a closed
    /// file (natively a NULL stream dereference) is left alone; `file.rewind()`.
    pub fn rewind(&mut self) {
        let calls = std::sync::Arc::clone(&self.node.calls);
        if let Some(stream) = self.stream.as_mut() {
            stream.rewind(calls.as_ref());
        }
    }
    /// `ReadLastLine`: up to `tries` `ReadLine(line, 1000)` calls, sleeping `delay`
    /// seconds (`OSD::SecSleep`) after each empty one; `file.read_last_line(&mut line, 1, 3)?`.
    pub fn read_last_line(
        &mut self,
        line: &mut AsciiString,
        delay: i32,
        tries: i32,
    ) -> Result<bool, Failure> {
        for _ in 0..tries.max(0) {
            self.read_line(line, 1000)?;
            if !line.is_empty() {
                return Ok(true);
            }
            sec_sleep(delay);
        }
        Ok(false)
    }
}

/// `OSD::SecSleep`: `usleep(seconds * 1000 * 1000)` with the native int arithmetic, so
/// the product wraps and is passed as unsigned microseconds.
fn sec_sleep(seconds: i32) {
    let micros: u32 = seconds.wrapping_mul(1_000_000) as u32;
    std::thread::sleep(std::time::Duration::from_micros(u64::from(micros)));
}

#[cfg(unix)]
const fn ebadf() -> i32 {
    libc::EBADF
}
#[cfg(not(unix))]
const fn ebadf() -> i32 {
    9
}
