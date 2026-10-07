//! `Draw::Load`: DRAW plugins.
//!
//! A C++ plugin is a shared library exporting `PLUGINFACTORY(Draw_Interpretor&)`, found
//! through a resource file (`DrawPlugin`) that maps keys to library names. Rust code
//! cannot be loaded that way, so libraries are registered in the session by file name
//! (`libTKTopTest.so`, ...) with their factory, as OCCT's own static build
//! (`OCCT_NO_PLUGINS`) links the plugins in. The resource lookup, the caching by key and
//! the `Draw_Failure` messages (with the `dlerror()` wording of glibc) are kept.
use super::session::DrawSession;
use rustycad_tkernel::message::Message;
use rustycad_tkernel::plugin::library_file_name;
use rustycad_tkernel::resource::{ResourceHost, ResourceManager};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::BTreeMap;
use std::fmt::Write;

/// A plugin's `PLUGINFACTORY`: registers its commands in the session.
pub type DrawPluginFactory = fn(&DrawSession) -> Result<(), Failure>;

/// The plugin libraries linked into this program, by file name.
#[derive(Default)]
pub struct DrawPluginLibraries {
    libraries: BTreeMap<String, Option<DrawPluginFactory>>,
}

impl DrawPluginLibraries {
    /// Registers a library and its factory; `libraries.register("libTKTopTest.so", f)`.
    pub fn register(&mut self, file_name: &str, factory: DrawPluginFactory) {
        self.libraries.insert(file_name.to_string(), Some(factory));
    }

    /// Registers a library that exports no factory.
    pub fn register_without_factory(&mut self, file_name: &str) {
        self.libraries.insert(file_name.to_string(), None);
    }

    /// `DlOpen`: the `dlerror()` text when the library is unknown.
    fn open(&self, file_name: &str) -> Result<(), String> {
        if self.libraries.contains_key(file_name) {
            return Ok(());
        }
        Err(format!(
            "{file_name}: cannot open shared object file: No such file or directory"
        ))
    }

    /// `DlSymb("PLUGINFACTORY")`: the `dlerror()` text when there is no factory.
    fn factory(&self, file_name: &str) -> Result<DrawPluginFactory, String> {
        match self.libraries.get(file_name) {
            Some(Some(factory)) => Ok(*factory),
            _ => Err(format!("{file_name}: undefined symbol: PLUGINFACTORY")),
        }
    }
}

/// A `Draw_Failure` with `message`.
pub fn draw_failure(message: &str) -> Failure {
    Failure::with_message("Draw_Failure", message)
}

impl DrawSession {
    /// `Draw::Load(theDI, theKey, theResourceFileName, theDefaultsDirectory,
    /// theUserDefaultsDirectory, theIsVerbose)`: resolves `key` in the resource file to a
    /// library, finds its factory (cached by key) and runs it on this session.
    pub fn load_plugin(
        &self,
        key: &str,
        resource_file: &str,
        defaults_directory: &str,
        user_defaults_directory: &str,
        verbose: bool,
    ) -> Result<(), Failure> {
        let cached = self.loaded_plugins.borrow().get(key).copied();
        let factory = match cached {
            Some(factory) => factory,
            None => {
                let directories = [defaults_directory, user_defaults_directory];
                let factory = self.resolve_plugin(key, resource_file, directories, verbose)?;
                self.loaded_plugins
                    .borrow_mut()
                    .insert(key.to_string(), factory);
                factory
            }
        };
        factory(self)
    }

    /// The resource lookup and library search of `Draw::Load`.
    fn resolve_plugin(
        &self,
        key: &str,
        resource_file: &str,
        directories: [&str; 2],
        verbose: bool,
    ) -> Result<DrawPluginFactory, Failure> {
        let resources = ResourceManager::with_directories(
            resource_file,
            directories[0],
            directories[1],
            verbose,
            ResourceHost::native(),
        )?;
        let Some(library) = resources.find_value(&AsciiString::new(key)) else {
            let _ = write!(Message::send_fail(), "could not find the resource:{key}");
            return Err(draw_failure(&format!(
                "Could not find the resource:{key}\n"
            )));
        };
        let file_name = library_file_name(&String::from_utf8_lossy(library.as_bytes()));
        let libraries = self.plugin_libraries.borrow();
        if let Err(reason) = libraries.open(&file_name) {
            return Err(draw_failure(&format!(
                "Could not open: {file_name}; reason: {reason}"
            )));
        }
        // No separator between the file name and the reason, as upstream.
        libraries.factory(&file_name).map_err(|reason| {
            draw_failure(&format!(
                "Could not find the factory in: {file_name}{reason}"
            ))
        })
    }

    /// The plugin libraries of this program, for registration.
    pub fn plugin_libraries(&self) -> std::cell::RefMut<'_, DrawPluginLibraries> {
        self.plugin_libraries.borrow_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_libraries_fail_with_dlerror_wording() {
        let mut libraries = DrawPluginLibraries::default();
        libraries.register_without_factory("libX.so");
        assert_eq!(
            libraries.open("libY.so").unwrap_err(),
            "libY.so: cannot open shared object file: No such file or directory"
        );
        assert!(libraries.open("libX.so").is_ok());
        assert_eq!(
            libraries.factory("libX.so").unwrap_err(),
            "libX.so: undefined symbol: PLUGINFACTORY"
        );
        assert_eq!(draw_failure("m").to_string(), "Draw_Failure: m");
    }
}
