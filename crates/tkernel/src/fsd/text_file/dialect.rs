// Copyright (c) 1996-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! The virtual members `FSD_CmpFile` overrides in `FSD_File`.
use super::TextFile;
use crate::standard::Failure;
use crate::storage::DriverPrimitives;
use crate::tcollection::{AsciiString, ExtendedString};

/// The members that differ between `FSD_File` ([`FsdDialect`]) and `FSD_CmpFile`
/// ([`CmpDialect`]); every other member of [`TextFile`] is shared.
pub trait TextDialect: Sized {
    /// `MagicNumber()`.
    const MAGIC_NUMBER: &'static str;

    /// Protected virtual `ReadLine`.
    fn read_line(file: &mut TextFile<Self>) -> AsciiString;

    /// Protected virtual `ReadString`.
    fn read_string(file: &mut TextFile<Self>) -> AsciiString;

    /// Protected virtual `ReadExtendedLine`.
    fn read_extended_line(file: &mut TextFile<Self>, text: &mut ExtendedString) -> Result<(), Failure>;

    /// Protected virtual `WriteExtendedLine`.
    fn write_extended_line(file: &mut TextFile<Self>, text: &ExtendedString) -> Result<(), Failure>;

    /// `WritePersistentObjectHeader`.
    fn write_object_header(file: &mut TextFile<Self>, reference: i32, type_number: i32) -> Result<(), Failure>;

    /// `BeginWritePersistentObjectData` and `BeginWriteObjectData` (the same text).
    fn begin_write_object(file: &mut TextFile<Self>, member: &str) -> Result<(), Failure>;

    /// `EndWriteObjectData` (`last` false) and `EndWritePersistentObjectData` (`last` true).
    fn end_write_object(file: &mut TextFile<Self>, last: bool, member: &str) -> Result<(), Failure>;

    /// `ReadPersistentObjectHeader`.
    fn read_object_header(
        file: &mut TextFile<Self>,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure>;

    /// `BeginReadPersistentObjectData` and `BeginReadObjectData`.
    fn begin_read_object(file: &mut TextFile<Self>) -> Result<(), Failure>;

    /// `EndReadObjectData` (`last` false) and `EndReadPersistentObjectData` (`last` true).
    fn end_read_object(file: &mut TextFile<Self>, last: bool) -> Result<(), Failure>;
}

/// `FSD_File`'s own members.
#[derive(Clone, Copy, Debug, Default)]
pub struct FsdDialect;

impl TextDialect for FsdDialect {
    const MAGIC_NUMBER: &'static str = "FSDFILE";

    fn read_line(file: &mut TextFile<Self>) -> AsciiString {
        file.fsd_read_line()
    }

    fn read_string(file: &mut TextFile<Self>) -> AsciiString {
        file.fsd_read_string()
    }

    fn read_extended_line(file: &mut TextFile<Self>, text: &mut ExtendedString) -> Result<(), Failure> {
        file.fsd_read_extended_line(text)
    }

    fn write_extended_line(file: &mut TextFile<Self>, text: &ExtendedString) -> Result<(), Failure> {
        file.fsd_write_extended_line(text);
        Ok(())
    }

    fn write_object_header(file: &mut TextFile<Self>, reference: i32, type_number: i32) -> Result<(), Failure> {
        file.stream.insert_bytes(b"\n#");
        file.stream.insert_integer(i64::from(reference));
        file.stream.insert_bytes(b"=%");
        file.stream.insert_integer(i64::from(type_number));
        file.check_written("FSD_File::WritePersistentObjectHeader")
    }

    fn begin_write_object(file: &mut TextFile<Self>, member: &str) -> Result<(), Failure> {
        file.stream.insert_bytes(b"( ");
        file.check_written(member)
    }

    fn end_write_object(file: &mut TextFile<Self>, last: bool, member: &str) -> Result<(), Failure> {
        file.stream.insert_bytes(if last { b")" } else { b") " });
        file.check_written(member)
    }

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

    fn begin_read_object(file: &mut TextFile<Self>) -> Result<(), Failure> {
        file.expect_marker(b'(')
    }

    fn end_read_object(file: &mut TextFile<Self>, last: bool) -> Result<(), Failure> {
        file.expect_marker(b')')?;
        if last {
            file.expect_line_end(false)?;
        }
        Ok(())
    }
}

/// `FSD_CmpFile`'s overrides: carriage returns are tolerated at line ends, extended lines
/// are written as a count and `PutExtCharacter` values, and objects are framed by their
/// header alone.
#[derive(Clone, Copy, Debug, Default)]
pub struct CmpDialect;

/// Drops trailing `\r` and `\n` bytes, as `FSD_CmpFile::ReadLine`/`ReadString` do.
fn trim_line_end(mut text: AsciiString) -> AsciiString {
    let keep = text
        .as_bytes()
        .iter()
        .rposition(|byte| *byte != b'\r' && *byte != b'\n')
        .map_or(0, |last| last + 1);
    text.truncate(keep);
    text
}

impl TextDialect for CmpDialect {
    const MAGIC_NUMBER: &'static str = "CMPFILE";

    fn read_line(file: &mut TextFile<Self>) -> AsciiString {
        trim_line_end(file.fsd_read_line())
    }

    fn read_string(file: &mut TextFile<Self>) -> AsciiString {
        trim_line_end(file.fsd_read_string())
    }

    /// Reads the unit count, then one `GetExtCharacter` per unit already in `text`, which
    /// is not cleared: the native loop bound is `buffer.Length()`, so an empty buffer reads
    /// no unit and a non-empty one reads until a zero unit or an exception.
    fn read_extended_line(file: &mut TextFile<Self>, text: &mut ExtendedString) -> Result<(), Failure> {
        let mut count = 0;
        file.get_integer(&mut count)?;
        let mut unit = 0_u16;
        let mut index = 0;
        while index < text.length() {
            file.get_ext_character(&mut unit)?;
            text.assign_cat(unit);
            index += 1;
        }
        file.flush_end_of_line();
        Ok(())
    }

    fn write_extended_line(file: &mut TextFile<Self>, text: &ExtendedString) -> Result<(), Failure> {
        file.put_integer(text.length())?;
        for unit in text.as_utf16() {
            file.put_ext_character(*unit)?;
        }
        file.stream.insert_bytes(b"\n");
        Ok(())
    }

    fn write_object_header(file: &mut TextFile<Self>, reference: i32, type_number: i32) -> Result<(), Failure> {
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

    fn end_write_object(file: &mut TextFile<Self>, _last: bool, member: &str) -> Result<(), Failure> {
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
