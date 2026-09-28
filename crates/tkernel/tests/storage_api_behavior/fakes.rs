//! Named fakes: a scripted driver that records every call, a fixed clock and fixed
//! migration table sources.
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::{
    BaseDriver, CreationDateSource, Data, DataSections, DriverBase, DriverPrimitives,
    HeaderSections, IoStream, MigrationFile, MigrationSource, OpenMode, Position, StorageError,
    StreamException, TableSections,
};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};

/// A driver that logs each member it runs, fails with `failure` at the member named
/// `fail_at`, and answers reads from `roots`/`types`.
#[derive(Default)]
pub struct ScriptedDriver {
    pub base: DriverBase,
    pub log: Vec<String>,
    pub fail_at: Option<(&'static str, StreamException)>,
    pub status_at: Option<(&'static str, StorageError)>,
    pub roots: Vec<(&'static str, i32, &'static str)>,
    pub types: Vec<(i32, &'static str)>,
    pub info_count: i32,
}

impl ScriptedDriver {
    pub fn opened(mode: OpenMode) -> Self {
        let mut driver = Self::default();
        driver.base.set_open_mode(mode);
        driver
    }

    fn call(&mut self, member: &str) -> Result<(), Failure> {
        self.log.push(member.to_string());
        match self.fail_at {
            Some((name, kind)) if name == member => Err(kind.raise(member)),
            _ => Ok(()),
        }
    }

    fn step(&mut self, member: &str) -> Result<StorageError, Failure> {
        self.call(member)?;
        Ok(match self.status_at {
            Some((name, status)) if name == member => status,
            _ => StorageError::Ok,
        })
    }
}

impl HeaderSections for ScriptedDriver {
    fn begin_write_info_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginWriteInfoSection")
    }
    fn write_info(
        &mut self,
        count: i32,
        _: &AsciiString,
        _: &AsciiString,
        schema: &AsciiString,
        _: &AsciiString,
        _: &ExtendedString,
        _: &AsciiString,
        _: &ExtendedString,
        info: &Sequence<AsciiString>,
    ) -> Result<(), Failure> {
        let text = String::from_utf8_lossy(schema.as_bytes()).into_owned();
        self.call(&format!("WriteInfo {count} {text} {}", info.length()))
    }
    fn end_write_info_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndWriteInfoSection")
    }
    fn begin_read_info_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginReadInfoSection")
    }
    fn read_info(
        &mut self,
        count: &mut i32,
        version: &mut AsciiString,
        _: &mut AsciiString,
        _: &mut AsciiString,
        _: &mut AsciiString,
        _: &mut ExtendedString,
        _: &mut AsciiString,
        _: &mut ExtendedString,
        _: &mut Sequence<AsciiString>,
    ) -> Result<(), Failure> {
        // Values land before a scripted failure, as through the native references.
        *count = self.info_count;
        *version = AsciiString::new("1.3");
        self.call("ReadInfo")
    }
    fn read_complete_info(
        &mut self,
        _: &mut IoStream,
        _: &mut Handle<Data>,
    ) -> Result<(), Failure> {
        self.call("ReadCompleteInfo")
    }
    fn end_read_info_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndReadInfoSection")
    }
    fn begin_write_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginWriteCommentSection")
    }
    fn write_comment(&mut self, comments: &Sequence<ExtendedString>) -> Result<(), Failure> {
        self.call(&format!("WriteComment {}", comments.length()))
    }
    fn end_write_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndWriteCommentSection")
    }
    fn begin_read_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginReadCommentSection")
    }
    fn read_comment(&mut self, comments: &mut Sequence<ExtendedString>) -> Result<(), Failure> {
        comments.append(ExtendedString::new("scripted"));
        self.call("ReadComment")
    }
    fn end_read_comment_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndReadCommentSection")
    }
}

