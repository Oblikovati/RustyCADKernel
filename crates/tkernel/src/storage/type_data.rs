//! `Storage_TypeData`: the type section, an indexed map from type names to type numbers.
use super::{BaseDriver, OpenMode, SectionStatus, StorageError, StreamException};
use crate::ncollection::{HSequence, IndexedDataMap};
use crate::standard::{Failure, Handle, StandardTransient};
use crate::tcollection::AsciiString;
use std::sync::{Mutex, MutexGuard, PoisonError};

#[derive(Default)]
struct TypeFields {
    types: IndexedDataMap<AsciiString, i32>,
    status: SectionStatus,
}

/// Types used by a set of data, in insertion order.
///
/// ```
/// use rustycad_tkernel::storage::TypeData;
/// use rustycad_tkernel::tcollection::AsciiString;
/// let types = TypeData::default();
/// types.add_type(&AsciiString::new("PColStd"), 4);
/// assert_eq!(types.type_number(&AsciiString::new("PColStd")).unwrap(), 4);
/// assert_eq!(types.type_name(1).unwrap().as_bytes(), b"PColStd");
/// ```
#[derive(Default)]
pub struct TypeData {
    parent: StandardTransient,
    fields: Mutex<TypeFields>,
}

impl TypeData {
    fn lock(&self) -> MutexGuard<'_, TypeFields> {
        self.fields.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// `Read(theDriver)`: adds every entry of the type section; a type mismatch in
    /// `ReadTypeInformations` becomes the status, `TypeSectionSize` failures propagate.
    /// Example: `types.read(&mut driver)?`.
    pub fn read(&self, driver: &mut dyn BaseDriver) -> Result<bool, Failure> {
        if !matches!(driver.open_mode(), OpenMode::Read | OpenMode::ReadWrite) {
            return Ok(self.record(StorageError::ModeError, "OpenMode"));
        }
        let status = driver.begin_read_type_section()?;
        if !self.record(status, "BeginReadTypeSection") {
            return Ok(false);
        }
        let (mut number, mut name) = (0, AsciiString::default());
        let length = driver.type_section_size()?;
        for _ in 1..=length {
            match driver.read_type_informations(&mut number, &mut name) {
                Err(failure) if StreamException::TypeMismatch.is(&failure) => {
                    return Ok(self.record(StorageError::TypeMismatch, "ReadTypeInformations"));
                }
                outcome => outcome?,
            }
            self.lock().types.add(name.clone(), number);
        }
        let status = driver.end_read_type_section()?;
        Ok(self.record(status, "EndReadTypeSection"))
    }

    fn record(&self, error: StorageError, context: &str) -> bool {
        self.lock().status.record(error, context)
    }

    /// `NumberOfTypes`; `types.number_of_types()`.
    pub fn number_of_types(&self) -> i32 {
        self.lock().types.extent() as i32
    }

    /// `AddType`: an existing name keeps its first number; `types.add_type(&name, 2)`.
    pub fn add_type(&self, name: &AsciiString, number: i32) {
        self.lock().types.add(name.clone(), number);
    }

    /// `Type(int)`: the name at insertion index `number`, `Standard_NoSuchObject`
    /// outside `1..=NumberOfTypes`; `types.type_name(1)?`.
    pub fn type_name(&self, number: i32) -> Result<AsciiString, Failure> {
        let fields = self.lock();
        if number > 0 && number as usize <= fields.types.extent() {
            return fields.types.find_key(number as usize).cloned();
        }
        Err(Failure::with_type(
            "Standard_NoSuchObject",
            &format!(
                "Storage_TypeData::Type - aTypeNum not in range: {number}; expected 1..={}",
                fields.types.extent()
            ),
            "",
        ))
    }

    /// `Type(const TCollection_AsciiString&)`: the stored type number, or
    /// `Standard_NoSuchObject`; `types.type_number(&name)?`.
    pub fn type_number(&self, name: &AsciiString) -> Result<i32, Failure> {
        let fields = self.lock();
        fields.types.seek(name).copied().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                &format!(
                    "Storage_TypeData::Type - aTypeName not found: {:?}; expected a stored type",
                    String::from_utf8_lossy(name.as_bytes())
                ),
                "",
            )
        })
    }

    /// `IsType`; `types.is_type(&name)`.
    pub fn is_type(&self, name: &AsciiString) -> bool {
        self.lock().types.contains(name)
    }

    /// `Types`: names in insertion order; `types.types()`.
    pub fn types(&self) -> Handle<HSequence<AsciiString>> {
        let sequence = HSequence::default();
        for (name, _) in self.lock().types.items() {
            sequence.append(name.clone());
        }
        Handle::new(sequence)
    }

    /// `ErrorStatus`; `types.error_status()`.
    pub fn error_status(&self) -> StorageError {
        self.lock().status.error
    }

    /// `ErrorStatusExtension`; `types.error_status_extension()`.
    pub fn error_status_extension(&self) -> AsciiString {
        self.lock().status.extension.clone()
    }

    /// `ClearErrorStatus`; `types.clear_error_status()`.
    pub fn clear_error_status(&self) {
        self.lock().status.clear();
    }

    /// `Clear`: forgets every type, keeping the status; `types.clear()`.
    pub fn clear(&self) {
        self.lock().types.clear(false);
    }
}

crate::impl_transient!(TypeData, "Storage_TypeData", StandardTransient, parent);
