// Copyright (c) 1998-1999 Matra Datavision
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

//! The `Standard_IStream`/`Standard_OStream` overloads of `FSD_BinaryFile`, which read
//! and write the binary format through a stream instead of the driver's `FILE*`. The
//! writers return the byte size and, with `only_count`, write nothing.
use super::strings::{bytes_from_units, units_from_bytes};
use super::{BinaryFile, FileHeader, type_mismatch};
use crate::ncollection::Sequence;
use crate::standard::{Failure, Handle};
use crate::storage::{HeaderData, IoStream, StorageError, StreamException};
use crate::tcollection::{AsciiString, ExtendedString};

fn stream_write(stream: &mut IoStream, bytes: &[u8], member: &str) -> Result<(), Failure> {
    stream.write(bytes);
    if stream.fail() {
        return Err(StreamException::Write.raise(&format!(
            "{member}: the stream refused {} bytes",
            bytes.len()
        )));
    }
    Ok(())
}

fn read_error(member: &str) -> Failure {
    StreamException::Read.raise(&format!("{member}: the stream ended inside a string"))
}

/// `if (!good()) throw; read(n); if (gcount() != n) throw` of the string readers. The read
/// goes in steps so a corrupt length fails at the end of the stream instead of allocating
/// the whole length first, as the native `Standard::Allocate` does.
fn stream_block(stream: &mut IoStream, size: usize, member: &str) -> Result<Vec<u8>, Failure> {
    const STEP: usize = 1 << 16;
    if !stream.good() {
        return Err(read_error(member));
    }
    let mut block = Vec::new();
    while block.len() < size {
        let start = block.len();
        let step = (size - start).min(STEP);
        block.resize(start + step, 0);
        stream.read(&mut block[start..]);
        if stream.gcount() != step as i64 {
            return Err(read_error(member));
        }
    }
    Ok(block)
}

impl BinaryFile {
    /// `PutInteger(Standard_OStream&, int, bool)`; `BinaryFile::put_integer_stream(&mut out, 1, false)?`.
    pub fn put_integer_stream(
        stream: &mut IoStream,
        value: i32,
        only_count: bool,
    ) -> Result<i32, Failure> {
        if !only_count {
            stream_write(stream, &value.to_ne_bytes(), "FSD_BinaryFile::PutInteger")?;
        }
        Ok(4)
    }

    /// `GetInteger(Standard_IStream&, int&)`: a short read keeps the bytes it got and is a
    /// `Storage_StreamTypeMismatchError`; `BinaryFile::get_integer_stream(&mut input, &mut v)?`.
    pub fn get_integer_stream(stream: &mut IoStream, value: &mut i32) -> Result<(), Failure> {
        let mut bytes = value.to_ne_bytes();
        stream.read(&mut bytes);
        *value = i32::from_ne_bytes(bytes);
        if stream.gcount() != 4 {
            return Err(type_mismatch("FSD_BinaryFile::GetInteger"));
        }
        Ok(())
    }

    /// `GetReference(Standard_IStream&, int&)`, the same as `GetInteger`.
    /// Example: `BinaryFile::get_reference_stream(&mut input, &mut reference)?`.
    pub fn get_reference_stream(stream: &mut IoStream, value: &mut i32) -> Result<(), Failure> {
        Self::get_integer_stream(stream, value)
    }

    /// Protected static `WriteString`: the length and the bytes, `4 + length` bytes.
    /// Example: `BinaryFile::write_string_stream(&mut out, &text, false)?`.
    pub fn write_string_stream(
        stream: &mut IoStream,
        text: &AsciiString,
        only_count: bool,
    ) -> Result<i32, Failure> {
        let size = Self::put_integer_stream(stream, text.length(), only_count)?;
        if !text.is_empty() && !only_count {
            stream_write(stream, text.as_bytes(), "FSD_BinaryFile::WriteString")?;
        }
        Ok(size + text.length())
    }

