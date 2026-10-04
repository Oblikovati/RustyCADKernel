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
    ///     const USE_CMP_OBJECT_DEFAULTS: bool = true;
    /// }
    /// ```
    const USE_CMP_TEXT_DEFAULTS: bool = false;

    /// Selects `FSD_CmpFile` object-framing bodies for hooks this dialect does not override.
    const USE_CMP_OBJECT_DEFAULTS: bool = false;

    /// Protected virtual `ReadLine`; `MyDialect::read_line(file)`.
    fn read_line(file: &mut TextFile<Self>) -> AsciiString {
        if Self::USE_CMP_TEXT_DEFAULTS {
            cmp::read_line(file)
        } else {
            file.read_fsd_line_base()
        }
    }

    /// Protected virtual `ReadString`; `MyDialect::read_string(file)`.
    fn read_string(file: &mut TextFile<Self>) -> AsciiString {
        if Self::USE_CMP_TEXT_DEFAULTS {
            cmp::read_string(file)
        } else {
            file.read_fsd_string_base()
        }
    }

    /// Protected virtual `ReadChar`; `MyDialect::read_char(file, 4)` reads up to four bytes.
    fn read_char(file: &mut TextFile<Self>, size: usize) -> AsciiString {
        file.read_fsd_char_base(size)
    }

    /// Protected virtual `ReadWord`; `MyDialect::read_word(file)` reads the next word.
    fn read_word(file: &mut TextFile<Self>) -> AsciiString {
        file.read_fsd_word_base()
    }

    /// Protected virtual `ReadExtendedLine`.
    fn read_extended_line(
        file: &mut TextFile<Self>,
        text: &mut ExtendedString,
    ) -> Result<(), Failure> {
        if Self::USE_CMP_TEXT_DEFAULTS {
            cmp::read_extended_line(file, text)
        } else {
            file.read_fsd_extended_line_base(text)
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
            file.write_fsd_extended_line_base(text);
            Ok(())
        }
    }

    /// Protected virtual `FlushEndOfLine`; `MyDialect::flush_end_of_line(file)`.
    fn flush_end_of_line(file: &mut TextFile<Self>) {
        file.flush_fsd_end_of_line_base();
    }

    /// Protected virtual `FindTag`; `MyDialect::find_tag(file, "BEGIN_INFO_SECTION")`.
    fn find_tag(file: &mut TextFile<Self>, tag: &str) -> crate::storage::StorageError {
        file.find_fsd_tag_base(tag)
    }

    /// `WritePersistentObjectHeader`.
    fn write_object_header(
        file: &mut TextFile<Self>,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure> {
        if Self::USE_CMP_OBJECT_DEFAULTS {
            cmp::write_object_header(file, reference, type_number)
        } else {
            file.write_fsd_object_header_base(reference, type_number)
        }
    }

    /// `BeginWritePersistentObjectData` and `BeginWriteObjectData` (the same text).
    fn begin_write_object(file: &mut TextFile<Self>, member: &str) -> Result<(), Failure> {
        if Self::USE_CMP_OBJECT_DEFAULTS {
            file.begin_cmp_write_object_base(member)
        } else {
            file.begin_fsd_write_object_base(member)
        }
    }

    /// `EndWriteObjectData` (`last` false) and `EndWritePersistentObjectData` (`last` true).
    fn end_write_object(
        file: &mut TextFile<Self>,
        last: bool,
        member: &str,
    ) -> Result<(), Failure> {
        if Self::USE_CMP_OBJECT_DEFAULTS {
            file.end_cmp_write_object_base(last, member)
        } else {
            file.end_fsd_write_object_base(last, member)
        }
    }

    /// `ReadPersistentObjectHeader`.
    fn read_object_header(
        file: &mut TextFile<Self>,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        if Self::USE_CMP_OBJECT_DEFAULTS {
            file.read_cmp_object_header_base(reference, type_number)
        } else {
            file.read_fsd_object_header_base(reference, type_number)
        }
    }

    /// `BeginReadPersistentObjectData` and `BeginReadObjectData`.
    fn begin_read_object(file: &mut TextFile<Self>) -> Result<(), Failure> {
        if Self::USE_CMP_OBJECT_DEFAULTS {
            file.begin_cmp_read_object_base()
        } else {
            file.begin_fsd_read_object_base()
        }
    }

    /// `EndReadObjectData` (`last` false) and `EndReadPersistentObjectData` (`last` true).
    fn end_read_object(file: &mut TextFile<Self>, last: bool) -> Result<(), Failure> {
        if Self::USE_CMP_OBJECT_DEFAULTS {
            file.end_cmp_read_object_base(last)
        } else {
            file.end_fsd_read_object_base(last)
        }
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
    const USE_CMP_OBJECT_DEFAULTS: bool = true;
}
