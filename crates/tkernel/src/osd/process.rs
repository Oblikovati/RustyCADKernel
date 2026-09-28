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

//! Process directory lookup through an injectable read-only environment interface.
use super::CadPath;
use std::{io, path::PathBuf};
/// Source of a process working directory; implement for deterministic tests.
pub trait DirectorySource {
    /// Reads a working directory; `source.current_directory()`.
    fn current_directory(&self) -> io::Result<PathBuf>;
}
/// Native environment source; `NativeDirectory.current_directory()`.
pub struct NativeDirectory;
impl DirectorySource for NativeDirectory {
    fn current_directory(&self) -> io::Result<PathBuf> {
        std::env::current_dir()
    }
}
/// Process path services; `Process::current_directory()?`.
pub struct Process;
impl Process {
    /// Reads the actual process directory as a directory trek; `Process::current_directory()?`.
    pub fn current_directory() -> io::Result<CadPath> {
        Self::current_directory_with(&NativeDirectory)
    }
    /// Reads an injected source and appends the directory separator; `Process::current_directory_with(&source)?`.
    pub fn current_directory_with(source: &impl DirectorySource) -> io::Result<CadPath> {
        let directory = source.current_directory()?;
        let mut bytes = path_bytes(&directory);
        bytes.push(b'/');
        Ok(CadPath::new(bytes))
    }
}
#[cfg(unix)]
fn path_bytes(path: &std::path::Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    path.as_os_str().as_bytes().to_vec()
}
#[cfg(not(unix))]
fn path_bytes(path: &std::path::Path) -> Vec<u8> {
    path.to_string_lossy().as_bytes().to_vec()
}