    /// `ReadString(Standard_IStream&, TCollection_AsciiString&)`; the text stops at its
    /// first NUL; `BinaryFile::read_string_stream(&mut input, &mut text)?`.
    pub fn read_string_stream(
        stream: &mut IoStream,
        text: &mut AsciiString,
    ) -> Result<(), Failure> {
        let mut size = 0;
        Self::get_integer_stream(stream, &mut size)?;
        if size <= 0 {
            *text = AsciiString::default();
            return Ok(());
        }
        let block = stream_block(stream, size as usize, "FSD_BinaryFile::ReadString")?;
        *text = AsciiString::from_bytes(&block);
        Ok(())
    }

    /// Protected static `WriteExtendedString`: the unit count and native-order units.
    /// Example: `BinaryFile::write_extended_string_stream(&mut out, &text, false)?`.
    pub fn write_extended_string_stream(
        stream: &mut IoStream,
        text: &ExtendedString,
        only_count: bool,
    ) -> Result<i32, Failure> {
        let size = Self::put_integer_stream(stream, text.length(), only_count)?;
        if !text.is_empty() && !only_count {
            let bytes = bytes_from_units(text.as_utf16());
            stream_write(stream, &bytes, "FSD_BinaryFile::WriteExtendedString")?;
        }
        Ok(size + 2 * text.length())
    }

    /// `ReadExtendedString(Standard_IStream&, TCollection_ExtendedString&)`; the text stops
    /// at its first zero unit; `BinaryFile::read_extended_string_stream(&mut input, &mut text)?`.
    pub fn read_extended_string_stream(
        stream: &mut IoStream,
        text: &mut ExtendedString,
    ) -> Result<(), Failure> {
        let mut size = 0;
        Self::get_integer_stream(stream, &mut size)?;
        if size <= 0 {
            *text = ExtendedString::default();
            return Ok(());
        }
        let block = stream_block(
            stream,
            size as usize * 2,
            "FSD_BinaryFile::ReadExtendedString",
        )?;
        *text = ExtendedString::from_utf16(&units_from_bytes(&block));
        Ok(())
    }

    /// `WriteHeader(Standard_OStream&, const FSD_FileHeader&, bool)`: 52 bytes.
    /// Example: `BinaryFile::write_header_stream(&mut out, &header, false)?`.
    pub fn write_header_stream(
        stream: &mut IoStream,
        header: &FileHeader,
        only_count: bool,
    ) -> Result<i32, Failure> {
        let mut size = 0;
        for field in header.fields() {
            size += Self::put_integer_stream(stream, field, only_count)?;
        }
        Ok(size)
    }

    /// `ReadHeader(Standard_IStream&, FSD_FileHeader&)`; a short read stops at that field.
    /// Example: `BinaryFile::read_header_stream(&mut input, &mut header)?`.
    pub fn read_header_stream(
        stream: &mut IoStream,
        header: &mut FileHeader,
    ) -> Result<(), Failure> {
        for field in header.fields_mut() {
            Self::get_integer_stream(stream, field)?;
        }
        Ok(())
    }
}

