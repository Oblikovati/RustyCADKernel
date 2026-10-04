//! Reusable `FSD_CmpFile` virtual bodies for custom dialects.
use super::super::TextFile;
use super::TextDialect;
use crate::standard::Failure;
use crate::storage::DriverPrimitives;
use crate::tcollection::{AsciiString, ExtendedString};

impl<D: TextDialect> TextFile<D> {
    /// Calls qualified Cmp header writing without redispatching the dialect hook.
    /// Example: `file.write_cmp_object_header_base(4, 2)`.
    pub fn write_cmp_object_header_base(
        &mut self,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure> {
        write_object_header(self, reference, type_number)
    }

    /// Calls qualified Cmp object opening with native error context.
    /// Example: `file.begin_cmp_write_object_base("FSD_File::BeginWriteObjectData")`.
    pub fn begin_cmp_write_object_base(&mut self, member: &str) -> Result<(), Failure> {
        begin_write_object(self, member)
    }

    /// Calls qualified Cmp object closing with native error context.
    /// Example: `file.end_cmp_write_object_base(false, "FSD_File::EndWriteObjectData")`.
    pub fn end_cmp_write_object_base(&mut self, last: bool, member: &str) -> Result<(), Failure> {
        end_write_object(self, last, member)
    }

    /// Calls qualified Cmp header parsing without redispatching the dialect hook.
    /// Example: `file.read_cmp_object_header_base(&mut reference, &mut type_number)`.
    pub fn read_cmp_object_header_base(
        &mut self,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        read_object_header(self, reference, type_number)
    }

    /// Calls the qualified Cmp no-op for either object-data begin hook.
    /// Example: `file.begin_cmp_read_object_base()`.
    pub fn begin_cmp_read_object_base(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    /// Calls qualified Cmp object closing; persistent data validates its line ending.
    /// Example: `file.end_cmp_read_object_base(true)`.
    pub fn end_cmp_read_object_base(&mut self, last: bool) -> Result<(), Failure> {
        end_read_object(self, last)
    }

    /// Calls qualified Cmp `ReadLine` behavior without redispatching this hook.
    /// Example: `file.read_cmp_line_base()`.
    pub fn read_cmp_line_base(&mut self) -> AsciiString {
        read_line(self)
    }

    /// Calls qualified Cmp `ReadString` behavior without redispatching this hook.
    /// Example: `file.read_cmp_string_base()`.
    pub fn read_cmp_string_base(&mut self) -> AsciiString {
        read_string(self)
    }

    /// Calls qualified Cmp extended-line reading and retains virtual flush dispatch.
    /// Example: `file.read_cmp_extended_line_base(&mut text)`.
    pub fn read_cmp_extended_line_base(
        &mut self,
        text: &mut ExtendedString,
    ) -> Result<(), Failure> {
        read_extended_line(self, text)
    }

    /// Calls qualified Cmp extended-line writing without redispatching this hook.
    /// Example: `file.write_cmp_extended_line_base(&text)`.
    pub fn write_cmp_extended_line_base(&mut self, text: &ExtendedString) -> Result<(), Failure> {
        write_extended_line(self, text)
    }
}

/// Reads through the base FSD reader, then removes native CR/LF suffix bytes.
pub(super) fn read_line<D: TextDialect>(file: &mut TextFile<D>) -> AsciiString {
    trim_line_end(file.fsd_read_line())
}

/// Reads a string through FSD behavior and removes native CR/LF suffix bytes.
pub(super) fn read_string<D: TextDialect>(file: &mut TextFile<D>) -> AsciiString {
    trim_line_end(file.fsd_read_string())
}

/// Applies the native Cmp count loop and dispatches the virtual flush hook.
pub(super) fn read_extended_line<D: TextDialect>(
    file: &mut TextFile<D>,
    text: &mut ExtendedString,
) -> Result<(), Failure> {
    let mut count = 0;
    file.get_integer(&mut count)?;
    // Native Cmp reads this count but loops to the mutable output length instead.
    read_cmp_units(file, text)?;
    file.flush_end_of_line();
    Ok(())
}

/// Writes the Cmp count, UTF-16 units, and line ending.
pub(super) fn write_extended_line<D: TextDialect>(
    file: &mut TextFile<D>,
    text: &ExtendedString,
) -> Result<(), Failure> {
    file.put_integer(text.length())?;
    for unit in text.as_utf16() {
        file.put_ext_character(*unit)?;
    }
    file.stream().insert_bytes(b"\n");
    Ok(())
}

/// Writes the Cmp header delimiter and reports a bad stream like the native method.
pub(super) fn write_object_header<D: TextDialect>(
    file: &mut TextFile<D>,
    reference: i32,
    type_number: i32,
) -> Result<(), Failure> {
    file.stream().insert_bytes(b"\n#");
    file.stream().insert_integer(i64::from(reference));
    file.stream().insert_bytes(b"%");
    file.stream().insert_integer(i64::from(type_number));
    file.stream().insert_bytes(b" ");
    file.check_written("FSD_CmpFile::WritePersistentObjectHeader")
}

/// Retains Cmp's empty write body while still checking the stream state.
pub(super) fn begin_write_object<D: TextDialect>(
    file: &mut TextFile<D>,
    member: &str,
) -> Result<(), Failure> {
    file.check_written(&cmp_member(member))
}

/// Retains Cmp's empty write body while still checking the stream state.
pub(super) fn end_write_object<D: TextDialect>(
    file: &mut TextFile<D>,
    _last: bool,
    member: &str,
) -> Result<(), Failure> {
    file.check_written(&cmp_member(member))
}

/// Parses Cmp's `#reference%type` header with native error identity.
pub(super) fn read_object_header<D: TextDialect>(
    file: &mut TextFile<D>,
    reference: &mut i32,
    type_number: &mut i32,
) -> Result<(), Failure> {
    file.expect_marker(b'#')?;
    file.extract_integer(reference, "FSD_CmpFile::ReadPersistentObjectHeader")?;
    file.expect_marker(b'%')?;
    file.extract_integer(type_number, "FSD_CmpFile::ReadPersistentObjectHeader")
}

/// Cmp object-data begin/end are no-ops; persistent end accepts CR/LF line endings.
pub(super) fn end_read_object<D: TextDialect>(
    file: &mut TextFile<D>,
    last: bool,
) -> Result<(), Failure> {
    if last {
        file.expect_line_end(true)?;
    }
    Ok(())
}

/// Maps shared trait member names to their concrete Cmp override for error details.
fn cmp_member(member: &str) -> String {
    let suffix = member.strip_prefix("FSD_File::").unwrap_or(member);
    format!("FSD_CmpFile::{suffix}")
}

/// Preserves the native loop bound while appending each unit as decimal digits.
fn read_cmp_units<D: TextDialect>(
    file: &mut TextFile<D>,
    text: &mut ExtendedString,
) -> Result<(), Failure> {
    let mut unit = 0_u16;
    let mut index = 0;
    while index < text.length() {
        file.get_ext_character(&mut unit)?;
        // Native `buffer += c` binds to the integer overload, so the unit becomes decimal text.
        text.assign_cat(i32::from(unit));
        index += 1;
    }
    Ok(())
}

/// Removes the carriage-return and line-feed suffix bytes native Cmp methods strip.
fn trim_line_end(text: AsciiString) -> AsciiString {
    let bytes = text.as_bytes();
    let keep = bytes
        .iter()
        .rposition(|byte| *byte != b'\r' && *byte != b'\n')
        .map_or(0, |last| last + 1);
    AsciiString::from_raw(&bytes[..keep])
}
