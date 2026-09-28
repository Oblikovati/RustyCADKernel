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

//! The file-system seam behind `OSD_OpenFile`/`OSD_OpenStream` for the FSD drivers.
use crate::storage::StreamDevice;
use crate::tcollection::AsciiString;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::rc::Rc;

/// How a driver opens its file: the `fopen` modes `rb`/`wb`/`w+b` and the `fstream` modes
/// `in`/`out`/`in|out` differ only in these three properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileOpenMode {
    /// The device may be read.
    pub read: bool,
    /// The device may be written.
    pub write: bool,
    /// A missing file is created and an existing one truncated.
    pub create_truncate: bool,
}

impl FileOpenMode {
    /// `fopen(..., "rb")` and `std::ios::in`: an existing file, read only.
    pub const READ: Self = Self {
        read: true,
        write: false,
        create_truncate: false,
    };
    /// `fopen(..., "wb")` and `std::ios::out`: created or truncated, write only.
    pub const WRITE: Self = Self {
        read: false,
        write: true,
        create_truncate: true,
    };
    /// `fopen(..., "w+b")`: created or truncated, read and write.
    pub const CREATE_READ_WRITE: Self = Self {
        read: true,
        write: true,
        create_truncate: true,
    };
    /// `std::ios::in | std::ios::out`: an existing file, read and write, not truncated.
    pub const UPDATE: Self = Self {
        read: true,
        write: true,
        create_truncate: false,
    };
}

/// Opens driver files by their OCCT name (bytes, UTF-8 on Windows); inject a fake in tests.
pub trait DriverFileSystem {
    /// Opens `name` with `mode`; a failure is the native null `FILE*` or failed stream.
    /// Example: `fs.open(&AsciiString::new("doc.fsd"), FileOpenMode::READ)?`.
    fn open(&self, name: &AsciiString, mode: FileOpenMode) -> io::Result<Box<dyn StreamDevice>>;
}

/// The host file system; `Rc::new(NativeDriverFileSystem)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeDriverFileSystem;

impl DriverFileSystem for NativeDriverFileSystem {
    fn open(&self, name: &AsciiString, mode: FileOpenMode) -> io::Result<Box<dyn StreamDevice>> {
        let file = std::fs::OpenOptions::new()
            .read(mode.read)
            .write(mode.write)
            .create(mode.create_truncate)
            .truncate(mode.create_truncate)
            .open(native_path(name))?;
        Ok(Box::new(file))
    }
}

#[cfg(unix)]
fn native_path(name: &AsciiString) -> std::path::PathBuf {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::OsStr::from_bytes(name.as_bytes()).into()
}

#[cfg(not(unix))]
fn native_path(name: &AsciiString) -> std::path::PathBuf {
    String::from_utf8_lossy(name.as_bytes()).into_owned().into()
}

type SharedFiles = Rc<RefCell<BTreeMap<Vec<u8>, Vec<u8>>>>;

/// An in-memory file system whose files outlive the devices opened on them.
///
/// ```
/// use rustycad_tkernel::fsd::{DriverFileSystem, FileOpenMode, MemoryFileSystem};
/// use rustycad_tkernel::tcollection::AsciiString;
/// use std::io::Write;
/// let files = MemoryFileSystem::default();
/// let name = AsciiString::new("a.bin");
/// files.open(&name, FileOpenMode::WRITE).unwrap().write_all(b"BINFILE").unwrap();
/// assert_eq!(files.contents(&name).unwrap(), b"BINFILE");
/// ```
#[derive(Clone, Default)]
pub struct MemoryFileSystem {
    files: SharedFiles,
}

impl MemoryFileSystem {
    /// The current bytes of a file; `files.contents(&name)`.
    pub fn contents(&self, name: &AsciiString) -> Option<Vec<u8>> {
        self.files.borrow().get(name.as_bytes()).cloned()
    }

    /// Creates or replaces a file; `files.insert(&name, bytes)`.
    pub fn insert(&self, name: &AsciiString, bytes: Vec<u8>) {
        self.files
            .borrow_mut()
            .insert(name.as_bytes().to_vec(), bytes);
    }
}

impl DriverFileSystem for MemoryFileSystem {
    fn open(&self, name: &AsciiString, mode: FileOpenMode) -> io::Result<Box<dyn StreamDevice>> {
        let key = name.as_bytes().to_vec();
        let mut files = self.files.borrow_mut();
        if mode.create_truncate {
            files.insert(key.clone(), Vec::new());
        } else if !files.contains_key(&key) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{:?}: no such file", String::from_utf8_lossy(&key)),
            ));
        }
        Ok(Box::new(MemoryFile {
            files: self.files.clone(),
            key,
            position: 0,
        }))
    }
}

/// A device over one entry of a [`MemoryFileSystem`].
struct MemoryFile {
    files: SharedFiles,
    key: Vec<u8>,
    position: u64,
}

impl MemoryFile {
    fn with_bytes<R>(&self, action: impl FnOnce(&mut Vec<u8>) -> R) -> R {
        let mut files = self.files.borrow_mut();
        action(files.entry(self.key.clone()).or_default())
    }
}

impl Read for MemoryFile {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let position = self.position;
        let count = self.with_bytes(|bytes| {
            let mut cursor = Cursor::new(bytes.as_slice());
            cursor.set_position(position);
            cursor.read(buffer)
        })?;
        self.position += count as u64;
        Ok(count)
    }
}

impl Write for MemoryFile {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let position = self.position;
        let count = self.with_bytes(|bytes| {
            let mut cursor = Cursor::new(bytes);
            cursor.set_position(position);
            cursor.write(data)
        })?;
        self.position += count as u64;
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for MemoryFile {
    fn seek(&mut self, target: SeekFrom) -> io::Result<u64> {
        let length = self.with_bytes(|bytes| bytes.len() as u64);
        let position = match target {
            SeekFrom::Start(offset) => Some(offset),
            SeekFrom::End(delta) => length.checked_add_signed(delta),
            SeekFrom::Current(delta) => self.position.checked_add_signed(delta),
        };
        self.position = position.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "seek {target:?} from {}; expected a non-negative offset",
                    self.position
                ),
            )
        })?;
        Ok(self.position)
    }
}
