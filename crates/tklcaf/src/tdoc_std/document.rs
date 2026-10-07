// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_Document.hxx, .lxx and .cxx
// (fields, constructor, accessors and CDM overrides) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDocStd_Document`: an OCAF document, a CDM document over a TDF data framework, with
//! its undo/redo stacks (ocaf-design.md section 6).
use super::{
    TDocStdContext, TDocStdFormatVersion, TDocStdModified, TDocStdOwner, TDocStdXLinkIterator,
};
use crate::live;
use crate::tdf::{
    TdfAttributeIterator, TdfAttributeState, TdfData, TdfDelta, TdfLabel, TdfLabelMap,
};
use rustycad_tkcdf::cdm::{
    CdmDocument, CdmDocumentBase, CdmDocumentReferences, CdmDocumentStorage, ModifContext,
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::{AsciiString, ExtendedString},
};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// The fields of `TDocStd_Document`.
pub(crate) struct DocumentState {
    pub(crate) storage_format: ExtendedString,
    pub(crate) undos: Vec<Handle<TdfDelta>>,
    pub(crate) redos: Vec<Handle<TdfDelta>>,
    pub(crate) data: Handle<TdfData>,
    pub(crate) undo_limit: i32,
    /// The transaction number the native `myUndoTransaction` ("UNDO") opened, 0 when closed.
    pub(crate) undo_transaction: i32,
    pub(crate) from_undo: Handle<TdfDelta>,
    pub(crate) from_redo: Handle<TdfDelta>,
    pub(crate) save_time: i32,
    pub(crate) nested_transaction_mode: bool,
    /// Native `myUndoFILO`, its first element at index 0.
    pub(crate) undo_filo: Vec<Handle<TdfDelta>>,
    pub(crate) only_transaction_modification: bool,
    pub(crate) save_empty_labels: bool,
    pub(crate) storage_format_version: TDocStdFormatVersion,
}

/// `TDocStd_Document`; `TDocStdDocument::new(&ExtendedString::new("BinOcaf"))?`.
///
/// ```
/// use rustycad_tklcaf::tdoc_std::TDocStdDocument;
/// use rustycad_tkernel::tcollection::ExtendedString;
/// let document = TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap();
/// assert!(document.is_empty().unwrap() && document.main().unwrap().tag() == 1);
/// assert!(TDocStdDocument::get(&document.main().unwrap()).unwrap() == document);
/// ```
pub struct TDocStdDocument {
    transient: StandardTransient,
    base: CdmDocumentBase,
    pub(crate) state: SharedState<DocumentState>,
}
impl_transient!(TDocStdDocument, "TDocStd_Document", dyn CdmDocument, transient, interfaces: [dyn CdmDocument]);

