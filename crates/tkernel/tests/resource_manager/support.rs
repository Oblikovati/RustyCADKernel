use rustycad_tkernel::resource::{ResourceConsole, ResourceEnvironment, ResourceHost, ResourceIo};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
    path::{Path, PathBuf},
    rc::Rc,
};
#[derive(Default)]
pub struct FakeResourceSystem {
    pub files: Rc<RefCell<BTreeMap<PathBuf, Vec<u8>>>>,
    pub directories: RefCell<BTreeSet<PathBuf>>,
    environment: RefCell<BTreeMap<String, PathBuf>>,
    pub console: RefCell<Vec<u8>>,
}
impl FakeResourceSystem {
    pub fn host(self: &Rc<Self>) -> ResourceHost {
        ResourceHost::new(self.clone(), self.clone(), self.clone())
    }
    pub fn write_resource(&self, directory: &Path, name: &str, key: &str, value: &str) {
        self.add_directory(directory.to_path_buf());
        self.files.borrow_mut().insert(
            directory.join(name),
            format!("{key} : {value}\n").into_bytes(),
        );
    }
    pub fn add_directory(&self, directory: PathBuf) {
        self.directories.borrow_mut().insert(directory);
    }
    pub fn set_environment(&self, name: &str, value: PathBuf) {
        self.environment.borrow_mut().insert(name.into(), value);
    }
    pub fn has_file(&self, path: &Path) -> bool {
        self.files.borrow().contains_key(path)
    }
}
struct FakeResourceWriter {
    files: Rc<RefCell<BTreeMap<PathBuf, Vec<u8>>>>,
    path: PathBuf,
}
impl Write for FakeResourceWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.files
            .borrow_mut()
            .get_mut(&self.path)
            .unwrap()
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl ResourceEnvironment for FakeResourceSystem {
    fn directory(&self, name: &str) -> Option<PathBuf> {
        self.environment.borrow().get(name).cloned()
    }
}
impl ResourceIo for FakeResourceSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }
    fn create_directories(&self, path: &Path) -> io::Result<()> {
        for ancestor in path.ancestors() {
            self.directories.borrow_mut().insert(ancestor.to_path_buf());
        }
        Ok(())
    }
    fn open_write(&self, path: &Path) -> io::Result<Box<dyn Write>> {
        self.files
            .borrow_mut()
            .insert(path.to_path_buf(), Vec::new());
        Ok(Box::new(FakeResourceWriter {
            files: self.files.clone(),
            path: path.to_path_buf(),
        }))
    }
}
impl ResourceConsole for FakeResourceSystem {
    fn write(&self, bytes: &[u8]) {
        self.console.borrow_mut().extend_from_slice(bytes);
    }
}
