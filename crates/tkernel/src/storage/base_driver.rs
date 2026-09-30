//! `Storage_BaseDriver`: the abstract container driver.
//!
//! The native class declares every step as a pure virtual that may throw. Rust splits the
//! steps over the supertraits of [`BaseDriver`] (header sections, table sections, data
//! section, scalar I/O) so each driver implements them in focused modules, and every step
//! returns `Result` so a driver exception can propagate through the schema and data readers.
use super::{Data, IoStream, OpenMode, Position, StorageError};
use crate::ncollection::Sequence;
use crate::standard::{Failure, Handle};
use crate::tcollection::{AsciiString, ExtendedString};
mod scalars;
pub use scalars::{DriverPrimitives, DriverValue};

/// The protected state of `Storage_BaseDriver`: `myName` and `myOpenMode`.
///
/// ```
/// use rustycad_tkernel::storage::{DriverBase, OpenMode};
/// let base = DriverBase::default();
/// assert_eq!(base.open_mode(), OpenMode::None);
/// ```
#[derive(Clone, Debug, Default)]
pub struct DriverBase {
    name: AsciiString,
    open_mode: OpenMode,
}

impl DriverBase {
    /// `Name`; `base.name()`.
    pub fn name(&self) -> AsciiString {
        self.name.clone()
    }

    /// `OpenMode`; `base.open_mode()`.
    pub fn open_mode(&self) -> OpenMode {
        self.open_mode
    }

    /// Protected `SetName`; `base.set_name(&AsciiString::new("file.fsd"))`.
    pub fn set_name(&mut self, name: &AsciiString) {
        self.name = name.clone();
    }

    /// Protected `SetOpenMode`; `base.set_open_mode(OpenMode::Read)`.
    pub fn set_open_mode(&mut self, mode: OpenMode) {
        self.open_mode = mode;
    }

    /// `ReadMagicNumber`: up to seven characters (`BINFILE`, `CMPFILE`, `FSDFILE`) while the
    /// stream is good. A failed `get` appends the previous character again, as natively; the
    /// native first character is uninitialised, Rust starts from NUL, which appends nothing.
    /// Example: `DriverBase::read_magic_number(&mut stream)`.
    pub fn read_magic_number(stream: &mut IoStream) -> AsciiString {
        const MAGIC_NUMBER_LENGTH: i64 = 7;
        let mut magic = AsciiString::default();
        let mut character = 0_u8;
        let mut count = 0;
        while stream.good() && count < MAGIC_NUMBER_LENGTH {
            stream.get(&mut character);
            count += stream.gcount();
            magic.assign_cat(character);
        }
        magic
    }
}

/// Info and comment sections of a container.
pub trait HeaderSections {
    /// `BeginWriteInfoSection`; `driver.begin_write_info_section()?`.
    fn begin_write_info_section(&mut self) -> Result<StorageError, Failure>;

    /// `WriteInfo`: object count, versions, date, schema, application, data type and user
    /// info lines; `driver.write_info(1, &version, &date, ...)?`.
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<(), Failure>;

    /// `EndWriteInfoSection`; `driver.end_write_info_section()?`.
    fn end_write_info_section(&mut self) -> Result<StorageError, Failure>;

    /// `BeginReadInfoSection`; `driver.begin_read_info_section()?`.
    fn begin_read_info_section(&mut self) -> Result<StorageError, Failure>;

    /// `ReadInfo`: fills the out-parameters in `WriteInfo` order. Like the native
    /// references, fields read before a failure keep their new values.
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<(), Failure>;

    /// `ReadCompleteInfo`: reads a whole container from `stream` into `data`, creating it when
    /// null; `driver.read_complete_info(&mut stream, &mut data)?`.
    fn read_complete_info(
        &mut self,
        stream: &mut IoStream,
        data: &mut Handle<Data>,
    ) -> Result<(), Failure>;

    /// `EndReadInfoSection`; `driver.end_read_info_section()?`.
    fn end_read_info_section(&mut self) -> Result<StorageError, Failure>;

    /// `BeginWriteCommentSection`; `driver.begin_write_comment_section()?`.
    fn begin_write_comment_section(&mut self) -> Result<StorageError, Failure>;

    /// `WriteComment`; `driver.write_comment(&comments)?`.
    fn write_comment(&mut self, comments: &Sequence<ExtendedString>) -> Result<(), Failure>;

    /// `EndWriteCommentSection`; `driver.end_write_comment_section()?`.
    fn end_write_comment_section(&mut self) -> Result<StorageError, Failure>;