impl TableSections for ScriptedDriver {
    fn begin_write_type_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginWriteTypeSection")
    }
    fn set_type_section_size(&mut self, size: i32) -> Result<(), Failure> {
        self.call(&format!("SetTypeSectionSize {size}"))
    }
    fn write_type_informations(&mut self, number: i32, _: &AsciiString) -> Result<(), Failure> {
        self.call(&format!("WriteTypeInformations {number}"))
    }
    fn end_write_type_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndWriteTypeSection")
    }
    fn begin_read_type_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginReadTypeSection")
    }
    fn type_section_size(&mut self) -> Result<i32, Failure> {
        self.call("TypeSectionSize")?;
        Ok(self.types.len() as i32)
    }
    fn read_type_informations(
        &mut self,
        number: &mut i32,
        name: &mut AsciiString,
    ) -> Result<(), Failure> {
        self.call("ReadTypeInformations")?;
        let (value, text) = self.types.remove(0);
        *number = value;
        *name = AsciiString::new(text);
        Ok(())
    }
    fn end_read_type_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndReadTypeSection")
    }
    fn begin_write_root_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginWriteRootSection")
    }
    fn set_root_section_size(&mut self, size: i32) -> Result<(), Failure> {
        self.call(&format!("SetRootSectionSize {size}"))
    }
    fn write_root(
        &mut self,
        name: &AsciiString,
        reference: i32,
        kind: &AsciiString,
    ) -> Result<(), Failure> {
        let name = String::from_utf8_lossy(name.as_bytes()).into_owned();
        let kind = String::from_utf8_lossy(kind.as_bytes()).into_owned();
        self.call(&format!("WriteRoot {name} {reference} {kind}"))
    }
    fn end_write_root_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndWriteRootSection")
    }
    fn begin_read_root_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginReadRootSection")
    }
    fn root_section_size(&mut self) -> Result<i32, Failure> {
        self.call("RootSectionSize")?;
        Ok(self.roots.len() as i32)
    }
    fn read_root(
        &mut self,
        name: &mut AsciiString,
        reference: &mut i32,
        kind: &mut AsciiString,
    ) -> Result<(), Failure> {
        self.call("ReadRoot")?;
        let (text, value, type_name) = self.roots.remove(0);
        *name = AsciiString::new(text);
        *reference = value;
        *kind = AsciiString::new(type_name);
        Ok(())
    }
    fn end_read_root_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndReadRootSection")
    }
    fn begin_write_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginWriteRefSection")
    }
    fn set_ref_section_size(&mut self, size: i32) -> Result<(), Failure> {
        self.call(&format!("SetRefSectionSize {size}"))
    }
    fn write_reference_type(&mut self, reference: i32, number: i32) -> Result<(), Failure> {
        self.call(&format!("WriteReferenceType {reference} {number}"))
    }
    fn end_write_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndWriteRefSection")
    }
    fn begin_read_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginReadRefSection")
    }
    fn ref_section_size(&mut self) -> Result<i32, Failure> {
        self.call("RefSectionSize")?;
        Ok(0)
    }
    fn read_reference_type(&mut self, _: &mut i32, _: &mut i32) -> Result<(), Failure> {
        self.call("ReadReferenceType")
    }
    fn end_read_ref_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndReadRefSection")
    }
}

impl DataSections for ScriptedDriver {
    fn begin_write_data_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginWriteDataSection")
    }
    fn write_persistent_object_header(
        &mut self,
        reference: i32,
        number: i32,
    ) -> Result<(), Failure> {
        self.call(&format!("WritePersistentObjectHeader {reference} {number}"))
    }
    fn begin_write_persistent_object_data(&mut self) -> Result<(), Failure> {
        self.call("BeginWritePersistentObjectData")
    }
    fn begin_write_object_data(&mut self) -> Result<(), Failure> {
        self.call("BeginWriteObjectData")
    }
    fn end_write_object_data(&mut self) -> Result<(), Failure> {
        self.call("EndWriteObjectData")
    }
    fn end_write_persistent_object_data(&mut self) -> Result<(), Failure> {
        self.call("EndWritePersistentObjectData")
    }
    fn end_write_data_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndWriteDataSection")
    }
    fn begin_read_data_section(&mut self) -> Result<StorageError, Failure> {
        self.step("BeginReadDataSection")
    }
    fn read_persistent_object_header(&mut self, _: &mut i32, _: &mut i32) -> Result<(), Failure> {
        self.call("ReadPersistentObjectHeader")
    }
    fn begin_read_persistent_object_data(&mut self) -> Result<(), Failure> {
        self.call("BeginReadPersistentObjectData")
    }
    fn begin_read_object_data(&mut self) -> Result<(), Failure> {
        self.call("BeginReadObjectData")
    }
    fn end_read_object_data(&mut self) -> Result<(), Failure> {
        self.call("EndReadObjectData")
    }
    fn end_read_persistent_object_data(&mut self) -> Result<(), Failure> {
        self.call("EndReadPersistentObjectData")
    }
    fn end_read_data_section(&mut self) -> Result<StorageError, Failure> {
        self.step("EndReadDataSection")
    }
}

