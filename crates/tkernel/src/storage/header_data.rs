//! `Storage_HeaderData`: the info and comment sections of a container.
use super::{BaseDriver, OpenMode, SectionStatus, StorageError, StreamException};
use crate::ncollection::Sequence;
use crate::standard::{Failure, StandardTransient};
use crate::tcollection::{AsciiString, ExtendedString};
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// The fields of the header, lent by [`HeaderData::fields`] and [`HeaderData::change_fields`].
#[derive(Default)]
pub struct HeaderFields {
    /// `myNBObj`.
    pub number_of_objects: i32,
    /// `myStorageVersion`.
    pub storage_version: AsciiString,
    /// `mySchemaVersion`.
    pub schema_version: AsciiString,
    /// `mySchemaName`.
    pub schema_name: AsciiString,
    /// `myApplicationVersion`.
    pub application_version: AsciiString,
    /// `myApplicationName`.
    pub application_name: ExtendedString,
    /// `myDataType`.
    pub data_type: ExtendedString,
    /// `myDate`.
    pub date: AsciiString,
    /// `myUserInfo`.
    pub user_info: Sequence<AsciiString>,
    /// `myComments`.
    pub comments: Sequence<ExtendedString>,
    /// `myErrorStatus` and `myErrorStatusExt`.
    pub status: SectionStatus,
}

/// Header of a set of stored data, shared by handles like the native transient.
///
/// ```
/// use rustycad_tkernel::storage::HeaderData;
/// use rustycad_tkernel::tcollection::AsciiString;
/// let header = HeaderData::default();
/// header.set_schema_name(&AsciiString::new("Schema"));
/// assert_eq!(header.schema_name().as_bytes(), b"Schema");
/// ```
#[derive(Default)]
pub struct HeaderData {
    parent: StandardTransient,
    fields: RwLock<HeaderFields>,
}

impl HeaderData {
    /// Read access to every field; `header.fields().comments.len()`.
    pub fn fields(&self) -> RwLockReadGuard<'_, HeaderFields> {
        self.fields.read().unwrap_or_else(PoisonError::into_inner)
    }

    /// Write access seen by every handle holder; `header.change_fields().number_of_objects = 1`.
    pub fn change_fields(&self) -> RwLockWriteGuard<'_, HeaderFields> {
        self.fields.write().unwrap_or_else(PoisonError::into_inner)
    }

    /// `Read(theDriver)`: the info then the comment section. Type-mismatch and
    /// ext-character parity exceptions become the status; any other failure propagates.
    /// Example: `header.read(&mut driver)?`.
    pub fn read(&self, driver: &mut dyn BaseDriver) -> Result<bool, Failure> {
        if !matches!(driver.open_mode(), OpenMode::Read | OpenMode::ReadWrite) {
            return Ok(self.fail(StorageError::ModeError, "OpenMode"));
        }
        if !self.step(driver.begin_read_info_section()?, "BeginReadInfoSection") {
            return Ok(false);
        }
        if !self.guarded(self.read_info(driver), "ReadInfo")? {
            return Ok(false);
        }
        if !self.step(driver.end_read_info_section()?, "EndReadInfoSection") {
            return Ok(false);
        }
        self.read_comments(driver)
    }

    fn read_comments(&self, driver: &mut dyn BaseDriver) -> Result<bool, Failure> {
        if !self.step(
            driver.begin_read_comment_section()?,
            "BeginReadCommentSection",
        ) {
            return Ok(false);
        }
        let comments = driver.read_comment(&mut self.change_fields().comments);
        if !self.guarded(comments, "ReadComment")? {
            return Ok(false);
        }
        Ok(self.step(driver.end_read_comment_section()?, "EndReadCommentSection"))
    }

    fn read_info(&self, driver: &mut dyn BaseDriver) -> Result<(), Failure> {
        let mut guard = self.change_fields();
        let fields = &mut *guard;
        driver.read_info(
            &mut fields.number_of_objects,
            &mut fields.storage_version,
            &mut fields.date,
            &mut fields.schema_name,
            &mut fields.schema_version,
            &mut fields.application_name,
            &mut fields.application_version,
            &mut fields.data_type,
            &mut fields.user_info,
        )
    }

    /// Records `status` with `context` unless it is `Ok`; returns whether reading goes on.
    fn step(&self, status: StorageError, context: &str) -> bool {
        self.change_fields().status.record(status, context)
    }

    /// The native `try { ... } catch` around one driver call.
    fn guarded(&self, outcome: Result<(), Failure>, context: &str) -> Result<bool, Failure> {
        match outcome {
            Ok(()) => Ok(true),
            Err(failure) if StreamException::TypeMismatch.is(&failure) => {
                Ok(self.fail(StorageError::TypeMismatch, context))
            }
            Err(failure) if StreamException::ExtCharParity.is(&failure) => {
                Ok(self.fail(StorageError::ExtCharParityError, context))
            }
            Err(failure) => Err(failure),
        }
    }

    fn fail(&self, status: StorageError, context: &str) -> bool {
        self.change_fields().status.record(status, context);
        false
    }
}

