//! Type, root and reference sections of the text formats: one entry per line.
use super::{TextDialect, TextFile};
use crate::standard::Failure;
use crate::storage::{StorageError, StreamException, TableSections};
use crate::tcollection::AsciiString;

/// `TCollection_AsciiString::Read`'s `char buffer[8190]` with `width(8190)`.
const WORD_BUFFER: usize = 8190;

impl<D: TextDialect> TextFile<D> {
    /// `myStream << a << " " << b ...` entries, then the `bad()` check.
    fn write_fields(&mut self, fields: &[Field<'_>], member: &str) -> Result<(), Failure> {
        for (index, field) in fields.iter().enumerate() {
            if index > 0 {
                self.stream.insert_bytes(b" ");
            }
            match field {
                Field::Number(value) => {
                    self.stream.insert_integer(i64::from(*value));
                }
                Field::Text(text) => self.write_c_string(text.as_bytes()),
            }
        }
        self.stream.insert_bytes(b"\n");
        self.check_written(member)
    }

    /// `myStream >> text` for a `TCollection_AsciiString`: one word of at most 8189 bytes.
    fn extract_ascii_word(&mut self, text: &mut AsciiString, member: &str) -> Result<(), Failure> {
        let previous = self.stream.set_width(WORD_BUFFER as i64);
        let word = self.stream.extract_word(WORD_BUFFER);
        self.stream.set_width(previous);
        *text = AsciiString::from_bytes(&word);
        if self.stream.fail() {
            return Err(StreamException::TypeMismatch.raise(&format!("{member}: expected a word")));
        }
        Ok(())
    }

    fn write_count(&mut self, size: i32, member: &str) -> Result<(), Failure> {
        self.stream.insert_integer(i64::from(size));
        self.stream.insert_bytes(b"\n");
        self.check_written(member)
    }
}

/// One field of a table entry line.
enum Field<'a> {
    Number(i32),
    Text(&'a AsciiString),
}

impl<D: TextDialect> TableSections for TextFile<D> {
    fn begin_write_type_section(&mut self) -> Result<StorageError, Failure> {
        self.write_tag(b"BEGIN_TYPE_SECTION\n", "FSD_File::BeginWriteTypeSection")
    }

    fn set_type_section_size(&mut self, size: i32) -> Result<(), Failure> {
        self.write_count(size, "FSD_File::SetTypeSectionSize")
    }

    fn write_type_informations(&mut self, number: i32, name: &AsciiString) -> Result<(), Failure> {
        let fields = [Field::Number(number), Field::Text(name)];
        self.write_fields(&fields, "FSD_File::WriteTypeInformations")
    }

    fn end_write_type_section(&mut self) -> Result<StorageError, Failure> {
        self.write_tag(b"END_TYPE_SECTION\n", "FSD_File::EndWriteTypeSection")
    }

    fn begin_read_type_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("BEGIN_TYPE_SECTION"))
    }

    fn type_section_size(&mut self) -> Result<i32, Failure> {
        self.read_count_line("FSD_File::TypeSectionSize")
    }

    fn read_type_informations(
        &mut self,
        number: &mut i32,
        name: &mut AsciiString,
    ) -> Result<(), Failure> {
        self.extract_integer(number, "FSD_File::ReadTypeInformations")?;
        self.extract_ascii_word(name, "FSD_File::ReadTypeInformations")?;
        self.flush_end_of_line();
        Ok(())
    }

    fn end_read_type_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("END_TYPE_SECTION"))
    }

    fn begin_write_root_section(&mut self) -> Result<StorageError, Failure> {
        self.write_tag(b"BEGIN_ROOT_SECTION\n", "FSD_File::BeginWriteRootSection")
    }

    fn set_root_section_size(&mut self, size: i32) -> Result<(), Failure> {
        self.write_count(size, "FSD_File::SetRootSectionSize")
    }

    fn write_root(
        &mut self,
        name: &AsciiString,
        reference: i32,
        type_name: &AsciiString,
    ) -> Result<(), Failure> {
        let fields = [
            Field::Number(reference),
            Field::Text(name),
            Field::Text(type_name),
        ];
        self.write_fields(&fields, "FSD_File::WriteRoot")
    }

    fn end_write_root_section(&mut self) -> Result<StorageError, Failure> {
        self.write_tag(b"END_ROOT_SECTION\n", "FSD_File::EndWriteRootSection")
    }

    fn begin_read_root_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("BEGIN_ROOT_SECTION"))
    }

    fn root_section_size(&mut self) -> Result<i32, Failure> {
        self.read_count_line("FSD_File::RootSectionSize")
    }

    /// The reference, then two words; the rest of the line stays unread.
    fn read_root(
        &mut self,
        name: &mut AsciiString,
        reference: &mut i32,
        type_name: &mut AsciiString,
    ) -> Result<(), Failure> {
        self.extract_integer(reference, "FSD_File::ReadRoot")?;
        *name = self.read_word();
        *type_name = self.read_word();
        Ok(())
    }

    fn end_read_root_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("END_ROOT_SECTION"))
    }

    fn begin_write_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.write_tag(b"BEGIN_REF_SECTION\n", "FSD_File::BeginWriteRefSection")
    }

    fn set_ref_section_size(&mut self, size: i32) -> Result<(), Failure> {
        self.write_count(size, "FSD_File::SetRefSectionSize")
    }

    fn write_reference_type(&mut self, reference: i32, type_number: i32) -> Result<(), Failure> {
        let fields = [Field::Number(reference), Field::Number(type_number)];
        self.write_fields(&fields, "FSD_File::WriteReferenceType")
    }

    fn end_write_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.write_tag(b"END_REF_SECTION\n", "FSD_File::EndWriteRefSection")
    }

    fn begin_read_ref_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("BEGIN_REF_SECTION"))
    }

    fn ref_section_size(&mut self) -> Result<i32, Failure> {
        self.read_count_line("FSD_File::RefSectionSize")
    }

    fn read_reference_type(
        &mut self,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        self.extract_integer(reference, "FSD_File::ReadReferenceType")?;
        self.extract_integer(type_number, "FSD_File::ReadReferenceType")?;
        self.flush_end_of_line();
        Ok(())
    }

    fn end_read_ref_section(&mut self) -> Result<StorageError, Failure> {
        Ok(self.find_tag("END_REF_SECTION"))
    }
}
