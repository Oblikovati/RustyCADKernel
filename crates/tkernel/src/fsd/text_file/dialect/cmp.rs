//! Reusable `FSD_CmpFile` virtual bodies for custom dialects.
use super::super::TextFile;
use super::TextDialect;
use crate::standard::Failure;
use crate::storage::DriverPrimitives;
use crate::tcollection::{AsciiString, ExtendedString};

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
