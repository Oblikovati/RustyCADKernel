//! Named fakes for the units tests: an in-memory resource store with its own
//! environment, and a recorder of the units console and messages.
#![allow(dead_code)]
use rustycad_tkernel::resource::{ResourceConsole, ResourceEnvironment, ResourceHost, ResourceIo};
use rustycad_tkernel::units::{Units, UnitsDiagnostics};
use rustycad_tkernel::units_api::UnitsApi;
use std::{
    collections::BTreeMap,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Resource files and `CSF_*` variables held in memory.
#[derive(Default)]
pub struct InMemoryUnitsResources {
    pub files: Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
    pub variables: Mutex<BTreeMap<String, PathBuf>>,
    pub console: Mutex<Vec<u8>>,
}

impl InMemoryUnitsResources {
    /// Defines `CSF_<name>Defaults` as `directory` holding the file `name` with `text`.
    pub fn define(&self, name: &str, directory: &str, text: &str) {
        let directory = PathBuf::from(directory);
        let variable = format!("CSF_{name}Defaults");
        self.variables
            .lock()
            .unwrap()
            .insert(variable, directory.clone());
        let path = directory.join(name);
        self.files
            .lock()
            .unwrap()
            .insert(path, text.as_bytes().to_vec());
    }
    /// Defines `CSF_<name>UserDefaults` as `directory`.
    pub fn define_user(&self, name: &str, directory: &str) {
        let variable = format!("CSF_{name}UserDefaults");
        self.variables
            .lock()
            .unwrap()
            .insert(variable, PathBuf::from(directory));
    }
}

struct InMemoryWriter {
    files: Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
    path: PathBuf,
}

impl Write for InMemoryWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut files = self.files.lock().unwrap();
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
        self.variables.lock().unwrap().get(name).cloned()
    }
}

impl ResourceConsole for InMemoryUnitsResources {
    fn write(&self, bytes: &[u8]) {
        self.console.lock().unwrap().extend_from_slice(bytes);
    }
}

impl ResourceIo for InMemoryUnitsResources {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let files = self.files.lock().unwrap();
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
            .lock()
            .unwrap()
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
    api_with(Arc::new(InMemoryUnitsResources::default()))
}

/// A `UnitsApi` over a fresh context and the given resource store.
pub fn api_with(resources: Arc<InMemoryUnitsResources>) -> UnitsApi {
    let units = units_with(&RecordingUnitsDiagnostics::default());
    let host = ResourceHost::new(resources.clone(), resources.clone(), resources);
    UnitsApi::new(units, host)
}
