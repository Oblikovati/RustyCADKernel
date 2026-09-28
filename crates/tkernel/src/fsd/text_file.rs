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

//! `FSD_File` and its subclass `FSD_CmpFile`: the text container drivers over an
//! `std::fstream` (modelled by [`IoStream`]) in the "C" locale with 17 significant digits.
//!
//! The two classes share every member except the virtual line readers and the object
//! framing that `FSD_CmpFile` overrides; [`TextFile`] implements the shared members once
//! and a [`TextDialect`] supplies the overrides, so `FSD_File`'s own members still reach
//! `FSD_CmpFile`'s line readers the way the native virtual calls do.
use super::{DriverFileSystem, FileOpenMode};
use crate::standard::Failure;
use crate::storage::{BaseDriver, DriverBase, IoStream, OpenMode, Position, StorageError};
use crate::tcollection::AsciiString;
use std::marker::PhantomData;
use std::rc::Rc;
mod dialect;
mod lines;
mod objects;
mod scalars;
mod sections;
mod tables;
pub use dialect::{CmpDialect, FsdDialect, TextDialect};

/// `FSD_File`: the text driver with the `FSDFILE` magic number.
pub type File = TextFile<FsdDialect>;

/// `FSD_CmpFile`: the compatible text driver with the `CMPFILE` magic number.
pub type CmpFile = TextFile<CmpDialect>;

/// A text storage driver of dialect `D`.
///
/// ```
/// use rustycad_tkernel::fsd::{File, MemoryFileSystem};
/// use rustycad_tkernel::storage::{BaseDriver, DriverPrimitives, OpenMode};
/// use rustycad_tkernel::tcollection::AsciiString;
/// use std::rc::Rc;
/// let files = MemoryFileSystem::default();
/// let name = AsciiString::new("doc.fsd");
/// let mut driver = File::new(Rc::new(files.clone()));
/// driver.open(&name, OpenMode::Write).unwrap();
/// driver.put_real(0.1).unwrap();
/// driver.close().unwrap();
/// assert_eq!(files.contents(&name).unwrap(), b"0.10000000000000001 ");
/// ```
pub struct TextFile<D: TextDialect> {
    base: DriverBase,
    file_system: Rc<dyn DriverFileSystem>,
    stream: IoStream,
    dialect: PhantomData<D>,
}

impl<D: TextDialect> TextFile<D> {
    /// `FSD_File()`/`FSD_CmpFile()`: a closed stream; `File::new(file_system)`.
    pub fn new(file_system: Rc<dyn DriverFileSystem>) -> Self {
        Self {
            base: DriverBase::default(),
            file_system,
            stream: IoStream::default(),
            dialect: PhantomData,
        }
    }

    /// `MagicNumber`: `"FSDFILE"` or `"CMPFILE"`; `File::magic_number()`.
    pub fn magic_number() -> &'static str {
        D::MAGIC_NUMBER
    }

    /// `IsGoodFileType`: opens `name` for reading and compares its first seven characters
    /// with the magic number; `File::is_good_file_type(file_system, &name)`.
    pub fn is_good_file_type(
        file_system: Rc<dyn DriverFileSystem>,
        name: &AsciiString,
    ) -> Result<StorageError, Failure> {
        let mut file = Self::new(file_system);
        let status = file.open(name, OpenMode::Read)?;
        if status != StorageError::Ok {
            return Ok(status);
        }
        let magic = file.read_char(D::MAGIC_NUMBER.len());
        file.close()?;
        Ok(super::binary_file::magic_status(&magic, D::MAGIC_NUMBER))
    }

    /// `Destroy`: closes an open driver, as the destructor does; `driver.destroy()`.
    pub fn destroy(&mut self) {
        if self.base.open_mode() != OpenMode::None {
            let _ = self.close();
        }
    }

    /// The underlying stream (`myStream`, protected natively); `driver.stream()`.
    pub fn stream(&mut self) -> &mut IoStream {
        &mut self.stream
    }

    /// `myStream << text`, stopping at the first NUL like `ToCString()`.
    pub(crate) fn write_c_string(&mut self, text: &[u8]) {
        let length = text.iter().position(|byte| *byte == 0).unwrap_or(text.len());
        self.stream.insert_bytes(&text[..length]);
    }

    /// `if (myStream.bad()) throw Storage_StreamWriteError()`.
    pub(crate) fn check_written(&self, member: &str) -> Result<(), Failure> {
        if self.stream.bad() {
            return Err(super::binary_file::write_error(member));
        }
        Ok(())
    }
}

impl<D: TextDialect> Drop for TextFile<D> {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// The `std::ios::openmode` of each storage mode; `None` opens nothing.
fn stream_mode(mode: OpenMode) -> Option<FileOpenMode> {
    match mode {
        OpenMode::None => None,
        OpenMode::Read => Some(FileOpenMode::READ),
        OpenMode::Write => Some(FileOpenMode::WRITE),
        OpenMode::ReadWrite => Some(FileOpenMode::UPDATE),
    }
}

impl<D: TextDialect> BaseDriver for TextFile<D> {
    fn driver_base(&self) -> &DriverBase {
        &self.base
    }

    fn driver_base_mut(&mut self) -> &mut DriverBase {
        &mut self.base
    }

    /// `Open`: `in`, `out` or `in|out` with precision 17. `OpenMode::None` opens nothing
    /// yet succeeds on a stream that has not failed, as natively.
    fn open(&mut self, name: &AsciiString, mode: OpenMode) -> Result<StorageError, Failure> {
        self.base.set_name(name);
        if self.base.open_mode() != OpenMode::None {
            return Ok(StorageError::AlreadyOpen);
        }
        if let Some(file_mode) = stream_mode(mode) {
            match self.file_system.open(name, file_mode) {
                Ok(device) => self.stream.open(device, file_mode.read, file_mode.write),
                Err(_) => self.stream.fail_open(),
            }
        }
        if self.stream.fail() {
            return Ok(StorageError::OpenError);
        }
        self.stream.set_precision(17);
        self.base.set_open_mode(mode);
        Ok(StorageError::Ok)
    }

    fn is_end(&mut self) -> Result<bool, Failure> {
        Ok(self.stream.eof())
    }

    /// `Tell`: a read-only file reports `tellp` and a write-only file `tellg`, swapped as
    /// natively; a read-write file reports the larger.
    fn tell(&mut self) -> Result<Position, Failure> {
        Ok(match self.base.open_mode() {
            OpenMode::Read => self.stream.tellp(),
            OpenMode::Write => self.stream.tellg(),
            OpenMode::ReadWrite => {
                let read = self.stream.tellp();
                let write = self.stream.tellg();
                read.max(write)
            }
            OpenMode::None => -1,
        })
    }

    /// `SkipObject`: the rest of the line.
    fn skip_object(&mut self) -> Result<(), Failure> {
        self.flush_end_of_line();
        Ok(())
    }

    fn close(&mut self) -> Result<StorageError, Failure> {
        if self.base.open_mode() == OpenMode::None {
            return Ok(StorageError::NotOpen);
        }
        self.stream.close();
        self.base.set_open_mode(OpenMode::None);
        Ok(StorageError::Ok)
    }
}
