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

//! Named fakes for the units tests: an in-memory resource store with its own
//! environment, and a recorder of the units console and messages.
#![allow(dead_code)]
use rustycad_tkernel::resource::{ResourceEnvironment, ResourceIo};
use rustycad_tkernel::units::{Units, UnitsDiagnostics};
use rustycad_tkernel::units_api::UnitsApi;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};

/// Resource files and `CSF_*` variables held in memory.
#[derive(Default)]
pub struct InMemoryUnitsResources {
    pub files: Rc<RefCell<BTreeMap<PathBuf, Vec<u8>>>>,
    pub variables: RefCell<BTreeMap<String, PathBuf>>,
}

impl InMemoryUnitsResources {
    /// Defines `CSF_<name>Defaults` as `directory` holding the file `name` with `text`.
    pub fn define(&self, name: &str, directory: &str, text: &str) {
        let directory = PathBuf::from(directory);
        let variable = format!("CSF_{name}Defaults");
        self.variables
            .borrow_mut()
            .insert(variable, directory.clone());
        let path = directory.join(name);
        self.files
            .borrow_mut()
            .insert(path, text.as_bytes().to_vec());
    }
    /// Defines `CSF_<name>UserDefaults` as `directory`.
    pub fn define_user(&self, name: &str, directory: &str) {
        let variable = format!("CSF_{name}UserDefaults");
        self.variables
            .borrow_mut()
            .insert(variable, PathBuf::from(directory));
    }
}

struct InMemoryWriter {
    files: Rc<RefCell<BTreeMap<PathBuf, Vec<u8>>>>,
    path: PathBuf,
}

impl Write for InMemoryWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut files = self.files.borrow_mut();
        files
            .entry(self.path.clone())
            .or_default()
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl ResourceEnvironment for InMemoryUnitsResources {
    fn directory(&self, name: &str) -> Option<PathBuf> {
        self.variables.borrow().get(name).cloned()
    }
}

impl ResourceIo for InMemoryUnitsResources {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let files = self.files.borrow();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }
    fn create_directories(&self, _path: &Path) -> io::Result<()> {
        Ok(())
    }
    fn open_write(&self, path: &Path) -> io::Result<Box<dyn Write>> {
        self.files
            .borrow_mut()
            .insert(path.to_path_buf(), Vec::new());
        let files = self.files.clone();
        let path = path.to_path_buf();
        Ok(Box::new(InMemoryWriter { files, path }))
    }
}

/// Records console text and messages of a units context.
#[derive(Default, Clone)]
pub struct RecordingUnitsDiagnostics {
    pub printed: Arc<Mutex<Vec<u8>>>,
    pub messages: Arc<Mutex<Vec<String>>>,
}

impl UnitsDiagnostics for RecordingUnitsDiagnostics {
    fn print(&self, text: &[u8]) {
        self.printed.lock().unwrap().extend_from_slice(text);
    }
    fn warning(&self, text: &[u8]) {
        let text = format!("W:{}", String::from_utf8_lossy(text));
        self.messages.lock().unwrap().push(text);
    }
    fn info(&self, text: &[u8]) {
        let text = format!("I:{}", String::from_utf8_lossy(text));
        self.messages.lock().unwrap().push(text);
    }
}

/// A fresh units context reporting into `recorder`.
pub fn units_with(recorder: &RecordingUnitsDiagnostics) -> Arc<Units> {
    Arc::new(Units::with_diagnostics(Box::new(recorder.clone())))
}

/// A `UnitsApi` over a fresh context and an empty resource store (no `CSF_*`).
pub fn api() -> UnitsApi {
    api_with(Rc::new(InMemoryUnitsResources::default()))
}

/// A `UnitsApi` over a fresh context and the given resource store.
pub fn api_with(resources: Rc<InMemoryUnitsResources>) -> UnitsApi {
    let units = units_with(&RecordingUnitsDiagnostics::default());
    UnitsApi::new(units, resources.clone(), resources)
}
