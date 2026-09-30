//! The `myErrorStatus`/`myErrorStatusExt` pair that every Storage data section keeps.
use super::StorageError;
use crate::tcollection::AsciiString;

/// Last error of a section and the name of the step that raised it.
///
/// ```
/// use rustycad_tkernel::storage::{SectionStatus, StorageError};
/// let mut status = SectionStatus::default();
/// assert!(!status.record(StorageError::FormatError, "BeginReadRootSection"));
/// assert_eq!(status.extension.as_bytes(), b"BeginReadRootSection");
/// ```
#[derive(Clone, Debug, Default)]
pub struct SectionStatus {
    /// `myErrorStatus`.
    pub error: StorageError,
    /// `myErrorStatusExt`.
    pub extension: AsciiString,
}

impl SectionStatus {
    /// The native `myErrorStatus = step(); if (myErrorStatus != Storage_VSOk) myErrorStatusExt = ...`:
    /// the status is always stored, the context only on failure. Returns whether reading goes on.
    /// Example: `status.record(driver.begin_read_type_section()?, "BeginReadTypeSection")`.
    pub fn record(&mut self, error: StorageError, context: &str) -> bool {
        self.error = error;
        if error == StorageError::Ok {
            return true;
        }
        self.extension = AsciiString::new(context);
        false
    }

    /// `ClearErrorStatus`: back to `Ok` with an empty extension; `status.clear()`.
    pub fn clear(&mut self) {
        self.error = StorageError::Ok;
        self.extension = AsciiString::default();
    }
}
