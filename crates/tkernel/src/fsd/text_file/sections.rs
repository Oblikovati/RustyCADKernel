//! Info and comment sections of the text formats.
use super::{TextDialect, TextFile};
use crate::fsd::binary_file::magic_status;
use crate::ncollection::Sequence;
use crate::standard::{Failure, Handle};
use crate::storage::{Data, HeaderSections, IoStream, StorageError};
use crate::tcollection::{AsciiString, ExtendedString};

impl<D: TextDialect> TextFile<D> {
    /// `myStream << text << "\n"` with `text` as a C string.
    fn write_text_line(&mut self, text: &[u8]) {
        self.write_c_string(text);
        self.stream.insert_bytes(b"\n");
    }

    /// `myStream << value << "\n"`.
    fn write_count_line(&mut self, value: i32) {
        self.stream.insert_integer(i64::from(value));
        self.stream.insert_bytes(b"\n");
    }

    /// Writes a section tag line and checks the stream.
    pub(crate) fn write_tag(&mut self, tag: &[u8], member: &str) -> Result<StorageError, Failure> {
        self.stream.insert_bytes(tag);
        self.check_written(member)?;
        Ok(StorageError::Ok)
    }

    /// `>> count` then `FlushEndOfLine`, as every text section size is read.
    pub(crate) fn read_count_line(&mut self, member: &str) -> Result<i32, Failure> {
        let mut count = 0;
        self.extract_integer(&mut count, member)?;
        self.flush_end_of_line();
        Ok(count)
    }
}

impl<D: TextDialect> HeaderSections for TextFile<D> {
    /// The magic number line and `BEGIN_INFO_SECTION`.
    fn begin_write_info_section(&mut self) -> Result<StorageError, Failure> {
        self.write_text_line(D::MAGIC_NUMBER.as_bytes());
        self.write_tag(b"BEGIN_INFO_SECTION\n", "FSD_File::BeginWriteInfoSection")
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
        self.write_count_line(object_count);
        for text in [storage_version, date, schema_name, schema_version] {
            self.write_text_line(text.as_bytes());
        }
        D::write_extended_line(self, application_name)?;
        self.write_text_line(application_version.as_bytes());
        D::write_extended_line(self, data_type)?;
        self.write_count_line(user_info.length());
        self.check_written("FSD_File::WriteInfo")?;
        for line in user_info.iter() {
            self.write_text_line(line.as_bytes());
            self.check_written("FSD_File::WriteInfo")?;
        }
        Ok(())
    }

    fn end_write_info_section(&mut self) -> Result<StorageError, Failure> {
        self.write_tag(b"END_INFO_SECTION\n", "FSD_File::EndWriteInfoSection")
    }

    /// Checks the magic number, then finds `BEGIN_INFO_SECTION`.
    fn begin_read_info_section(&mut self) -> Result<StorageError, Failure> {
        let magic = self.read_char(D::MAGIC_NUMBER.len());
        let status = magic_status(&magic, D::MAGIC_NUMBER);
        if status != StorageError::Ok {
            return Ok(status);
        }
        Ok(self.find_tag("BEGIN_INFO_SECTION"))
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
        self.extract_integer(object_count, "FSD_File::ReadInfo")?;
        self.flush_end_of_line();
        for text in [storage_version, date, schema_name, schema_version] {
            *text = D::read_line(self);
        }
        D::read_extended_line(self, application_name)?;
        *application_version = D::read_line(self);
        D::read_extended_line(self, data_type)?;
        let length = self.read_count_line("FSD_File::ReadInfo")?;
        let mut index = 1;
        while index <= length && !self.stream.eof() {
            user_info.append(D::read_line(self));
            index += 1;
        }
        Ok(())
    }

    /// Does nothing in the text formats.
    fn read_complete_info(
        &mut self,
        _: &mut IoStream,
        _: &mut Handle<Data>,
    ) -> Result<(), Failure> {
        Ok(())
    }

    fn end_read_info_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("END_INFO_SECTION"))
    }

    fn begin_write_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.write_tag(
            b"BEGIN_COMMENT_SECTION\n",
            "FSD_File::BeginWriteCommentSection",
        )
    }

    fn write_comment(&mut self, comments: &Sequence<ExtendedString>) -> Result<(), Failure> {
        self.write_count_line(comments.length());
        self.check_written("FSD_File::WriteComment")?;
        for comment in comments.iter() {
            D::write_extended_line(self, comment)?;
            self.check_written("FSD_File::WriteComment")?;
        }
        Ok(())
    }

    fn end_write_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.write_tag(b"END_COMMENT_SECTION\n", "FSD_File::EndWriteCommentSection")
    }

    fn begin_read_comment_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("BEGIN_COMMENT_SECTION"))
    }

    /// Each comment is read into a cleared line, so `FSD_CmpFile` reads no unit of it.
    fn read_comment(&mut self, comments: &mut Sequence<ExtendedString>) -> Result<(), Failure> {
        let length = self.read_count_line("FSD_File::ReadComment")?;
        let mut line = ExtendedString::default();
        let mut index = 1;
        while index <= length && !self.stream.eof() {
            D::read_extended_line(self, &mut line)?;
            comments.append(line.clone());
            line = ExtendedString::default();
            index += 1;
        }
        Ok(())
    }

    fn end_read_comment_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("END_COMMENT_SECTION"))
    }
}