    /// `BeginReadCommentSection`; `driver.begin_read_comment_section()?`.
    fn begin_read_comment_section(&mut self) -> Result<StorageError, Failure>;

    /// `ReadComment`: appends the comments to `comments`; `driver.read_comment(&mut comments)?`.
    fn read_comment(&mut self, comments: &mut Sequence<ExtendedString>) -> Result<(), Failure>;

    /// `EndReadCommentSection`; `driver.end_read_comment_section()?`.
    fn end_read_comment_section(&mut self) -> Result<StorageError, Failure>;
}

/// Type, root and reference sections.
pub trait TableSections {
    /// `BeginWriteTypeSection`; `driver.begin_write_type_section()?`.
    fn begin_write_type_section(&mut self) -> Result<StorageError, Failure>;
    /// `SetTypeSectionSize`; `driver.set_type_section_size(2)?`.
    fn set_type_section_size(&mut self, size: i32) -> Result<(), Failure>;
    /// `WriteTypeInformations`; `driver.write_type_informations(1, &name)?`.
    fn write_type_informations(&mut self, number: i32, name: &AsciiString) -> Result<(), Failure>;
    /// `EndWriteTypeSection`; `driver.end_write_type_section()?`.
    fn end_write_type_section(&mut self) -> Result<StorageError, Failure>;
    /// `BeginReadTypeSection`; `driver.begin_read_type_section()?`.
    fn begin_read_type_section(&mut self) -> Result<StorageError, Failure>;
    /// `TypeSectionSize`; `driver.type_section_size()?`.
    fn type_section_size(&mut self) -> Result<i32, Failure>;
    /// `ReadTypeInformations`; `driver.read_type_informations(&mut number, &mut name)?`.
    fn read_type_informations(
        &mut self,
        number: &mut i32,
        name: &mut AsciiString,
    ) -> Result<(), Failure>;
    /// `EndReadTypeSection`; `driver.end_read_type_section()?`.
    fn end_read_type_section(&mut self) -> Result<StorageError, Failure>;

    /// `BeginWriteRootSection`; `driver.begin_write_root_section()?`.
    fn begin_write_root_section(&mut self) -> Result<StorageError, Failure>;
    /// `SetRootSectionSize`; `driver.set_root_section_size(1)?`.
    fn set_root_section_size(&mut self, size: i32) -> Result<(), Failure>;
    /// `WriteRoot`; `driver.write_root(&name, 1, &type_name)?`.
    fn write_root(
        &mut self,
        name: &AsciiString,
        reference: i32,
        type_name: &AsciiString,
    ) -> Result<(), Failure>;
    /// `EndWriteRootSection`; `driver.end_write_root_section()?`.
    fn end_write_root_section(&mut self) -> Result<StorageError, Failure>;
    /// `BeginReadRootSection`; `driver.begin_read_root_section()?`.
    fn begin_read_root_section(&mut self) -> Result<StorageError, Failure>;
    /// `RootSectionSize`; `driver.root_section_size()?`.
    fn root_section_size(&mut self) -> Result<i32, Failure>;
    /// `ReadRoot`; `driver.read_root(&mut name, &mut reference, &mut type_name)?`.
    fn read_root(
        &mut self,
        name: &mut AsciiString,
        reference: &mut i32,
        type_name: &mut AsciiString,
    ) -> Result<(), Failure>;
    /// `EndReadRootSection`; `driver.end_read_root_section()?`.
    fn end_read_root_section(&mut self) -> Result<StorageError, Failure>;