impl BinaryFile {
    /// `WriteInfo(Standard_OStream&, ...)`: the info section; returns its byte size.
    /// Example: `BinaryFile::write_info_stream(&mut out, 0, &v, &d, &n, &sv, &an, &av, &dt, &info, false)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn write_info_stream(
        stream: &mut IoStream,
        object_count: i32,
        storage_version: &AsciiString,
        date: &AsciiString,
        schema_name: &AsciiString,
        schema_version: &AsciiString,
        application_name: &ExtendedString,
        application_version: &AsciiString,
        data_type: &ExtendedString,
        user_info: &Sequence<AsciiString>,
        only_count: bool,
    ) -> Result<i32, Failure> {
        let mut size = Self::put_integer_stream(stream, object_count, only_count)?;
        for text in [storage_version, date, schema_name, schema_version] {
            size += Self::write_string_stream(stream, text, only_count)?;
        }
        size += Self::write_extended_string_stream(stream, application_name, only_count)?;
        size += Self::write_string_stream(stream, application_version, only_count)?;
        size += Self::write_extended_string_stream(stream, data_type, only_count)?;
        size += Self::put_integer_stream(stream, user_info.length(), only_count)?;
        for line in user_info.iter() {
            size += Self::write_string_stream(stream, line, only_count)?;
        }
        Ok(size)
    }

    /// `WriteComment(Standard_OStream&, ...)`: the comment section; returns its byte size.
    /// Example: `BinaryFile::write_comment_stream(&mut out, &comments, false)?`.
    pub fn write_comment_stream(
        stream: &mut IoStream,
        comments: &Sequence<ExtendedString>,
        only_count: bool,
    ) -> Result<i32, Failure> {
        let mut size = Self::put_integer_stream(stream, comments.length(), only_count)?;
        for comment in comments.iter() {
            size += Self::write_extended_string_stream(stream, comment, only_count)?;
        }
        Ok(size)
    }

    /// `ReadComment(Standard_IStream&, ...)`: appends comments while the stream is good.
    /// Example: `BinaryFile::read_comment_stream(&mut input, &mut comments)?`.
    pub fn read_comment_stream(
        stream: &mut IoStream,
        comments: &mut Sequence<ExtendedString>,
    ) -> Result<(), Failure> {
        let mut length = 0;
        Self::get_integer_stream(stream, &mut length)?;
        let mut line = ExtendedString::default();
        let mut index = 1;
        while index <= length && stream.good() {
            Self::read_extended_string_stream(stream, &mut line)?;
            comments.append(line.clone());
            index += 1;
        }
        Ok(())
    }

    /// `ReadHeaderData`: reads the info section into locals, then stores them in `header`.
    /// Example: `BinaryFile::read_header_data(&mut input, &header)?`.
    pub fn read_header_data(
        stream: &mut IoStream,
        header: &Handle<HeaderData>,
    ) -> Result<(), Failure> {
        let info = InfoFields::read(stream)?;
        let header = header.get().ok_or_else(null_header)?;
        info.store(header);
        Ok(())
    }
}

/// The info section as `ReadHeaderData` reads it into locals before storing it.
#[derive(Default)]
struct InfoFields {
    object_count: i32,
    texts: [AsciiString; 5],
    names: [ExtendedString; 2],
    user_info: Sequence<AsciiString>,
}

impl InfoFields {
    fn read(stream: &mut IoStream) -> Result<Self, Failure> {
        let mut info = Self::default();
        BinaryFile::get_integer_stream(stream, &mut info.object_count)?;
        for text in info.texts.iter_mut().take(4) {
            BinaryFile::read_string_stream(stream, text)?;
        }
        BinaryFile::read_extended_string_stream(stream, &mut info.names[0])?;
        BinaryFile::read_string_stream(stream, &mut info.texts[4])?;
        BinaryFile::read_extended_string_stream(stream, &mut info.names[1])?;
        info.user_info = read_user_info(stream)?;
        Ok(info)
    }

    fn store(&self, header: &HeaderData) {
        let [
            version,
            date,
            schema_name,
            schema_version,
            application_version,
        ] = &self.texts;
        header.set_number_of_objects(self.object_count);
        header.set_storage_version(version);
        header.set_creation_date(date);
        header.set_schema_name(schema_name);
        header.set_schema_version(schema_version);
        header.set_application_name(&self.names[0]);
        header.set_application_version(application_version);
        header.set_data_type(&self.names[1]);
        for line in self.user_info.iter() {
            header.add_to_user_info(line);
        }
    }
}

