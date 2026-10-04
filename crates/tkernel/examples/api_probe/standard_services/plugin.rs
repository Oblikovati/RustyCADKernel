//! Rust side of the Plugin::Load case (tkernel_standard_services_plugin.cxx).
use crate::common::{guarded, hex, next, next_bytes};
use rustycad_tkernel::plugin::{PluginLoader, StaticPluginLibraries};
use rustycad_tkernel::resource::{ResourceEnvironment, ResourceIo};
use rustycad_tkernel::standard::{Guid, Handle, StandardTransient};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str::SplitWhitespace;
use std::sync::Arc;
use std::sync::Mutex;

const DIRECTORY: &str = "/probe/plugin";

/// The one resource file the native probe writes.
struct ProbeResources {
    file: Vec<u8>,
}
impl ResourceIo for ProbeResources {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        if path == Path::new(DIRECTORY).join("Plugin") {
            return Ok(self.file.clone());
        }
        Err(io::Error::from(io::ErrorKind::NotFound))
    }
    fn create_directories(&self, _: &Path) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::PermissionDenied))
    }
    fn open_write(&self, _: &Path) -> io::Result<Box<dyn Write>> {
        Err(io::Error::from(io::ErrorKind::PermissionDenied))
    }
}

/// CSF_PluginDefaults names the probe directory; CSF_PluginUserDefaults is unset.
struct ProbeEnvironment;
impl ResourceEnvironment for ProbeEnvironment {
    fn directory(&self, name: &str) -> Option<PathBuf> {
        (name == "CSF_PluginDefaults").then(|| PathBuf::from(DIRECTORY))
    }
}

/// Captures what the loader prints.
#[derive(Clone, Default)]
struct CapturedConsole(Arc<Mutex<Vec<u8>>>);
impl Write for CapturedConsole {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// "services_plugin <guid> <hex resource file> <verbose>".
pub fn plugin_case(input: &mut SplitWhitespace<'_>) -> String {
    let guid: Guid = input.next().expect("guid").parse().expect("GUID text");
    let file = next_bytes(input);
    let verbose = next::<i32>(input) != 0;
    let console = CapturedConsole::default();
    let libraries: StaticPluginLibraries<Handle<StandardTransient>> =
        StaticPluginLibraries::default();
    let mut loader = PluginLoader::new(
        Arc::new(ProbeResources { file }),
        Arc::new(ProbeEnvironment),
        Box::new(libraries),
        Box::new(console.clone()),
    );
    let result = guarded(
        loader
            .load(&guid, verbose)
            .map(|product| if product.is_null() { "null" } else { "product" }.to_owned()),
    );
    let printed = console.0.lock().unwrap().clone();
    format!("{} {}", hex(result.as_bytes()), hex(&printed))
}
