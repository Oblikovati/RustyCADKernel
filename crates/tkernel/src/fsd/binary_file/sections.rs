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

//! Info and comment sections of the binary format, and its offset header.
use super::{BinaryFile, MAGIC_NUMBER, magic_status};
use crate::ncollection::Sequence;
use crate::standard::{Failure, Handle};
use crate::storage::{Data, DriverPrimitives, HeaderSections, IoStream, StorageError};
use crate::tcollection::{AsciiString, ExtendedString};

impl BinaryFile {
    /// Private `WriteHeader`: the thirteen header fields at the current position.
    pub(crate) fn write_header(&mut self) -> Result<(), Failure> {
        for field in self.header.fields() {
            self.put_integer(field)?;
        }
        Ok(())
    }

    /// Private `ReadHeader`: fills the header field by field; a short read stops there.
    fn read_header(&mut self) -> Result<(), Failure> {
        let mut header = self.header;
        let mut outcome = Ok(());
        for field in header.fields_mut() {
            outcome = self.read_int(field, "FSD_BinaryFile::GetInteger");
            if outcome.is_err() {
                break;
            }
        }
        self.header = header;
        outcome
    }
}

impl HeaderSections for BinaryFile {
    /// Writes the magic number and a provisional header: the byte-order probe is the
    /// bytes 1, 2, 3, 4 read as a native `int`, the begin-info offset follows the magic.
    fn begin_write_info_section(&mut self) -> Result<StorageError, Failure> {
        self.header.test_indian = i32::from_ne_bytes([1, 2, 3, 4]);
        self.write_item(
            MAGIC_NUMBER.as_bytes(),
            "FSD_BinaryFile::BeginWriteInfoSection",
        )?;
        self.header.begin_info = self.position()?;
        self.write_header()?;
        Ok(StorageError::Ok)
    }

    fn write_info(
        &mut self,
        object_count: i32,
        storage_version: &AsciiString,
        date: &AsciiString,
        schema_name: &AsciiString,
        schema_version: &AsciiString,
        application_name: &ExtendedString,
        application_version: &AsciiString,
        data_type: &ExtendedString,
        user_info: &Sequence<AsciiString>,
    ) -> Result<(), Failure> {
        self.put_integer(object_count)?;
        self.write_string(storage_version)?;
        self.write_string(date)?;
        self.write_string(schema_name)?;
        self.write_string(schema_version)?;
        self.write_extended_string(application_name)?;
        self.write_string(application_version)?;
        self.write_extended_string(data_type)?;
        self.put_integer(user_info.length())?;
        for line in user_info.iter() {
            self.write_string(line)?;
        }
        Ok(())
    }

    fn end_write_info_section(&mut self) -> Result<StorageError, Failure> {
        self.header.end_info = self.position()?;
        Ok(StorageError::Ok)
    }

    /// Checks the magic number, then reads the offset header.
    fn begin_read_info_section(&mut self) -> Result<StorageError, Failure> {
        let magic = self.read_char(MAGIC_NUMBER.len())?;
        let status = magic_status(&magic, MAGIC_NUMBER);
        if status == StorageError::Ok {
            self.read_header()?;
        }
        Ok(status)
    }

    fn read_info(
        &mut self,
        object_count: &mut i32,
        storage_version: &mut AsciiString,
        date: &mut AsciiString,
        schema_name: &mut AsciiString,
        schema_version: &mut AsciiString,
        application_name: &mut ExtendedString,
        application_version: &mut AsciiString,
        data_type: &mut ExtendedString,
        user_info: &mut Sequence<AsciiString>,
    ) -> Result<(), Failure> {
        self.get_integer(object_count)?;
        self.read_string(storage_version)?;
        self.read_string(date)?;
        self.read_string(schema_name)?;
        self.read_string(schema_version)?;
        self.read_extended_string(application_name)?;
        self.read_string(application_version)?;
        self.read_extended_string(data_type)?;
        let mut length = 0;
        self.get_integer(&mut length)?;
        let mut line = AsciiString::default();
        let mut index = 1;
        while index <= length && !self.file()?.is_end_of_file() {
            self.read_string(&mut line)?;
            user_info.append(line.clone());
            index += 1;
        }
        Ok(())
    }

    fn read_complete_info(
        &mut self,
        stream: &mut IoStream,
        data: &mut Handle<Data>,
    ) -> Result<(), Failure> {
        self.read_complete(stream, data)
    }

    fn end_read_info_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.end_info)
    }

    fn begin_write_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.header.begin_comment = self.position()?;
        Ok(StorageError::Ok)
    }

    fn write_comment(&mut self, comments: &Sequence<ExtendedString>) -> Result<(), Failure> {
        self.put_integer(comments.length())?;
        for comment in comments.iter() {
            self.write_extended_string(comment)?;
        }
        Ok(())
    }

    fn end_write_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.header.end_comment = self.position()?;
        Ok(StorageError::Ok)
    }

    fn begin_read_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.begin_comment)
    }

    fn read_comment(&mut self, comments: &mut Sequence<ExtendedString>) -> Result<(), Failure> {
        let mut length = 0;
        self.get_integer(&mut length)?;
        let mut line = ExtendedString::default();
        let mut index = 1;
        while index <= length && !self.file()?.is_end_of_file() {
            self.read_extended_string(&mut line)?;
            comments.append(line.clone());
            index += 1;
        }
        Ok(())
    }

    fn end_read_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.end_comment)
    }
}
