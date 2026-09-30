//! Type, root, reference and data sections of the binary format. Each begin/end of a
//! written section records its offset; each read section seeks to the recorded offset.
use super::BinaryFile;
use crate::standard::Failure;
use crate::storage::{DataSections, DriverPrimitives, StorageError, TableSections};
use crate::tcollection::AsciiString;

impl BinaryFile {
    /// `GetInteger` into a fresh `int`, as the `*SectionSize` members do.
    fn section_size(&mut self) -> Result<i32, Failure> {
        let mut size = 0;
        self.get_integer(&mut size)?;
        Ok(size)
    }
}

impl TableSections for BinaryFile {
    fn begin_write_type_section(&mut self) -> Result<StorageError, Failure> {
        self.header.begin_type = self.position()?;
        Ok(StorageError::Ok)
    }

    fn set_type_section_size(&mut self, size: i32) -> Result<(), Failure> {
        self.put_integer(size)
    }

    fn write_type_informations(&mut self, number: i32, name: &AsciiString) -> Result<(), Failure> {
        self.put_integer(number)?;
        self.write_string(name)
    }

    fn end_write_type_section(&mut self) -> Result<StorageError, Failure> {
        self.header.end_type = self.position()?;
        Ok(StorageError::Ok)
    }

    fn begin_read_type_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.begin_type)
    }

    fn type_section_size(&mut self) -> Result<i32, Failure> {
        self.section_size()
    }

    fn read_type_informations(
        &mut self,
        number: &mut i32,
        name: &mut AsciiString,
    ) -> Result<(), Failure> {
        self.get_integer(number)?;
        self.read_string(name)
    }

    fn end_read_type_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.end_type)
    }

    fn begin_write_root_section(&mut self) -> Result<StorageError, Failure> {
        self.header.begin_root = self.position()?;
        Ok(StorageError::Ok)
    }

    fn set_root_section_size(&mut self, size: i32) -> Result<(), Failure> {
        self.put_integer(size)
    }

    fn write_root(
        &mut self,
        name: &AsciiString,
        reference: i32,
        type_name: &AsciiString,
    ) -> Result<(), Failure> {
        self.put_reference(reference)?;
        self.write_string(name)?;
        self.write_string(type_name)
    }

    fn end_write_root_section(&mut self) -> Result<StorageError, Failure> {
        self.header.end_root = self.position()?;
        Ok(StorageError::Ok)
    }

    fn begin_read_root_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.begin_root)
    }

    fn root_section_size(&mut self) -> Result<i32, Failure> {
        self.section_size()
    }

    fn read_root(
        &mut self,
        name: &mut AsciiString,
        reference: &mut i32,
        type_name: &mut AsciiString,
    ) -> Result<(), Failure> {
        self.get_reference(reference)?;
        self.read_string(name)?;
        self.read_string(type_name)
    }

    fn end_read_root_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.end_root)
    }

    fn begin_write_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.header.begin_ref = self.position()?;
        Ok(StorageError::Ok)
    }

    fn set_ref_section_size(&mut self, size: i32) -> Result<(), Failure> {
        self.put_integer(size)
    }

    fn write_reference_type(&mut self, reference: i32, type_number: i32) -> Result<(), Failure> {
        self.put_reference(reference)?;
        self.put_integer(type_number)
    }

    fn end_write_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.header.end_ref = self.position()?;
        Ok(StorageError::Ok)
    }

    fn begin_read_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.begin_ref)
    }

    fn ref_section_size(&mut self) -> Result<i32, Failure> {
        self.section_size()
    }

    fn read_reference_type(
        &mut self,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        self.get_reference(reference)?;
        self.get_integer(type_number)
    }

    fn end_read_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.end_ref)
    }
}

impl DataSections for BinaryFile {
    fn begin_write_data_section(&mut self) -> Result<StorageError, Failure> {
        self.header.begin_data = self.position()?;
        Ok(StorageError::Ok)
    }

    fn write_persistent_object_header(
        &mut self,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure> {
        self.put_reference(reference)?;
        self.put_integer(type_number)
    }

    /// Object framing is implicit in the binary format.
    fn begin_write_persistent_object_data(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    fn begin_write_object_data(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    fn end_write_object_data(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    fn end_write_persistent_object_data(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    /// Records the end offset, then rewrites the header after the magic number; the
    /// file position stays after the header, as the native `fseek` leaves it.
    fn end_write_data_section(&mut self) -> Result<StorageError, Failure> {
        self.header.end_data = self.position()?;
        let begin_info = i64::from(self.header.begin_info);
        self.file()?.seek(begin_info);
        self.write_header()?;
        Ok(StorageError::Ok)
    }

    fn begin_read_data_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.begin_data)
    }

    fn read_persistent_object_header(
        &mut self,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure> {
        self.get_reference(reference)?;
        self.get_integer(type_number)
    }

    fn begin_read_persistent_object_data(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    fn begin_read_object_data(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    fn end_read_object_data(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    fn end_read_persistent_object_data(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    fn end_read_data_section(&mut self) -> Result<StorageError, Failure> {
        self.seek_section(self.header.end_data)
    }
}
