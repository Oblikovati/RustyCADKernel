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
