//! `FSD_BinaryFile`: the binary container driver over a C `FILE*`.
//!
//! Scalars are written in native byte order: OCCT only reverses them on SOLARIS and IRIX
//! (`OCCT_BINARY_FILE_DO_INVERSE`), so a little-endian host writes little-endian files.
//! Operations on a driver that is not open dereference a null or closed `FILE*` natively;
//! Rust reports `Standard_NullObject` instead.
use super::{CFile, DriverFileSystem, FileHeader, FileOpenMode};
use crate::standard::Failure;
use crate::storage::{BaseDriver, DriverBase, OpenMode, Position, StorageError, StreamException};
use crate::tcollection::AsciiString;
use std::rc::Rc;
mod complete_info;
mod inverse;
mod scalars;
mod sections;
mod stream_io;
mod strings;
mod tables;

const MAGIC_NUMBER: &str = "BINFILE";

/// The binary storage driver.
///
/// ```
/// use rustycad_tkernel::fsd::{BinaryFile, MemoryFileSystem};
/// use rustycad_tkernel::storage::{BaseDriver, DriverPrimitives, OpenMode, StorageError};
/// use rustycad_tkernel::tcollection::AsciiString;
/// use std::rc::Rc;
/// let files = MemoryFileSystem::default();
/// let name = AsciiString::new("doc.bin");
/// let mut driver = BinaryFile::new(Rc::new(files.clone()));
/// assert_eq!(driver.open(&name, OpenMode::Write).unwrap(), StorageError::Ok);
/// driver.put_integer(7).unwrap();
/// driver.close().unwrap();
/// assert_eq!(files.contents(&name).unwrap(), 7_i32.to_ne_bytes());
/// ```
pub struct BinaryFile {
    base: DriverBase,
    file_system: Rc<dyn DriverFileSystem>,
    stream: Option<CFile>,
    header: FileHeader,
}

impl BinaryFile {
    /// `FSD_BinaryFile()`: not open, every header offset at -1; `BinaryFile::new(file_system)`.
    pub fn new(file_system: Rc<dyn DriverFileSystem>) -> Self {
        Self {
            base: DriverBase::default(),
            file_system,
            stream: None,
            header: FileHeader::unset(),
        }
    }

    /// `MagicNumber`: `"BINFILE"`; `BinaryFile::magic_number()`.
    pub fn magic_number() -> &'static str {
        MAGIC_NUMBER
    }

    /// `IsGoodFileType`: opens `name` for reading and compares its first seven characters
    /// with the magic number; `BinaryFile::is_good_file_type(file_system, &name)`.
    pub fn is_good_file_type(
        file_system: Rc<dyn DriverFileSystem>,
        name: &AsciiString,
    ) -> Result<StorageError, Failure> {
        let mut file = Self::new(file_system);
        let status = file.open(name, OpenMode::Read)?;
        if status != StorageError::Ok {
            return Ok(status);
        }
        let magic = file.read_char(MAGIC_NUMBER.len())?;
        file.close()?;
        Ok(magic_status(&magic, MAGIC_NUMBER))
    }

    /// `Destroy`: closes an open driver, as the destructor does; `driver.destroy()`.
    pub fn destroy(&mut self) {
        if self.base.open_mode() != OpenMode::None {
            let _ = self.close();
        }
    }

    /// The section offsets (`myHeader`), as written or read so far; `driver.file_header()`.
    pub fn file_header(&self) -> FileHeader {
        self.header
    }

    /// The open `FILE*`, or `Standard_NullObject` for a driver that is not open.
    fn file(&mut self) -> Result<&mut CFile, Failure> {
        self.stream.as_mut().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "FSD_BinaryFile: no open FILE; expected Open before any I/O",
                "",
            )
        })
    }

    /// Protected `ReadChar`: up to `size` single-byte `fread`s while not at end of file.
    /// A failed read appends the previous byte again, as the native loop does.
    pub(crate) fn read_char(&mut self, size: usize) -> Result<AsciiString, Failure> {
        let mut buffer = AsciiString::default();
        let mut byte = [0_u8];
        let mut count = 0;
        while !self.file()?.is_end_of_file() && count < size {
            count += self.file()?.read(&mut byte, 1);
            buffer.assign_cat(byte[0]);
        }
        Ok(buffer)
    }

    fn seek_section(&mut self, offset: i32) -> Result<StorageError, Failure> {
        Ok(if self.file()?.seek(i64::from(offset)) == 0 {
            StorageError::Ok
        } else {
            StorageError::SectionNotFound
        })
    }

    /// `(int)ftell(myStream)`.
    fn position(&mut self) -> Result<i32, Failure> {
        Ok(self.file()?.tell() as i32)
    }
}

/// `strncmp(magic, read, len) != 0` gives `Storage_VSFormatError`.
pub(crate) fn magic_status(read: &AsciiString, magic: &str) -> StorageError {
    let expected = magic.as_bytes();
    let actual = read.as_bytes();
    let prefix = &actual[..actual.len().min(expected.len())];
    if prefix == expected {
        StorageError::Ok
    } else {
        StorageError::FormatError
    }
}

pub(crate) fn write_error(member: &str) -> Failure {
    StreamException::Write.raise(&format!("{member}: the file refused the bytes"))
}

pub(crate) fn type_mismatch(member: &str) -> Failure {
    StreamException::TypeMismatch.raise(&format!("{member}: expected more bytes in the file"))
}

impl Drop for BinaryFile {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl BaseDriver for BinaryFile {
    fn driver_base(&self) -> &DriverBase {
        &self.base
    }

    fn driver_base_mut(&mut self) -> &mut DriverBase {
        &mut self.base
    }

    /// `Open`: `rb`, `wb` or `w+b`; an already open driver is `AlreadyOpen`.
    fn open(&mut self, name: &AsciiString, mode: OpenMode) -> Result<StorageError, Failure> {
        self.base.set_name(name);
        if self.base.open_mode() != OpenMode::None {
            return Ok(StorageError::AlreadyOpen);
        }
        let file_mode = match mode {
            OpenMode::Read => FileOpenMode::READ,
            OpenMode::Write => FileOpenMode::WRITE,
            OpenMode::ReadWrite => FileOpenMode::CREATE_READ_WRITE,
            OpenMode::None => return Ok(StorageError::OpenError),
        };
        match self.file_system.open(name, file_mode) {
            Ok(device) => {
                self.stream = Some(CFile::new(device, file_mode.read, file_mode.write));
                self.base.set_open_mode(mode);
                Ok(StorageError::Ok)
            }
            Err(_) => Ok(StorageError::OpenError),
        }
    }

    fn is_end(&mut self) -> Result<bool, Failure> {
        Ok(self.file()?.is_end_of_file())
    }

    fn tell(&mut self) -> Result<Position, Failure> {
        Ok(self.file()?.tell())
    }

    /// `SkipObject` does nothing in the binary format.
    fn skip_object(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    fn close(&mut self) -> Result<StorageError, Failure> {
        if self.base.open_mode() == OpenMode::None {
            return Ok(StorageError::NotOpen);
        }
        if let Some(file) = self.stream.take() {
            file.close();
        }
        self.base.set_open_mode(OpenMode::None);
        Ok(StorageError::Ok)
    }
}
