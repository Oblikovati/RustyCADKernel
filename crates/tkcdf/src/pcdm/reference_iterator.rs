// Rust port of ApplicationFramework/TKCDF/PCDM/PCDM_ReferenceIterator.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PCDM_ReferenceIterator`: re-creates in a retrieved document the references stored in
//! its file.
use super::{PcdmReference, native_header};
use crate::cdm::{CdmApplication, CdmDocument, CdmDocumentReferences, CdmMetaData};
use crate::live;
use rustycad_tkernel::{
    impl_transient,
    message::Messenger,
    osd::{CadPath, File},
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::{AsciiString, ExtendedString},
};
use std::sync::Arc;

#[derive(Default)]
struct IteratorState {
    references: Vec<PcdmReference>,
    iterator: i32,
}

/// `PCDM_ReferenceIterator`; `PcdmReferenceIterator::new(messenger)`. No OCCT class
/// overrides its virtual members, so it is a struct.
pub struct PcdmReferenceIterator {
    transient: StandardTransient,
    state: SharedState<IteratorState>,
    message_driver: Arc<Messenger>,
}
impl_transient!(
    PcdmReferenceIterator,
    "PCDM_ReferenceIterator",
    StandardTransient,
    transient
);

impl PcdmReferenceIterator {
    /// `PCDM_ReferenceIterator(theMessageDriver)`.
    pub fn new(message_driver: &Arc<Messenger>) -> Self {
        Self {
            transient: StandardTransient::default(),
            state: SharedState::default(),
            message_driver: message_driver.clone(),
        }
    }
    /// `LoadReferences`: creates in the document one stored reference per reference of
    /// the meta-data's file, with the stored identifier and version.
    pub fn load_references(
        &self,
        document: &Handle<dyn CdmDocument>,
        meta_data: &Handle<CdmMetaData>,
        application: &Handle<dyn CdmApplication>,
        use_storage_configuration: bool,
    ) -> Result<(), Failure> {
        let target = live(document, "PCDM_ReferenceIterator::LoadReferences")?;
        let app = live(application, "PCDM_ReferenceIterator::LoadReferences")?;
        self.init(meta_data)?;
        while self.more() {
            let referenced = {
                let mut table = app.meta_data_look_up_table();
                self.meta_data(&mut table, use_storage_configuration)?
            };
            target.create_reference_with_identifier(
                &referenced,
                self.reference_identifier()?,
                application,
                self.document_version()?,
                use_storage_configuration,
            )?;
            self.next();
        }
        Ok(())
    }
    /// `Init`: reads the references of the meta-data's file (none for a file in no native
    /// format, see `pcdm::native_header`) and restarts at the first.
    pub fn init(&self, meta_data: &Handle<CdmMetaData>) -> Result<(), Failure> {
        let file_name = live(meta_data, "PCDM_ReferenceIterator::Init")?.file_name();
        let mut references = Vec::new();
        native_header::references(&file_name, &mut references, &self.message_driver);
        let mut state = self.state.write();
        state.references = references;
        state.iterator = 1;
        Ok(())
    }
    /// Private `More`.
    fn more(&self) -> bool {
        let state = self.state.read();
        state.iterator <= state.references.len() as i32
    }
    /// Private `Next`.
    fn next(&self) {
        self.state.write().iterator += 1;
    }
    fn current(&self) -> Result<PcdmReference, Failure> {
        let state = self.state.read();
        let index = state.iterator;
        usize::try_from(index - 1)
            .ok()
            .and_then(|position| state.references.get(position).cloned())
            .ok_or_else(|| {
                exceptions::out_of_range(&format!(
                    "PCDM_ReferenceIterator: index {index} outside 1..{}",
                    state.references.len()
                ))
            })
    }
    /// Private `MetaData`: the meta-data of the current reference's file; its folder and
    /// name split as on the pinned (non-Windows) platform, read-only when not writable.
    fn meta_data(
        &self,
        table: &mut crate::cdm::MetaDataLookUpTable,
        _use_storage_configuration: bool,
    ) -> Result<Handle<CdmMetaData>, Failure> {
        let file = self.current()?.file_name();
        let path = AsciiString::from_extended(&file, None);
        let (mut folder, name) = CadPath::folder_and_file_from_path(path.as_bytes());
        if !folder.is_empty() {
            folder.truncate(folder.length() - 1)?;
        }
        let read_only = !File::new(path.as_bytes()).is_writeable();
        let folder = ExtendedString::from_ascii_string(&folder, true);
        let name = ExtendedString::from_ascii_string(&name, true);
        Ok(CdmMetaData::look_up(
            table, &folder, &name, &file, &file, read_only,
        ))
    }
    /// Private `ReferenceIdentifier` of the current reference.
    fn reference_identifier(&self) -> Result<i32, Failure> {
        Ok(self.current()?.reference_identifier())
    }
    /// Private `DocumentVersion` of the current reference.
    fn document_version(&self) -> Result<i32, Failure> {
        Ok(self.current()?.document_version())
    }
}
