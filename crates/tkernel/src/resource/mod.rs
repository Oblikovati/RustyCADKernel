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

//! OCCT resource files with separately injected storage and environment lookup.
use crate::{ncollection::DataMap, standard::Failure, tcollection::AsciiString};
use std::{path::PathBuf, rc::Rc};
mod io;
mod parser;
mod values;
pub use io::{NativeResourceEnvironment, NativeResourceIo, ResourceEnvironment, ResourceIo};

/// Reference resources and explicit user overrides; `catalog.value("key")?`.
pub struct ResourceCatalog {
    name: String,
    reference: DataMap<AsciiString, AsciiString>,
    overrides: DataMap<AsciiString, AsciiString>,
    initialized: bool,
    io: Rc<dyn ResourceIo>,
    environment: Rc<dyn ResourceEnvironment>,
}
impl ResourceCatalog {
    /// Loads defaults followed by user defaults; `ResourceCatalog::new("App", io, environment)`.
    pub fn new(
        name: &str,
        io: Rc<dyn ResourceIo>,
        environment: Rc<dyn ResourceEnvironment>,
    ) -> Self {
        let mut catalog = Self {
            name: name.split('\0').next().unwrap().into(),
            reference: DataMap::default(),
            overrides: DataMap::default(),
            initialized: false,
            io,
            environment,
        };
        for suffix in ["Defaults", "UserDefaults"] {
            if let Some(directory) = catalog.directory(suffix) {
                catalog.load(directory);
            }
        }
        catalog
    }
    fn directory(&self, suffix: &str) -> Option<PathBuf> {
        self.environment
            .directory(&format!("CSF_{}{suffix}", self.name))
            .filter(|directory| !directory.as_os_str().is_empty())
    }
    fn load(&mut self, directory: PathBuf) {
        let Ok(bytes) = self.io.read(&directory.join(&self.name)) else {
            return;
        };
        self.initialized = true;
        for line in bytes.split_inclusive(|byte| *byte == b'\n') {
            if let Some((key, value)) = parser::resource_line(line) {
                self.reference.bind(key, value);
            }
        }
    }
    /// Reports whether a reference file was opened, even when empty; `catalog.is_initialized()`.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    /// Looks in overrides before defaults; `catalog.find("language")`.
    pub fn find(&self, name: &str) -> bool {
        self.value(name).is_ok()
    }
    /// Returns Resource_NoSuchResource when absent; `catalog.value("language")?`.
    pub fn value(&self, name: &str) -> Result<&AsciiString, Failure> {
        let key = AsciiString::new(name);
        self.overrides
            .find(&key)
            .or_else(|_| self.reference.find(&key))
            .map_err(|_| Failure::with_type("Resource_NoSuchResource", name, ""))
    }
    /// Saves only explicit overrides, reevaluating the destination environment; `catalog.save()`.
    pub fn save(&self) -> bool {
        let Some(directory) = self.directory("UserDefaults") else {
            return false;
        };
        if self.io.create_directories(&directory).is_err() {
            return false;
        }
        let Ok(mut output) = self.io.open_write(&directory.join(&self.name)) else {
            return false;
        };
        let mut entries: Vec<_> = self.overrides.items().collect();
        entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        for (key, value) in entries {
            // Resource_Manager::Save checks opening, but intentionally ignores Write/Close errors.
            let _ = output.write_all(&parser::saved_line(key, value));
        }
        true
    }
}