impl DriverPrimitives for ScriptedDriver {
    fn put_reference(&mut self, value: i32) -> Result<(), Failure> {
        self.call(&format!("PutReference {value}"))
    }
    fn put_character(&mut self, value: u8) -> Result<(), Failure> {
        self.call(&format!("PutCharacter {value}"))
    }
    fn put_ext_character(&mut self, value: u16) -> Result<(), Failure> {
        self.call(&format!("PutExtCharacter {value}"))
    }
    fn put_integer(&mut self, value: i32) -> Result<(), Failure> {
        self.call(&format!("PutInteger {value}"))
    }
    fn put_boolean(&mut self, value: bool) -> Result<(), Failure> {
        self.call(&format!("PutBoolean {value}"))
    }
    fn put_real(&mut self, value: f64) -> Result<(), Failure> {
        self.call(&format!("PutReal {value}"))
    }
    fn put_short_real(&mut self, value: f32) -> Result<(), Failure> {
        self.call(&format!("PutShortReal {value}"))
    }
    fn get_reference(&mut self, value: &mut i32) -> Result<(), Failure> {
        *value = 11;
        self.call("GetReference")
    }
    fn get_character(&mut self, value: &mut u8) -> Result<(), Failure> {
        *value = b'c';
        self.call("GetCharacter")
    }
    fn get_ext_character(&mut self, value: &mut u16) -> Result<(), Failure> {
        *value = 0x20AC;
        self.call("GetExtCharacter")
    }
    fn get_integer(&mut self, value: &mut i32) -> Result<(), Failure> {
        *value = 12;
        self.call("GetInteger")
    }
    fn get_boolean(&mut self, value: &mut bool) -> Result<(), Failure> {
        *value = true;
        self.call("GetBoolean")
    }
    fn get_real(&mut self, value: &mut f64) -> Result<(), Failure> {
        *value = 1.5;
        self.call("GetReal")
    }
    fn get_short_real(&mut self, value: &mut f32) -> Result<(), Failure> {
        *value = 2.5;
        self.call("GetShortReal")
    }
}

impl BaseDriver for ScriptedDriver {
    fn driver_base(&self) -> &DriverBase {
        &self.base
    }
    fn driver_base_mut(&mut self) -> &mut DriverBase {
        &mut self.base
    }
    fn open(&mut self, name: &AsciiString, mode: OpenMode) -> Result<StorageError, Failure> {
        self.base.set_name(name);
        self.base.set_open_mode(mode);
        self.step("Open")
    }
    fn is_end(&mut self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn tell(&mut self) -> Result<Position, Failure> {
        Ok(self.log.len() as Position)
    }
    fn skip_object(&mut self) -> Result<(), Failure> {
        self.call("SkipObject")
    }
    fn close(&mut self) -> Result<StorageError, Failure> {
        self.base.set_open_mode(OpenMode::None);
        self.step("Close")
    }
}

/// A clock stuck on 2 March 2001.
pub struct FixedClock;

impl CreationDateSource for FixedClock {
    fn local_date(&self) -> (u32, u32, i32) {
        (3, 2, 2001)
    }
}

/// A migration source serving fixed table bytes.
pub struct TableFile(pub &'static [u8]);

impl MigrationSource for TableFile {
    fn migration_file(&self) -> MigrationFile {
        MigrationFile::Contents(self.0.to_vec())
    }
}

/// A migration source whose table path exists but cannot be read.
pub struct UnreadableTable;

impl MigrationSource for UnreadableTable {
    fn migration_file(&self) -> MigrationFile {
        MigrationFile::Unreadable
    }
}
