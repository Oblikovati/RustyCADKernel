//! The virtual members `FSD_CmpFile` overrides in `FSD_File`.
use super::TextFile;
use crate::standard::Failure;
use crate::tcollection::{AsciiString, ExtendedString};
mod cmp;

/// The members that differ between `FSD_File` ([`FsdDialect`]) and `FSD_CmpFile`
/// ([`CmpDialect`]); every other member of [`TextFile`] is shared.
pub trait TextDialect: Sized {
    /// `MagicNumber()`.
    const MAGIC_NUMBER: &'static str;

    /// Selects the four `FSD_CmpFile` text bodies for hooks this dialect does not override.
    ///
    /// ```
    /// use rustycad_tkernel::fsd::TextDialect;
    /// struct DerivedCmp;
    /// impl TextDialect for DerivedCmp {
    ///     const MAGIC_NUMBER: &'static str = "CUSTOM";
    ///     const USE_CMP_TEXT_DEFAULTS: bool = true;
    /// }
    /// ```
    const USE_CMP_TEXT_DEFAULTS: bool = false;

    /// Protected virtual `ReadLine`; `MyDialect::read_line(file)`.
    fn read_line(file: &mut TextFile<Self>) -> AsciiString {
        if Self::USE_CMP_TEXT_DEFAULTS {
            cmp::read_line(file)
        } else {
            file.fsd_read_line()
        }
    }

    /// Protected virtual `ReadString`; `MyDialect::read_string(file)`.
    fn read_string(file: &mut TextFile<Self>) -> AsciiString {
        if Self::USE_CMP_TEXT_DEFAULTS {
            cmp::read_string(file)
        } else {
            file.fsd_read_string()
        }
    }

    /// Protected virtual `ReadChar`; `MyDialect::read_char(file, 4)` reads up to four bytes.
    fn read_char(file: &mut TextFile<Self>, size: usize) -> AsciiString {
        file.fsd_read_char(size)
    }

    /// Protected virtual `ReadWord`; `MyDialect::read_word(file)` reads the next word.
    fn read_word(file: &mut TextFile<Self>) -> AsciiString {
        file.fsd_read_word()
    }

    /// Protected virtual `ReadExtendedLine`.
    fn read_extended_line(
        file: &mut TextFile<Self>,
        text: &mut ExtendedString,
    ) -> Result<(), Failure> {
        if Self::USE_CMP_TEXT_DEFAULTS {
            cmp::read_extended_line(file, text)
        } else {
            file.fsd_read_extended_line(text)
        }
    }

    /// Protected virtual `WriteExtendedLine`.
    fn write_extended_line(
        file: &mut TextFile<Self>,
        text: &ExtendedString,
    ) -> Result<(), Failure> {
        if Self::USE_CMP_TEXT_DEFAULTS {
            cmp::write_extended_line(file, text)
        } else {
            file.fsd_write_extended_line(text);
            Ok(())
        }
    }

    /// Protected virtual `FlushEndOfLine`; `MyDialect::flush_end_of_line(file)`.
    fn flush_end_of_line(file: &mut TextFile<Self>) {
        Self::read_line(file);
    }

    /// Protected virtual `FindTag`; `MyDialect::find_tag(file, "BEGIN_INFO_SECTION")`.
    fn find_tag(file: &mut TextFile<Self>, tag: &str) -> crate::storage::StorageError {
        file.find_tag_c_string(tag.as_bytes())
    }

    /// `WritePersistentObjectHeader`.
    fn write_object_header(
        file: &mut TextFile<Self>,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure> {
        file.stream.insert_bytes(b"\n#");
        file.stream.insert_integer(i64::from(reference));
        file.stream.insert_bytes(b"=%");
        file.stream.insert_integer(i64::from(type_number));
        file.check_written("FSD_File::WritePersistentObjectHeader")
    }

    /// `BeginWritePersistentObjectData` and `BeginWriteObjectData` (the same text).
    fn begin_write_object(file: &mut TextFile<Self>, member: &str) -> Result<(), Failure> {
        file.stream.insert_bytes(b"( ");
        file.check_written(member)
    }

    /// `EndWriteObjectData` (`last` false) and `EndWritePersistentObjectData` (`last` true).
    fn end_write_object(
        file: &mut TextFile<Self>,
        last: bool,
        member: &str,
    ) -> Result<(), Failure> {
        file.stream.insert_bytes(if last { b")" } else { b") " });
        file.check_written(member)
    }

    /// `ReadPersistentObjectHeader`.
    fn read_object_header(
        file: &mut TextFile<Self>,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        file.expect_marker(b'#')?;
        file.extract_integer(reference, "FSD_File::ReadPersistentObjectHeader")?;
        file.expect_marker(b'=')?;
        file.expect_marker(b'%')?;
        file.extract_integer(type_number, "FSD_File::ReadPersistentObjectHeader")
    }

    /// `BeginReadPersistentObjectData` and `BeginReadObjectData`.
    fn begin_read_object(file: &mut TextFile<Self>) -> Result<(), Failure> {
        file.expect_marker(b'(')
    }

    /// `EndReadObjectData` (`last` false) and `EndReadPersistentObjectData` (`last` true).
    fn end_read_object(file: &mut TextFile<Self>, last: bool) -> Result<(), Failure> {
        file.expect_marker(b')')?;
        if last {
            file.expect_line_end(false)?;
        }
        Ok(())
    }
}

/// `FSD_File`'s own members.
#[derive(Clone, Copy, Debug, Default)]
pub struct FsdDialect;

impl TextDialect for FsdDialect {
    const MAGIC_NUMBER: &'static str = "FSDFILE";
}

/// `FSD_CmpFile`'s defaults: carriage returns are tolerated at line ends, extended lines
/// are written as a count and `PutExtCharacter` values, and objects are framed by their
/// header alone.
#[derive(Clone, Copy, Debug, Default)]
pub struct CmpDialect;

impl TextDialect for CmpDialect {
    const MAGIC_NUMBER: &'static str = "CMPFILE";
    const USE_CMP_TEXT_DEFAULTS: bool = true;

    fn write_object_header(
        file: &mut TextFile<Self>,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure> {
        file.stream.insert_bytes(b"\n#");
        file.stream.insert_integer(i64::from(reference));
        file.stream.insert_bytes(b"%");
        file.stream.insert_integer(i64::from(type_number));
        file.stream.insert_bytes(b" ");
        file.check_written("FSD_CmpFile::WritePersistentObjectHeader")
    }

    fn begin_write_object(file: &mut TextFile<Self>, member: &str) -> Result<(), Failure> {
        file.check_written(member)
    }

    fn end_write_object(
        file: &mut TextFile<Self>,
        _last: bool,
        member: &str,
    ) -> Result<(), Failure> {
        file.check_written(member)
    }

    fn read_object_header(
        file: &mut TextFile<Self>,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        file.expect_marker(b'#')?;
        file.extract_integer(reference, "FSD_CmpFile::ReadPersistentObjectHeader")?;
        file.expect_marker(b'%')?;
        file.extract_integer(type_number, "FSD_CmpFile::ReadPersistentObjectHeader")
    }

    fn begin_read_object(_file: &mut TextFile<Self>) -> Result<(), Failure> {
        Ok(())
    }

    fn end_read_object(file: &mut TextFile<Self>, last: bool) -> Result<(), Failure> {
        if last {
            file.expect_line_end(true)?;
        }
        Ok(())
    }
}
