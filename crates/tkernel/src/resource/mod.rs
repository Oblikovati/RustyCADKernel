//! OCCT's Resource package: resource files (Resource_Manager) with injected host services,
//! and the Resource_Unicode text conversions with their exact code tables.
use crate::{
    ncollection::DataMap,
    standard::Failure,
    tcollection::{AsciiString, ExtendedString},
};
use std::sync::atomic::{AtomicBool, Ordering};
mod cjk;
mod code_page;
pub mod convert_unicode;
mod format_type;
mod host;
mod lines;
mod multibyte;
mod paths;
mod save;
mod table_lookup;
mod tables;
mod unicode;
mod values;
pub use format_type::FormatType;
pub use host::{
    NativeResourceConsole, NativeResourceEnvironment, NativeResourceIo, ResourceConsole,
    ResourceEnvironment, ResourceHost, ResourceIo,
};
pub use unicode::ResourceUnicode;

/// The native file-static `Debug` flag: every name-based constructor re-reads it from the
/// `ResourceDebug` variable, and every manager's Load and Save consult it afterwards.
static DEBUG: AtomicBool = AtomicBool::new(false);

/// Reference resources and explicit user overrides (native Resource_Manager).
///
/// ```
/// use rustycad_tkernel::resource::{ResourceHost, ResourceManager};
/// let mut manager = ResourceManager::empty(ResourceHost::native());
/// manager.set_resource("key", b"value");
/// assert_eq!(manager.value("key").unwrap().as_bytes(), b"value");
/// ```
///
/// `Clone` is the implicit C++ copy constructor (`new Resource_Manager(*other)`, which
/// ShapeProcess_Context uses): the name, the three maps, the flags and the host.
#[derive(Clone)]
pub struct ResourceManager {
    name: AsciiString,
    reference: DataMap<AsciiString, AsciiString>,
    user: DataMap<AsciiString, AsciiString>,
    extended: DataMap<AsciiString, ExtendedString>,
    verbose: bool,
    initialized: bool,
    host: ResourceHost,
}
impl ResourceManager {
    /// Loads `$CSF_<name>Defaults/<name>` then `$CSF_<name>UserDefaults/<name>` (native
    /// `Resource_Manager(aName, Verbose)`); `ResourceManager::new("App", false, host)?`.
    ///
    /// Quirk preserved: the user file is loaded into the reference map too, so Save never
    /// writes it back. Errors only for the native Standard_OutOfRange of a line that starts
    /// with a NUL byte.
    pub fn new(name: &str, verbose: bool, host: ResourceHost) -> Result<Self, Failure> {
        DEBUG.store(host.environment.is_set("ResourceDebug"), Ordering::Relaxed);
        let verbose = verbose || host.environment.is_set("CSF_ResourceVerbose");
        let mut manager = Self::with_state(name, verbose, host);
        for user in [false, true] {
            let path = Self::resource_path_with(name, user, manager.host.environment.as_ref());
            if !path.is_empty() {
                manager.load(&path)?;
            } else if manager.verbose {
                manager.trace(&variable_not_set(name, user));
            }
        }
        Ok(manager)
    }
    /// Loads `<directory>/<name>` for each nonempty directory (native four-argument
    /// constructor); `ResourceManager::with_directories("App", "defs", "user", false, host)?`.
    pub fn with_directories(
        name: &str,
        defaults: &str,
        user_defaults: &str,
        verbose: bool,
        host: ResourceHost,
    ) -> Result<Self, Failure> {
        let mut manager = Self::with_state(name, verbose, host);
        for (directory, label) in [
            (defaults, "aDefaultsDirectory"),
            (user_defaults, "anUserDefaultsDirectory"),
        ] {
            let directory = c_string(directory);
            if !directory.is_empty() {
                manager.load(&paths::resource_file(directory.as_bytes(), name))?;
            } else if manager.verbose {
                let text = format!("Resource Manager Warning: {label} is empty.\n");
                manager.trace(text.as_bytes());
            }
        }
        Ok(manager)
    }
    /// An empty manager named "" (native default constructor); `ResourceManager::empty(host)`.
    pub fn empty(host: ResourceHost) -> Self {
        Self::with_state("", false, host)
    }
    fn with_state(name: &str, verbose: bool, host: ResourceHost) -> Self {
        Self {
            name: c_string(name),
            reference: DataMap::default(),
            user: DataMap::default(),
            extended: DataMap::default(),
            verbose,
            initialized: false,
            host,
        }
    }
    /// Reports whether a resource file was opened, even an empty one; `manager.is_initialized()`.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    /// Native `Load`, always into the reference map (both constructors pass `myRefMap`).
    fn load(&mut self, path: &AsciiString) -> Result<(), Failure> {
        let file_name = crate::osd::CadPath::new(path.as_bytes()).name().clone();
        let Ok(bytes) = self.host.io.read(&paths::host_path(path)) else {
            self.verbose_trace(&[
                b"Resource Manager Warning: Cannot read file \"",
                file_name.as_bytes(),
                b"\". File not found or permission denied.\n",
            ]);
            return Ok(());
        };
        self.initialized = true;
        self.read_resources(&bytes, &file_name)?;
        // Quirk preserved: `&aMap == &myUserMap` never holds, so the label is always Reference.
        self.verbose_trace(&[
            b"Resource Manager: Reference file \"",
            file_name.as_bytes(),
            b"\" loaded\n",
        ]);
        Ok(())
    }
    /// The `WhatKindOfLine` loop of `Load`, tracing keys and values when `ResourceDebug` is set.
    fn read_resources(&mut self, bytes: &[u8], file_name: &AsciiString) -> Result<(), Failure> {
        let (debug, console) = (DEBUG.load(Ordering::Relaxed), self.host.console.clone());
        let mut trace = |text: &[u8]| {
            if debug {
                console.write(text);
            }
        };
        let mut reader = lines::LineReader::new(bytes);
        let mut line_number = 1;
        while let Some(line) = reader.next_line()? {
            self.store_line(lines::line_kind(&line, &mut trace), line_number, file_name);
            line_number += 1;
        }
        Ok(())
    }
    fn store_line(&mut self, kind: lines::LineKind, line_number: i32, file_name: &AsciiString) {
        match kind {
            lines::LineKind::Resource(key, value) => {
                self.reference.bind(key, value);
            }
            lines::LineKind::Error if self.verbose => {
                let text =
                    format!("Resource Manager: Syntax error at line {line_number} in file : ");
                self.trace(&[text.as_bytes(), file_name.as_bytes(), b"\n"].concat());
            }
            _ => {}
        }
    }
    fn trace(&self, bytes: &[u8]) {
        self.host.console.write(bytes);
    }
    /// Writes the concatenated parts when the manager is verbose.
    fn verbose_trace(&self, parts: &[&[u8]]) {
        if self.verbose {
            self.trace(&parts.concat());
        }
    }
}
/// A `const char*` argument: the text up to its first NUL.
fn c_string(text: &str) -> AsciiString {
    AsciiString::from_raw(text.split('\0').next().unwrap_or_default().as_bytes())
}
fn variable_not_set(name: &str, user: bool) -> Vec<u8> {
    let kind = if user { "UserDefaults" } else { "Defaults" };
    let name = c_string(name);
    let prefix = b"Resource Manager Warning: Environment variable \"CSF_";
    [
        prefix.as_slice(),
        name.as_bytes(),
        kind.as_bytes(),
        b"\" not set.\n",
    ]
    .concat()
}