impl TDocStdDocument {
    /// `TDocStd_Document(astorageformat)`: a new framework owned by the document. The
    /// native constructor leaves `myOnlyTransactionModification` uninitialised; the port
    /// starts false.
    pub fn new(storage_format: &ExtendedString) -> Result<Handle<TDocStdDocument>, Failure> {
        let data = TdfData::new();
        let document = Handle::new(Self {
            transient: StandardTransient::default(),
            base: CdmDocumentBase::default(),
            state: SharedState::new(DocumentState {
                storage_format: storage_format.clone(),
                undos: Vec::new(),
                redos: Vec::new(),
                data: data.clone(),
                undo_limit: 0,
                undo_transaction: 0,
                from_undo: Handle::default(),
                from_redo: Handle::default(),
                save_time: 0,
                nested_transaction_mode: false,
                undo_filo: Vec::new(),
                only_transaction_modification: false,
                save_empty_labels: false,
                storage_format_version: TDocStdFormatVersion::CURRENT,
            }),
        });
        live(&document, "TDocStd_Document")?
            .base
            .bind_self(&document.cast());
        TDocStdOwner::set_document_of(&data, &document)?;
        Ok(document)
    }
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, DocumentState> {
        self.state.read()
    }
    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, DocumentState> {
        self.state.write()
    }
    /// The bound handle of this document (`this`).
    pub(crate) fn this_document(&self) -> Result<Handle<TDocStdDocument>, Failure> {
        Ok(self.base.this()?.cast())
    }
    /// `Get(L)`: the document owning the label's framework, null when the root has no
    /// attribute (to avoid a handle to a framework being destroyed).
    pub fn get(label: &TdfLabel) -> Result<Handle<TDocStdDocument>, Failure> {
        if !label.root()?.has_attribute()? {
            return Ok(Handle::default());
        }
        TDocStdOwner::get_document_of(&label.data())
    }
    /// `IsSaved`: the document is stored.
    pub fn is_saved(&self) -> bool {
        self.base.is_stored()
    }
    /// `IsChanged`: the framework time differs from the saved time.
    pub fn is_changed(&self) -> Result<bool, Failure> {
        let (data, saved) = {
            let s = self.read();
            (s.data.clone(), s.save_time)
        };
        Ok(live(&data, "TDocStd_Document::IsChanged")?.time() != saved)
    }
    /// `SetSaved`: the current framework time becomes the saved time.
    pub fn set_saved(&self) -> Result<(), Failure> {
        let data = self.get_data();
        let time = live(&data, "TDocStd_Document::SetSaved")?.time();
        self.write().save_time = time;
        Ok(())
    }
    /// `SetSavedTime`.
    pub fn set_saved_time(&self, time: i32) {
        self.write().save_time = time;
    }
    /// `GetSavedTime`.
    pub fn get_saved_time(&self) -> i32 {
        self.read().save_time
    }
    /// `GetName`: the meta-data name; `Standard_NoSuchObject` when not stored.
    pub fn get_name(&self) -> Result<ExtendedString, Failure> {
        Ok(live(&self.meta_data()?, "TDocStd_Document::GetName")?.name())
    }
    /// `GetPath`: the meta-data path; `Standard_NoSuchObject` when not stored.
    pub fn get_path(&self) -> Result<ExtendedString, Failure> {
        Ok(live(&self.meta_data()?, "TDocStd_Document::GetPath")?.path())
    }
    /// `SetData`: another framework (the undo transaction moves to it).
    pub fn set_data(&self, data: &Handle<TdfData>) -> Result<(), Failure> {
        super::document_commands::reinitialize_undo_transaction(self)?;
        self.write().data = data.clone();
        Ok(())
    }
    /// `GetData`.
    pub fn get_data(&self) -> Handle<TdfData> {
        self.read().data.clone()
    }
    /// `Main`: the label `0:1`, created when absent.
    pub fn main(&self) -> Result<TdfLabel, Failure> {
        live(&self.get_data(), "TDocStd_Document::Main")?
            .root()
            .find_child(1, true)
    }
    /// `IsEmpty`: the main label has no attribute.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        Ok(!TdfAttributeIterator::new(&self.main()?, true)?.more())
    }
    /// `IsValid`: no label is marked modified.
    pub fn is_valid(&self) -> Result<bool, Failure> {
        TDocStdModified::is_empty_at(&self.main()?)
    }
    /// `SetModified(L)`.
    pub fn set_modified(&self, label: &TdfLabel) -> Result<(), Failure> {
        TDocStdModified::add(label).map(|_| ())
    }
    /// `PurgeModified`: forgets the modified labels.
    pub fn purge_modified(&self) -> Result<(), Failure> {
        TDocStdModified::clear_at(&self.main()?)
    }
    /// `GetModified`: a copy of the modified labels; `Standard_DomainError` when none was
    /// ever marked.
    pub fn get_modified(&self) -> Result<TdfLabelMap, Failure> {
        TDocStdModified::get_at(&self.main()?)
    }
    /// `UpdateReferences(aDocEntry)`: updates the XLinks of that document entry and marks
    /// their labels modified.
    pub fn update_references(&self, document_entry: &AsciiString) -> Result<(), Failure> {
        let mut references = Vec::new();
        let mut links = TDocStdXLinkIterator::new(&self.this_document()?)?;
        while links.more() {
            let link = links.value();
            let link = live(&link, "TDocStd_Document::UpdateReferences")?;
            if link.document_entry() == *document_entry {
                references.push(link.update()?);
            }
            links.next()?;
        }
        for reference in references.iter().filter_map(Handle::get) {
            self.set_modified(&reference.label())?;
        }
        Ok(())
    }
    /// `Recompute`: nothing when valid, else forgets the modified labels.
    pub fn recompute(&self) -> Result<(), Failure> {
        if self.is_valid()? {
            return Ok(());
        }
        self.purge_modified()
    }
    /// `SetEmptyLabelsSavingMode`.
    pub fn set_empty_labels_saving_mode(&self, allowed: bool) {
        self.write().save_empty_labels = allowed;
    }
    /// `EmptyLabelsSavingMode`.
    pub fn empty_labels_saving_mode(&self) -> bool {
        self.read().save_empty_labels
    }
    /// `ChangeStorageFormat` (virtual natively, never overridden): reloads the resources of
    /// a new format.
    pub fn change_storage_format(
        &self,
        new_storage_format: &ExtendedString,
    ) -> Result<(), Failure> {
        if *new_storage_format == self.read().storage_format {
            return Ok(());
        }
        self.write().storage_format = new_storage_format.clone();
        self.base.set_resources_are_loaded(false);
        self.load_resources()
    }
    /// `StorageFormatVersion`.
    pub fn storage_format_version(&self) -> TDocStdFormatVersion {
        self.read().storage_format_version
    }
    /// `ChangeStorageFormatVersion`.
    pub fn change_storage_format_version(&self, version: TDocStdFormatVersion) {
        self.write().storage_format_version = version;
    }
    /// `CurrentStorageFormatVersion`: VERSION_12.
    pub fn current_storage_format_version() -> TDocStdFormatVersion {
        TDocStdFormatVersion::CURRENT
    }
}

impl CdmDocument for TDocStdDocument {
    fn document_base(&self) -> &CdmDocumentBase {
        &self.base
    }
    /// `StorageFormat`.
    fn storage_format(&self) -> ExtendedString {
        self.read().storage_format.clone()
    }
    /// `Update(aToDocument, aReferenceIdentifier, aModifContext)`: with a `TDocStd_Context`
    /// whose references changed, or an out-of-date reference, updates the XLinks of that
    /// entry. A context that is not a `TDocStd_Context` is `Standard_NullObject` (a null
    /// dereference natively).
    fn update_reference(
        &self,
        _to_document: &Handle<dyn CdmDocument>,
        reference_identifier: i32,
        modif_context: ModifContext<'_>,
    ) -> Result<(), Failure> {
        let context = modif_context
            .and_then(|context| context.downcast_ref::<TDocStdContext>())
            .ok_or_else(|| {
                exceptions::null_object(
                    "TDocStd_Document::Update: the context is not a TDocStd_Context",
                )
            })?;
        if context.modified_references() || !self.is_up_to_date(reference_identifier)? {
            self.update_references(&AsciiString::from_integer(reference_identifier))?;
            self.set_is_up_to_date(reference_identifier)?;
        }
        Ok(())
    }
}