impl HeaderData {
    /// `CreationDate`; `header.creation_date()`.
    pub fn creation_date(&self) -> AsciiString {
        self.fields().date.clone()
    }

    /// `StorageVersion`; `header.storage_version()`.
    pub fn storage_version(&self) -> AsciiString {
        self.fields().storage_version.clone()
    }

    /// `SchemaVersion`; `header.schema_version()`.
    pub fn schema_version(&self) -> AsciiString {
        self.fields().schema_version.clone()
    }

    /// `SchemaName`; `header.schema_name()`.
    pub fn schema_name(&self) -> AsciiString {
        self.fields().schema_name.clone()
    }

    /// `SetApplicationVersion`; `header.set_application_version(&version)`.
    pub fn set_application_version(&self, version: &AsciiString) {
        self.change_fields().application_version = version.clone();
    }

    /// `ApplicationVersion`; `header.application_version()`.
    pub fn application_version(&self) -> AsciiString {
        self.fields().application_version.clone()
    }

    /// `SetApplicationName`; `header.set_application_name(&name)`.
    pub fn set_application_name(&self, name: &ExtendedString) {
        self.change_fields().application_name = name.clone();
    }

    /// `ApplicationName`; `header.application_name()`.
    pub fn application_name(&self) -> ExtendedString {
        self.fields().application_name.clone()
    }

    /// `SetDataType`; `header.set_data_type(&data_type)`.
    pub fn set_data_type(&self, data_type: &ExtendedString) {
        self.change_fields().data_type = data_type.clone();
    }

    /// `DataType`; `header.data_type()`.
    pub fn data_type(&self) -> ExtendedString {
        self.fields().data_type.clone()
    }

    /// `AddToUserInfo`; `header.add_to_user_info(&line)`.
    pub fn add_to_user_info(&self, info: &AsciiString) {
        self.change_fields().user_info.append(info.clone());
    }

    /// `UserInfo`, copied out of the lock; `header.user_info().len()`.
    pub fn user_info(&self) -> Sequence<AsciiString> {
        self.fields().user_info.clone()
    }

    /// `AddToComments`; `header.add_to_comments(&comment)`.
    pub fn add_to_comments(&self, comment: &ExtendedString) {
        self.change_fields().comments.append(comment.clone());
    }

    /// `Comments`, copied out of the lock; `header.comments().len()`.
    pub fn comments(&self) -> Sequence<ExtendedString> {
        self.fields().comments.clone()
    }

    /// `NumberOfObjects`; `header.number_of_objects()`.
    pub fn number_of_objects(&self) -> i32 {
        self.fields().number_of_objects
    }
}

impl HeaderData {
    /// `ErrorStatus`; `header.error_status()`.
    pub fn error_status(&self) -> StorageError {
        self.fields().status.error
    }

    /// `ErrorStatusExtension`: the step that failed; `header.error_status_extension()`.
    pub fn error_status_extension(&self) -> AsciiString {
        self.fields().status.extension.clone()
    }

    /// `ClearErrorStatus`; `header.clear_error_status()`.
    pub fn clear_error_status(&self) {
        self.change_fields().status.clear();
    }

    /// `SetNumberOfObjects`; `header.set_number_of_objects(2)`.
    pub fn set_number_of_objects(&self, count: i32) {
        self.change_fields().number_of_objects = count;
    }

    /// `SetStorageVersion(const TCollection_AsciiString&)`; `header.set_storage_version(&v)`.
    pub fn set_storage_version(&self, version: &AsciiString) {
        self.change_fields().storage_version = version.clone();
    }

    /// `SetStorageVersion(int)`, stored in decimal; `header.set_storage_version_number(3)`.
    pub fn set_storage_version_number(&self, version: i32) {
        self.set_storage_version(&AsciiString::from_integer(version));
    }

    /// `SetCreationDate`; `header.set_creation_date(&date)`.
    pub fn set_creation_date(&self, date: &AsciiString) {
        self.change_fields().date = date.clone();
    }

    /// `SetSchemaVersion`; `header.set_schema_version(&version)`.
    pub fn set_schema_version(&self, version: &AsciiString) {
        self.change_fields().schema_version = version.clone();
    }

    /// `SetSchemaName`; `header.set_schema_name(&name)`.
    pub fn set_schema_name(&self, name: &AsciiString) {
        self.change_fields().schema_name = name.clone();
    }
}

crate::impl_transient!(HeaderData, "Storage_HeaderData", StandardTransient, parent);
