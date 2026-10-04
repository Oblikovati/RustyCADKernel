//! The data section of the text formats: `#ref=%type( ... )` objects in `FSD_File`,
//! `#ref%type ...` objects in `FSD_CmpFile`.
use super::{TextDialect, TextFile};
use crate::standard::Failure;
use crate::storage::{DataSections, StorageError, StreamException};

fn format_error(expected: u8) -> Failure {
    StreamException::Format.raise(&format!(
        "FSD_File: expected spaces then {:?} in the data section",
        char::from(expected)
    ))
}

impl<D: TextDialect> TextFile<D> {
    /// Calls qualified FSD header writing without redispatching the dialect hook.
    /// Example: `file.write_fsd_object_header_base(4, 2)`.
    pub fn write_fsd_object_header_base(
        &mut self,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure> {
        self.stream.insert_bytes(b"\n#");
        self.stream.insert_integer(i64::from(reference));
        self.stream.insert_bytes(b"=%");
        self.stream.insert_integer(i64::from(type_number));
        self.check_written("FSD_File::WritePersistentObjectHeader")
    }

    /// Calls qualified FSD object opening, using `member` for native error context.
    /// Example: `file.begin_fsd_write_object_base("FSD_File::BeginWriteObjectData")`.
    pub fn begin_fsd_write_object_base(&mut self, member: &str) -> Result<(), Failure> {
        self.stream.insert_bytes(b"( ");
        self.check_written(member)
    }

    /// Calls qualified FSD object closing; `last` selects persistent-object framing.
    /// Example: `file.end_fsd_write_object_base(false, "FSD_File::EndWriteObjectData")`.
    pub fn end_fsd_write_object_base(&mut self, last: bool, member: &str) -> Result<(), Failure> {
        self.stream.insert_bytes(if last { b")" } else { b") " });
        self.check_written(member)
    }

    /// Calls qualified FSD header parsing without redispatching the dialect hook.
    /// Example: `file.read_fsd_object_header_base(&mut reference, &mut type_number)`.
    pub fn read_fsd_object_header_base(
        &mut self,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        self.expect_marker(b'#')?;
        self.extract_integer(reference, "FSD_File::ReadPersistentObjectHeader")?;
        self.expect_marker(b'=')?;
        self.expect_marker(b'%')?;
        self.extract_integer(type_number, "FSD_File::ReadPersistentObjectHeader")
    }

    /// Calls qualified FSD object opening without redispatching the dialect hook.
    /// Example: `file.begin_fsd_read_object_base()`.
    pub fn begin_fsd_read_object_base(&mut self) -> Result<(), Failure> {
        self.expect_marker(b'(')
    }

    /// Calls qualified FSD object closing and optional line-end validation.
    /// Example: `file.end_fsd_read_object_base(true)`.
    pub fn end_fsd_read_object_base(&mut self, last: bool) -> Result<(), Failure> {
        self.expect_marker(b')')?;
        if last {
            self.expect_line_end(false)?;
        }
        Ok(())
    }

    /// The native marker loops: `get(c); while (c != marker) { if (IsEnd() || c != ' ')
    /// throw Storage_StreamFormatError(); get(c); }`.
    pub(crate) fn expect_marker(&mut self, marker: u8) -> Result<(), Failure> {
        let mut byte = 0_u8;
        self.stream.get(&mut byte);
        while byte != marker {
            if self.stream.eof() || byte != b' ' {
                return Err(format_error(marker));
            }
            self.stream.get(&mut byte);
        }
        Ok(())
    }

    /// Spaces up to the end of the line; `FSD_CmpFile` also accepts `\r` and `\r\n`.
    pub(crate) fn expect_line_end(&mut self, carriage_return: bool) -> Result<(), Failure> {
        let ends = |byte: u8| byte == b'\n' || (carriage_return && byte == b'\r');
        let mut byte = 0_u8;
        self.stream.get(&mut byte);
        while !ends(byte) {
            if self.stream.eof() || byte != b' ' {
                return Err(format_error(b'\n'));
            }
            self.stream.get(&mut byte);
        }
        if byte == b'\r' {
            self.stream.get(&mut byte);
        }
        Ok(())
    }

    /// `if (!(myStream >> value)) throw Storage_StreamTypeMismatchError()`.
    pub(crate) fn extract_integer(&mut self, value: &mut i32, member: &str) -> Result<(), Failure> {
        if !self.stream.extract_int(value) {
            return Err(StreamException::TypeMismatch
                .raise(&format!("{member}: expected a decimal integer")));
        }
        Ok(())
    }

    fn write_marker(&mut self, text: &[u8], member: &str) -> Result<StorageError, Failure> {
        self.stream.insert_bytes(text);
        self.check_written(member)?;
        Ok(StorageError::Ok)
    }
}

impl<D: TextDialect> DataSections for TextFile<D> {
    /// `BEGIN_DATA_SECTION` without a newline: each object header starts with one.
    fn begin_write_data_section(&mut self) -> Result<StorageError, Failure> {
        self.write_marker(b"BEGIN_DATA_SECTION", "FSD_File::BeginWriteDataSection")
    }

    fn write_persistent_object_header(
        &mut self,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure> {
        D::write_object_header(self, reference, type_number)
    }

    fn begin_write_persistent_object_data(&mut self) -> Result<(), Failure> {
        D::begin_write_object(self, "FSD_File::BeginWritePersistentObjectData")
    }

    fn begin_write_object_data(&mut self) -> Result<(), Failure> {
        D::begin_write_object(self, "FSD_File::BeginWriteObjectData")
    }

    fn end_write_object_data(&mut self) -> Result<(), Failure> {
        D::end_write_object(self, false, "FSD_File::EndWriteObjectData")
    }

    fn end_write_persistent_object_data(&mut self) -> Result<(), Failure> {
        D::end_write_object(self, true, "FSD_File::EndWritePersistentObjectData")
    }

    fn end_write_data_section(&mut self) -> Result<StorageError, Failure> {
        self.write_marker(b"\nEND_DATA_SECTION\n", "FSD_File::EndWriteDataSection")
    }

    fn begin_read_data_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("BEGIN_DATA_SECTION"))
    }

    fn read_persistent_object_header(
        &mut self,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        D::read_object_header(self, reference, type_number)
    }

    fn begin_read_persistent_object_data(&mut self) -> Result<(), Failure> {
        D::begin_read_object(self)
    }

    fn begin_read_object_data(&mut self) -> Result<(), Failure> {
        D::begin_read_object(self)
    }

    fn end_read_object_data(&mut self) -> Result<(), Failure> {
        D::end_read_object(self, false)
    }

    fn end_read_persistent_object_data(&mut self) -> Result<(), Failure> {
        D::end_read_object(self, true)
    }

    fn end_read_data_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("END_DATA_SECTION"))
    }
}
