// Copyright (c) 2026 OPEN CASCADE SAS
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

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
/// Environment seam without process-global mutation; `environment.directory("CSF_AppDefaults")`.
pub trait ResourceEnvironment {
    /// Resolves a resource-directory variable; `directory("CSF_AppUserDefaults")`.
    fn directory(&self, name: &str) -> Option<PathBuf>;
}
/// Filesystem seam for resource loading and saving; inject a named fake in tests.
pub trait ResourceIo {
    /// Reads a complete resource file; `io.read(Path::new("defaults/App"))`.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    /// Creates missing parent directories; `io.create_directories(Path::new("user/nested"))`.
    fn create_directories(&self, path: &Path) -> io::Result<()>;
    /// Opens and truncates the destination, matching OSD_File::Build; `io.open_write(path)`.
    fn open_write(&self, path: &Path) -> io::Result<Box<dyn Write>>;
}
/// Read-only access to the host environment; inject `Rc::new(NativeResourceEnvironment)`.
pub struct NativeResourceEnvironment;
impl ResourceEnvironment for NativeResourceEnvironment {
    fn directory(&self, name: &str) -> Option<PathBuf> {
        std::env::var_os(name).map(PathBuf::from)
    }
}
/// Host filesystem implementation; inject `Rc::new(NativeResourceIo)` in applications.
pub struct NativeResourceIo;
impl ResourceIo for NativeResourceIo {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }
    fn create_directories(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }
    fn open_write(&self, path: &Path) -> io::Result<Box<dyn Write>> {
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map(|file| Box::new(file) as Box<dyn Write>)
    }
}