    /// `BeginWriteRefSection`; `driver.begin_write_ref_section()?`.
    fn begin_write_ref_section(&mut self) -> Result<StorageError, Failure>;
    /// `SetRefSectionSize`; `driver.set_ref_section_size(3)?`.
    fn set_ref_section_size(&mut self, size: i32) -> Result<(), Failure>;
    /// `WriteReferenceType`; `driver.write_reference_type(1, 2)?`.
    fn write_reference_type(&mut self, reference: i32, type_number: i32) -> Result<(), Failure>;
    /// `EndWriteRefSection`; `driver.end_write_ref_section()?`.
    fn end_write_ref_section(&mut self) -> Result<StorageError, Failure>;
    /// `BeginReadRefSection`; `driver.begin_read_ref_section()?`.
    fn begin_read_ref_section(&mut self) -> Result<StorageError, Failure>;
    /// `RefSectionSize`; `driver.ref_section_size()?`.
    fn ref_section_size(&mut self) -> Result<i32, Failure>;
    /// `ReadReferenceType`; `driver.read_reference_type(&mut reference, &mut number)?`.
    fn read_reference_type(
        &mut self,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure>;
    /// `EndReadRefSection`; `driver.end_read_ref_section()?`.
    fn end_read_ref_section(&mut self) -> Result<StorageError, Failure>;
}

/// The data section and the per-object framing.
pub trait DataSections {
    /// `BeginWriteDataSection`; `driver.begin_write_data_section()?`.
    fn begin_write_data_section(&mut self) -> Result<StorageError, Failure>;
    /// `WritePersistentObjectHeader`; `driver.write_persistent_object_header(1, 2)?`.
    fn write_persistent_object_header(
        &mut self,
        reference: i32,
        type_number: i32,
    ) -> Result<(), Failure>;
    /// `BeginWritePersistentObjectData`; `driver.begin_write_persistent_object_data()?`.
    fn begin_write_persistent_object_data(&mut self) -> Result<(), Failure>;
    /// `BeginWriteObjectData`; `driver.begin_write_object_data()?`.
    fn begin_write_object_data(&mut self) -> Result<(), Failure>;
    /// `EndWriteObjectData`; `driver.end_write_object_data()?`.
    fn end_write_object_data(&mut self) -> Result<(), Failure>;
    /// `EndWritePersistentObjectData`; `driver.end_write_persistent_object_data()?`.
    fn end_write_persistent_object_data(&mut self) -> Result<(), Failure>;
    /// `EndWriteDataSection`; `driver.end_write_data_section()?`.
    fn end_write_data_section(&mut self) -> Result<StorageError, Failure>;
    /// `BeginReadDataSection`; `driver.begin_read_data_section()?`.
    fn begin_read_data_section(&mut self) -> Result<StorageError, Failure>;
    /// `ReadPersistentObjectHeader`; `driver.read_persistent_object_header(&mut r, &mut t)?`.
    fn read_persistent_object_header(
        &mut self,
        reference: &mut i32,
        type_number: &mut i32,
    ) -> Result<(), Failure>;
    /// `BeginReadPersistentObjectData`; `driver.begin_read_persistent_object_data()?`.
    fn begin_read_persistent_object_data(&mut self) -> Result<(), Failure>;
    /// `BeginReadObjectData`; `driver.begin_read_object_data()?`.
    fn begin_read_object_data(&mut self) -> Result<(), Failure>;
    /// `EndReadObjectData`; `driver.end_read_object_data()?`.
    fn end_read_object_data(&mut self) -> Result<(), Failure>;
    /// `EndReadPersistentObjectData`; `driver.end_read_persistent_object_data()?`.
    fn end_read_persistent_object_data(&mut self) -> Result<(), Failure>;
    /// `EndReadDataSection`; `driver.end_read_data_section()?`.
    fn end_read_data_section(&mut self) -> Result<StorageError, Failure>;
}

/// `Storage_BaseDriver`: a container driver; `&mut dyn BaseDriver` is the native handle.
pub trait BaseDriver: HeaderSections + TableSections + DataSections + DriverPrimitives {
    /// The protected `myName`/`myOpenMode`; `driver.driver_base().name()`.
    fn driver_base(&self) -> &DriverBase;

    /// Mutable protected state, for `SetName`/`SetOpenMode`; `driver.driver_base_mut()`.
    fn driver_base_mut(&mut self) -> &mut DriverBase;

    /// `Name`; `driver.name()`.
    fn name(&self) -> AsciiString {
        self.driver_base().name()
    }

    /// `OpenMode`; `driver.open_mode()`.
    fn open_mode(&self) -> OpenMode {
        self.driver_base().open_mode()
    }

    /// `Open`: opens the named container; `driver.open(&name, OpenMode::Read)?`.
    fn open(&mut self, name: &AsciiString, mode: OpenMode) -> Result<StorageError, Failure>;

    /// `IsEnd`: whether the container reached its end; `driver.is_end()?`.
    fn is_end(&mut self) -> Result<bool, Failure>;

    /// `Tell`: the position, -1 on error; `driver.tell()?`.
    fn tell(&mut self) -> Result<Position, Failure>;

    /// `SkipObject`: skips the data of an object of unknown type; `driver.skip_object()?`.
    fn skip_object(&mut self) -> Result<(), Failure>;

    /// `Close`; `driver.close()?`.
    fn close(&mut self) -> Result<StorageError, Failure>;
}
