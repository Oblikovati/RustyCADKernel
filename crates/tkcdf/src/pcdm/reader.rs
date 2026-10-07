// Rust port of ApplicationFramework/TKCDF/PCDM/PCDM_Reader.hxx, .lxx, .cxx and
// PCDM_RetrievalDriver.hxx, .cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PCDM_Reader` and `PCDM_RetrievalDriver`: the reading side of a document format.
use super::{PcdmReaderFilter, PcdmReaderStatus, native_header};
use crate::cdm::{CdmApplication, CdmDocument};
use rustycad_tkernel::{
    impl_transient_interface,
    message::{Messenger, ProgressRange},
    standard::{Failure, Handle, SharedState, StandardTransient, Transient},
    storage::Data,
    tcollection::ExtendedString,
};
use std::{io, sync::Arc};

/// The protected `myReaderStatus` of `PCDM_Reader`. Native code leaves it uninitialised
/// until a reader sets it; the port starts at `PCDM_RS_OK`.
#[derive(Default)]
pub struct PcdmReaderBase {
    status: SharedState<PcdmReaderStatus>,
}
impl PcdmReaderBase {
    /// Reads the status; `base.status()`.
    pub fn status(&self) -> PcdmReaderStatus {
        *self.status.read()
    }
    /// Assigns the protected status, as derived readers do; `base.set_status(status)`.
    pub fn set_status(&self, status: PcdmReaderStatus) {
        *self.status.write() = status;
    }
}

/// `PCDM_Reader`: reads a document from a file or a stream.
pub trait PcdmReader: Transient {
    /// The embedded base with the native protected field.
    fn reader_base(&self) -> &PcdmReaderBase;
    /// `Read(aFileName, aNewDocument, anApplication, theFilter, theProgress)` (pure virtual).
    fn read_file(
        &self,
        file_name: &ExtendedString,
        new_document: &Handle<dyn CdmDocument>,
        application: &Handle<dyn CdmApplication>,
        filter: &Handle<PcdmReaderFilter>,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure>;
    /// `Read(theIStream, theStorageData, theDoc, theApplication, theFilter, theProgress)`
    /// (pure virtual).
    #[allow(clippy::too_many_arguments)]
    fn read_stream(
        &self,
        stream: &mut dyn io::Read,
        storage_data: &Handle<Data>,
        document: &Handle<dyn CdmDocument>,
        application: &Handle<dyn CdmApplication>,
        filter: &Handle<PcdmReaderFilter>,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure>;
    /// `GetStatus`: the status of the last read.
    fn get_status(&self) -> PcdmReaderStatus {
        self.reader_base().status()
    }
}
impl_transient_interface!(dyn PcdmReader, "PCDM_Reader", StandardTransient);

/// The private `myFormat` of `PCDM_RetrievalDriver`, and its static members.
#[derive(Default)]
pub struct PcdmRetrievalDriverBase {
    format: SharedState<ExtendedString>,
}
impl PcdmRetrievalDriverBase {
    /// `DocumentVersion(theFileName, theMsgDriver)`: the modification counter stored in a
    /// native file header; -1 for a file in no native format, the only kind the port reads
    /// (ocaf-design.md section 1).
    pub fn document_version(file_name: &ExtendedString, messenger: &Arc<Messenger>) -> i32 {
        native_header::document_version(file_name, messenger)
    }
    /// `ReferenceCounter(theFileName, theMsgDriver)`: the reference counter stored in a
    /// native file header; 0 for a file in no native format.
    pub fn reference_counter(file_name: &ExtendedString, messenger: &Arc<Messenger>) -> i32 {
        native_header::reference_counter(file_name, messenger)
    }
}

/// `PCDM_RetrievalDriver`: a reader registered for one format.
pub trait PcdmRetrievalDriver: PcdmReader {
    /// The embedded base with the native private field.
    fn retrieval_driver_base(&self) -> &PcdmRetrievalDriverBase;
    /// `SetFormat`; `driver.set_format(&format)`.
    fn set_format(&self, format: &ExtendedString) {
        *self.retrieval_driver_base().format.write() = format.clone();
    }
    /// `GetFormat`; `driver.get_format()`.
    fn get_format(&self) -> ExtendedString {
        self.retrieval_driver_base().format.read().clone()
    }
}
impl_transient_interface!(
    dyn PcdmRetrievalDriver,
    "PCDM_RetrievalDriver",
    dyn PcdmReader
);