fn read_user_info(stream: &mut IoStream) -> Result<Sequence<AsciiString>, Failure> {
    let mut length = 0;
    BinaryFile::get_integer_stream(stream, &mut length)?;
    let mut lines = Sequence::default();
    let mut line = AsciiString::default();
    let mut index = 1;
    while index <= length && stream.good() {
        BinaryFile::read_string_stream(stream, &mut line)?;
        lines.append(line.clone());
        index += 1;
    }
    Ok(lines)
}

fn null_header() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "FSD_BinaryFile::ReadHeaderData: null header; expected a Storage_HeaderData",
        "",
    )
}

impl BinaryFile {
    /// `TypeSectionSize(Standard_IStream&)`; `BinaryFile::type_section_size_stream(&mut input)?`.
    pub fn type_section_size_stream(stream: &mut IoStream) -> Result<i32, Failure> {
        let mut size = 0;
        Self::get_integer_stream(stream, &mut size)?;
        Ok(size)
    }

    /// `ReadTypeInformations(Standard_IStream&, int&, TCollection_AsciiString&)`.
    /// Example: `BinaryFile::read_type_informations_stream(&mut input, &mut n, &mut name)?`.
    pub fn read_type_informations_stream(
        stream: &mut IoStream,
        number: &mut i32,
        name: &mut AsciiString,
    ) -> Result<(), Failure> {
        Self::get_integer_stream(stream, number)?;
        Self::read_string_stream(stream, name)
    }

    /// `RootSectionSize(Standard_IStream&)`; `BinaryFile::root_section_size_stream(&mut input)?`.
    pub fn root_section_size_stream(stream: &mut IoStream) -> Result<i32, Failure> {
        Self::type_section_size_stream(stream)
    }

    /// `ReadRoot(Standard_IStream&, ...)`: reference, name, type name.
    /// Example: `BinaryFile::read_root_stream(&mut input, &mut name, &mut r, &mut ty)?`.
    pub fn read_root_stream(
        stream: &mut IoStream,
        name: &mut AsciiString,
        reference: &mut i32,
        type_name: &mut AsciiString,
    ) -> Result<(), Failure> {
        Self::get_reference_stream(stream, reference)?;
        Self::read_string_stream(stream, name)?;
        Self::read_string_stream(stream, type_name)
    }

    /// `RefSectionSize(Standard_IStream&)`; `BinaryFile::ref_section_size_stream(&mut input)?`.
    pub fn ref_section_size_stream(stream: &mut IoStream) -> Result<i32, Failure> {
        Self::type_section_size_stream(stream)
    }

    /// `ReadReferenceType(Standard_IStream&, int&, int&)`.
    /// Example: `BinaryFile::read_reference_type_stream(&mut input, &mut r, &mut t)?`.
    pub fn read_reference_type_stream(
        stream: &mut IoStream,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        Self::get_reference_stream(stream, reference)?;
        Self::get_integer_stream(stream, type_number)
    }

    /// `EndWriteInfoSection(Standard_OStream&)`: records `tellp()` as the info end.
    /// Example: `driver.end_write_info_section_stream(&mut out)`.
    pub fn end_write_info_section_stream(&mut self, stream: &mut IoStream) -> StorageError {
        self.header.end_info = stream.tellp() as i32;
        StorageError::Ok
    }

    /// `BeginWriteCommentSection(Standard_OStream&)`: records `tellp()` as the comment start.
    /// Example: `driver.begin_write_comment_section_stream(&mut out)`.
    pub fn begin_write_comment_section_stream(&mut self, stream: &mut IoStream) -> StorageError {
        self.header.begin_comment = stream.tellp() as i32;
        StorageError::Ok
    }

    /// `EndWriteCommentSection(Standard_OStream&)`: records `tellp()` as the comment end.
    /// Example: `driver.end_write_comment_section_stream(&mut out)`.
    pub fn end_write_comment_section_stream(&mut self, stream: &mut IoStream) -> StorageError {
        self.header.end_comment = stream.tellp() as i32;
        StorageError::Ok
    }
}
