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

//! Injected host services: files (OSD_File/OSD_Directory), environment (OSD_Environment)
//! and the console (`std::cout`) that Resource_Manager reaches globally in C++.
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    rc::Rc,
};
/// Environment seam without process-global mutation; `environment.directory("CSF_AppDefaults")`.
pub trait ResourceEnvironment {
    /// Resolves a variable (OSD_Environment::Value); `directory("CSF_AppUserDefaults")`.
    fn directory(&self, name: &str) -> Option<PathBuf>;
    /// True when the variable holds a nonempty value, as `!OSD_Environment(name).Value().IsEmpty()`.
    ///
    /// ```
    /// use rustycad_tkernel::resource::{NativeResourceEnvironment, ResourceEnvironment};
    /// let _ = NativeResourceEnvironment.is_set("ResourceDebug");
    /// ```
    fn is_set(&self, name: &str) -> bool {
        self.directory(name)
            .is_some_and(|value| !value.as_os_str().is_empty())
    }
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
/// Console seam for the verbose and `ResourceDebug` traces native code prints to `std::cout`.
pub trait ResourceConsole {
    /// Writes raw bytes exactly as the native stream insertion would; `console.write(b"text\n")`.
    fn write(&self, bytes: &[u8]);
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
/// Standard output, the native destination of Resource_Manager traces.
pub struct NativeResourceConsole;
impl ResourceConsole for NativeResourceConsole {
    fn write(&self, bytes: &[u8]) {
        // Native `std::cout <<` ignores stream failures as well.
        let _ = io::stdout().write_all(bytes);
    }
}
/// The three host seams a Resource_Manager uses; `ResourceHost::native()`.
#[derive(Clone)]
pub struct ResourceHost {
    /// File access for Load and Save.
    pub io: Rc<dyn ResourceIo>,
    /// Environment lookup for `CSF_<Name>Defaults` and the trace switches.
    pub environment: Rc<dyn ResourceEnvironment>,
    /// Destination of verbose and debug traces.
    pub console: Rc<dyn ResourceConsole>,
}
impl ResourceHost {
    /// Bundles injected seams; `ResourceHost::new(io, environment, console)`.
    pub fn new(
        io: Rc<dyn ResourceIo>,
        environment: Rc<dyn ResourceEnvironment>,
        console: Rc<dyn ResourceConsole>,
    ) -> Self {
        Self {
            io,
            environment,
            console,
        }
    }
    /// The process filesystem, environment and standard output; `ResourceHost::native()`.
    pub fn native() -> Self {
        Self::new(
            Rc::new(NativeResourceIo),
            Rc::new(NativeResourceEnvironment),
            Rc::new(NativeResourceConsole),
        )
    }
}
