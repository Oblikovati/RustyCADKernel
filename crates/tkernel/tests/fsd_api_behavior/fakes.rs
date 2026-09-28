//! Named fake file systems.
use rustycad_tkernel::fsd::{DriverFileSystem, FileOpenMode, MemoryFileSystem};
use rustycad_tkernel::storage::StreamDevice;
use rustycad_tkernel::tcollection::AsciiString;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::rc::Rc;

/// A file system that opens everything but whose devices refuse every write.
pub struct RefusingFileSystem;

struct RefusingDevice(Cursor<Vec<u8>>);

impl Read for RefusingDevice {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.0.read(buffer)
    }
}

impl Write for RefusingDevice {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::StorageFull,
            "the fake disk is full",
        ))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for RefusingDevice {
    fn seek(&mut self, target: SeekFrom) -> io::Result<u64> {
        self.0.seek(target)
    }
}

impl DriverFileSystem for RefusingFileSystem {
    fn open(&self, _: &AsciiString, _: FileOpenMode) -> io::Result<Box<dyn StreamDevice>> {
        Ok(Box::new(RefusingDevice(Cursor::new(Vec::new()))))
    }
}

/// An in-memory file system holding one file named `name`.
pub fn with_file(name: &str, bytes: &[u8]) -> (MemoryFileSystem, Rc<dyn DriverFileSystem>) {
    let files = MemoryFileSystem::default();
    files.insert(&AsciiString::new(name), bytes.to_vec());
    let shared: Rc<dyn DriverFileSystem> = Rc::new(files.clone());
    (files, shared)
}
