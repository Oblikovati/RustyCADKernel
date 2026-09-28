//! Plugin::Load through injected resources, libraries and console.
use rustycad_tkernel::plugin::{PluginLoader, StaticPluginLibraries, library_file_name};
use rustycad_tkernel::resource::{ResourceEnvironment, ResourceIo};
use rustycad_tkernel::standard::Guid;
use std::cell::{Cell, RefCell};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const GUID: &str = "f69e10e5-9a4e-11d4-8b8c-0060b0ee281b";

/// A "Plugin" resource file in memory that counts how often it is read.
struct FakePluginResources {
    file: String,
    reads: Cell<usize>,
}
impl ResourceIo for FakePluginResources {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.reads.set(self.reads.get() + 1);
        if path == Path::new("/fake/Plugin") {
            return Ok(self.file.clone().into_bytes());
        }
        Err(io::Error::from(io::ErrorKind::NotFound))
    }
    fn create_directories(&self, _: &Path) -> io::Result<()> {
        Ok(())
    }
    fn open_write(&self, _: &Path) -> io::Result<Box<dyn Write>> {
        Err(io::Error::from(io::ErrorKind::PermissionDenied))
    }
}

struct FakePluginEnvironment;
impl ResourceEnvironment for FakePluginEnvironment {
    fn directory(&self, name: &str) -> Option<PathBuf> {
        (name == "CSF_PluginDefaults").then(|| PathBuf::from("/fake"))
    }
}

#[derive(Clone, Default)]
struct FakeConsole(Rc<RefCell<Vec<u8>>>);
impl Write for FakeConsole {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn plugin_loader(
    file: &str,
    libraries: StaticPluginLibraries<String>,
) -> (PluginLoader<String>, Rc<FakePluginResources>, FakeConsole) {
    let resources = Rc::new(FakePluginResources {
        file: file.to_owned(),
        reads: Cell::new(0),
    });
    let console = FakeConsole::default();
    let loader = PluginLoader::new(
        resources.clone(),
        Rc::new(FakePluginEnvironment),
        Box::new(libraries),
        Box::new(console.clone()),
    );
    (loader, resources, console)
}

#[test]
fn a_registered_factory_is_resolved_once_and_called_every_time() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let mut libraries = StaticPluginLibraries::default();
    libraries.register(
        &library_file_name("TKDemo"),
        Arc::new(move |guid: &Guid| {
            counter.fetch_add(1, Ordering::SeqCst);
            format!("driver for {guid}")
        }),
    );
    let (mut loader, resources, _) =
        plugin_loader(&format!("{GUID}.Location: TKDemo\n"), libraries);
    let guid: Guid = GUID.parse().unwrap();
    assert_eq!(
        loader.load(&guid, true).unwrap(),
        format!("driver for {GUID}")
    );
    let reads_after_first = resources.reads.get();
    loader.load(&guid, true).unwrap();
    assert_eq!(
        resources.reads.get(),
        reads_after_first,
        "the factory is cached"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn a_missing_resource_is_reported_and_printed_when_verbose() {
    let (mut loader, _, console) = plugin_loader("", StaticPluginLibraries::default());
    let guid: Guid = GUID.parse().unwrap();
    let failure = loader.load(&guid, false).unwrap_err();
    assert_eq!(failure.exception_type(), "Plugin_Failure");
    assert_eq!(
        failure.message(),
        format!("could not find the resource:{GUID}.Location\n")
    );
    assert!(console.0.borrow().is_empty());
    loader.load(&guid, true).unwrap_err();
    assert_eq!(
        console.0.borrow().as_slice(),
        format!("could not find the resource:{GUID}.Location\n").as_bytes()
    );
}

#[test]
fn libraries_that_do_not_open_or_lack_a_factory_fail_without_caching() {
    let mut libraries = StaticPluginLibraries::default();
    libraries.register_without_factory(&library_file_name("TKEmpty"));
    let file = format!("{GUID}.Location: TKEmpty\n");
    let (mut loader, resources, console) = plugin_loader(&file, libraries);
    let guid: Guid = GUID.parse().unwrap();
    let failure = loader.load(&guid, true).unwrap_err();
    let file_name = library_file_name("TKEmpty");
    assert_eq!(
        failure.message(),
        format!(
            "could not find the factory in:TKEmpty{file_name}: undefined symbol: PLUGINFACTORY"
        )
    );
    assert!(
        console.0.borrow().is_empty(),
        "the factory failure is not printed"
    );
    let reads = resources.reads.get();
    loader.load(&guid, false).unwrap_err();
    assert!(
        resources.reads.get() > reads,
        "failures are looked up again"
    );
    let (mut missing, _, _) = plugin_loader(
        &format!("{GUID}.Location: TKMissing\n"),
        StaticPluginLibraries::default(),
    );
    let failure = missing.load(&guid, false).unwrap_err();
    let expected = format!(
        "could not open:TKMissing; reason:{}: cannot open shared object file: No such file or directory",
        library_file_name("TKMissing")
    );
    assert_eq!(failure.message(), expected);
}

#[test]
fn library_names_follow_the_platform_convention() {
    let name = library_file_name("TKXCAF");
    if cfg!(windows) {
        assert_eq!(name, "TKXCAF.dll");
    } else if cfg!(target_vendor = "apple") {
        assert_eq!(name, "libTKXCAF.dylib");
    } else {
        assert_eq!(name, "libTKXCAF.so");
    }
}
