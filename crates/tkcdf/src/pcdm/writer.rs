// Rust port of ApplicationFramework/TKCDF/PCDM/PCDM_Writer.hxx, .cxx and
// PCDM_StorageDriver.hxx, .cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PCDM_Writer` and `PCDM_StorageDriver`: the writing side of a document format.
use super::PcdmStoreStatus;
use crate::cdm::CdmDocument;
use rustycad_tkernel::{
    impl_transient_interface,
    message::ProgressRange,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient},
    tcollection::ExtendedString,
};
use std::io;

/// `PCDM_Writer`: writes a document to a file or a stream.
pub trait PcdmWriter: Transient {
    /// `Write(aDocument, aFileName, theRange)` (pure virtual).
    fn write_file(
        &self,
        document: &Handle<dyn CdmDocument>,
        file_name: &ExtendedString,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure>;
    /// `Write(theDocument, theOStream, theRange)` (pure virtual).
    fn write_stream(
        &self,
        document: &Handle<dyn CdmDocument>,
        stream: &mut dyn io::Write,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure>;
}
impl_transient_interface!(dyn PcdmWriter, "PCDM_Writer", StandardTransient);

#[derive(Default)]
struct StorageDriverState {
    format: ExtendedString,
    // Native constructor leaves it uninitialised; the port starts false.
    is_error: bool,
    store_status: PcdmStoreStatus,
}

/// The private fields of `PCDM_StorageDriver`.
#[derive(Default)]
pub struct PcdmStorageDriverBase {
    state: SharedState<StorageDriverState>,
}

/// `PCDM_StorageDriver`: a writer registered for one format.
///
/// The native default `Write(aDocument, aFileName)` writes the Std container through
/// `FSD_CmpFile` and `Make` returns `PCDM_Document` persistents: both belong to the native
/// file formats, which are not ported (ocaf-design.md section 1). A concrete driver
/// implements [`PcdmWriter`] itself; its stream writer can delegate to
/// [`PcdmStorageDriver::storage_write_stream`], the native default.
pub trait PcdmStorageDriver: PcdmWriter {
    /// The embedded base with the native private fields.
    fn storage_driver_base(&self) -> &PcdmStorageDriverBase;
    /// The native `PCDM_StorageDriver::Write(theDocument, theOStream, theRange)`: writes nothing.
    fn storage_write_stream(
        &self,
        _document: &Handle<dyn CdmDocument>,
        _stream: &mut dyn io::Write,
        _range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `SetFormat`; `driver.set_format(&format)`.
    fn set_format(&self, format: &ExtendedString) {
        self.storage_driver_base().state.write().format = format.clone();
    }
    /// `GetFormat`; `driver.get_format()`.
    fn get_format(&self) -> ExtendedString {
        self.storage_driver_base().state.read().format.clone()
    }
    /// `IsError`; `driver.is_error()`.
    fn is_error(&self) -> bool {
        self.storage_driver_base().state.read().is_error
    }
    /// `SetIsError`; `driver.set_is_error(true)`.
    fn set_is_error(&self, is_error: bool) {
        self.storage_driver_base().state.write().is_error = is_error;
    }
    /// `GetStoreStatus`; `driver.get_store_status()`.
    fn get_store_status(&self) -> PcdmStoreStatus {
        self.storage_driver_base().state.read().store_status
    }
    /// `SetStoreStatus`; `driver.set_store_status(PcdmStoreStatus::Ok)`.
    fn set_store_status(&self, store_status: PcdmStoreStatus) {
        self.storage_driver_base().state.write().store_status = store_status;
    }
}
impl_transient_interface!(dyn PcdmStorageDriver, "PCDM_StorageDriver", dyn PcdmWriter);
