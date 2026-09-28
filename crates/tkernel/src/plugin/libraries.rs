// Created on: 1997-03-06
// Created by: Mister rmi
// Copyright (c) 1997-1999 Matra Datavision
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

//! The shared libraries `Plugin::Load` opens, as a seam: the port registers factories by
//! library file name instead of loading native code.
use crate::standard::Guid;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The `PLUGINFACTORY` entry point of a plugin library; `factory(&guid)`.
pub type PluginFactory<P> = Arc<dyn Fn(&Guid) -> P + Send + Sync>;

/// Where `Plugin::Load` finds libraries (`OSD_SharedLibrary`); inject a named fake in tests.
pub trait PluginLibraries<P> {
    /// Opens a library by file name (`DlOpen`), or returns the `dlerror()` text; `open("libX.so")`.
    fn open(&self, file_name: &str) -> Result<(), String>;
    /// Looks up `PLUGINFACTORY` in an opened library (`DlSymb`), or returns the `dlerror()` text.
    fn factory(&self, file_name: &str) -> Result<PluginFactory<P>, String>;
}

/// Libraries linked into the program and registered by file name; `libraries.register("libX.so", f)`.
///
/// Refusals use the glibc `dlerror()` wording for the same situations, so the
/// `Plugin_Failure` messages read as they do natively on Linux.
pub struct StaticPluginLibraries<P> {
    libraries: BTreeMap<String, Option<PluginFactory<P>>>,
}

impl<P> Default for StaticPluginLibraries<P> {
    fn default() -> Self {
        Self {
            libraries: BTreeMap::new(),
        }
    }
}

impl<P> StaticPluginLibraries<P> {
    /// Registers a library exporting a factory; `libraries.register("libTKXCAF.so", factory)`.
    pub fn register(&mut self, file_name: &str, factory: PluginFactory<P>) {
        self.libraries.insert(file_name.to_owned(), Some(factory));
    }

    /// Registers a library that opens but exports no factory; `libraries.register_without_factory("libX.so")`.
    pub fn register_without_factory(&mut self, file_name: &str) {
        self.libraries.insert(file_name.to_owned(), None);
    }
}

impl<P> PluginLibraries<P> for StaticPluginLibraries<P> {
    fn open(&self, file_name: &str) -> Result<(), String> {
        if self.libraries.contains_key(file_name) {
            return Ok(());
        }
        Err(format!(
            "{file_name}: cannot open shared object file: No such file or directory"
        ))
    }

    fn factory(&self, file_name: &str) -> Result<PluginFactory<P>, String> {
        match self.libraries.get(file_name) {
            Some(Some(factory)) => Ok(Arc::clone(factory)),
            _ => Err(format!("{file_name}: undefined symbol: PLUGINFACTORY")),
        }
    }
}

/// The platform's library file name for a resource value (`lib<name>.so`, `lib<name>.dylib`,
/// `<name>.dll`); `library_file_name("TKXCAF") == "libTKXCAF.so"` on Linux.
pub fn library_file_name(name: &str) -> String {
    let prefix = if cfg!(windows) { "" } else { "lib" };
    let suffix = if cfg!(windows) {
        ".dll"
    } else if cfg!(target_vendor = "apple") {
        ".dylib"
    } else {
        ".so"
    };
    format!("{prefix}{name}{suffix}")
}
