// Rust port of ApplicationFramework/TKCDF/CDM/CDM_Document.hxx and .cxx (virtual members,
// fields and constructor) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CDM_Document`: the abstract applicative document. Its virtual members are the trait
//! [`CdmDocument`]; its non-virtual members are the extension traits
//! [`super::CdmDocumentReferences`], [`super::CdmDocumentStorage`] and
//! [`super::CdmDocumentLifecycle`], implemented for every document.
use super::{CdmApplication, CdmMetaData, CdmReference};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{
        Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle, exceptions,
    },
    tcollection::ExtendedString,
};
use std::sync::{OnceLock, RwLockReadGuard, RwLockWriteGuard};

/// The native `void* const aModifContext` of the update protocol: an opaque object that
/// the framework passes through and an `Update` override may down cast (`TDocStd_Context`);
/// `None` is the null pointer.
pub type ModifContext<'a> = Option<&'a (dyn std::any::Any + Send + Sync)>;

/// `CDM_Document`'s virtual members; a concrete document embeds a [`CdmDocumentBase`].
///
/// ```
/// use rustycad_tkcdf::cdm::{CdmDocument, CdmDocumentBase, CdmDocumentLifecycle};
/// use rustycad_tkernel::{impl_transient, standard::{Handle, StandardTransient}};
/// use rustycad_tkernel::tcollection::ExtendedString;
/// #[derive(Default)]
/// struct Doc { transient: StandardTransient, base: CdmDocumentBase }
/// impl_transient!(Doc, "Doc", dyn CdmDocument, transient, interfaces: [dyn CdmDocument]);
/// impl CdmDocument for Doc {
///     fn document_base(&self) -> &CdmDocumentBase { &self.base }
///     fn storage_format(&self) -> ExtendedString { ExtendedString::new("Fmt") }
/// }
/// let doc: Handle<dyn CdmDocument> = Handle::new(Doc::default()).cast();
/// doc.document_base().bind_self(&doc);
/// doc.modify();
/// assert_eq!(doc.modifications(), 2);
/// ```
pub trait CdmDocument: Transient {
    /// The embedded base with the native private fields.
    fn document_base(&self) -> &CdmDocumentBase;
    /// `StorageFormat` (pure virtual): the key of the storage resources.
    fn storage_format(&self) -> ExtendedString;
    /// `Update(aToDocument, aReferenceIdentifier, aModifContext)`: called once per modified
    /// reference; does nothing by default.
    fn update_reference(
        &self,
        _to_document: &Handle<dyn CdmDocument>,
        _reference_identifier: i32,
        _modif_context: ModifContext<'_>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `Update(ErrorString)`: clears the error, calls [`CdmDocument::update`], succeeds.
    fn update_with_error(&self, error_string: &mut ExtendedString) -> bool {
        error_string.clear();
        self.update();
        true
    }
    /// `Update()`: the obsolete update hook; does nothing by default.
    fn update(&self) {}
    /// `Extensions`: empties the list by default.
    fn extensions(&self, extensions: &mut Vec<ExtendedString>) {
        extensions.clear();
    }
    /// `GetAlternativeDocument`: this document, valid when the format is its storage format.
    fn get_alternative_document(
        &self,
        format: &ExtendedString,
        alternative_document: &mut Handle<dyn CdmDocument>,
    ) -> Result<bool, Failure> {
        *alternative_document = self.document_base().this()?;
        Ok(*format == self.storage_format())
    }
    /// `CanCloseReference`: whether this document lets `document` close; true by default.
    fn can_close_reference(
        &self,
        _document: &Handle<dyn CdmDocument>,
        _reference_identifier: i32,
    ) -> bool {
        true
    }
    /// `CloseReference`: a referenced document is being closed; does nothing by default.
    fn close_reference(&self, _document: &Handle<dyn CdmDocument>, _reference_identifier: i32) {}
}
impl_transient_interface!(dyn CdmDocument, "CDM_Document", StandardTransient);

/// The private fields of `CDM_Document`, in native order.
pub(crate) struct DocumentState {
    pub(crate) resources_are_loaded: bool,
    pub(crate) comments: Vec<ExtendedString>,
    pub(crate) from_references: Vec<Handle<CdmReference>>,
    pub(crate) to_references: Vec<Handle<CdmReference>>,
    pub(crate) version: i32,
    pub(crate) actual_reference_identifier: i32,
    pub(crate) storage_version: i32,
    pub(crate) meta_data: Handle<CdmMetaData>,
    pub(crate) requested_comment: ExtendedString,
    pub(crate) requested_folder: ExtendedString,
    pub(crate) requested_folder_is_defined: bool,
    pub(crate) requested_name: ExtendedString,
    pub(crate) requested_name_is_defined: bool,
    pub(crate) requested_previous_version_is_defined: bool,
    pub(crate) requested_previous_version: ExtendedString,
    pub(crate) file_extension: ExtendedString,
    pub(crate) description: ExtendedString,
    pub(crate) file_extension_was_found: bool,
    pub(crate) description_was_found: bool,
    pub(crate) application: Handle<dyn CdmApplication>,
}

/// The base of every document: the native fields and the self handle (section 2);
/// `CdmDocumentBase::default()` is the protected `CDM_Document()` constructor.
pub struct CdmDocumentBase {
    this: OnceLock<WeakHandle<dyn CdmDocument>>,
    state: SharedState<DocumentState>,
}
impl Default for CdmDocumentBase {
    fn default() -> Self {
        Self {
            this: OnceLock::new(),
            state: SharedState::new(DocumentState {
                resources_are_loaded: false,
                comments: Vec::new(),
                from_references: Vec::new(),
                to_references: Vec::new(),
                version: 1,
                actual_reference_identifier: 0,
                storage_version: 0,
                meta_data: Handle::default(),
                requested_comment: ExtendedString::new(""),
                requested_folder: ExtendedString::default(),
                requested_folder_is_defined: false,
                requested_name: ExtendedString::default(),
                requested_name_is_defined: false,
                requested_previous_version_is_defined: false,
                requested_previous_version: ExtendedString::default(),
                file_extension: ExtendedString::default(),
                description: ExtendedString::default(),
                file_extension_was_found: false,
                description_was_found: false,
                application: Handle::default(),
            }),
        }
    }
}

impl CdmDocumentBase {
    /// Records the owning handle of this document, once (ocaf-design.md section 2);
    /// `document.document_base().bind_self(&document)`.
    pub fn bind_self(&self, document: &Handle<dyn CdmDocument>) {
        let _ = self.this.set(document.downgrade());
    }
    /// The handle `this` converts to natively; `Standard_NullObject` before `bind_self`.
    pub fn this(&self) -> Result<Handle<dyn CdmDocument>, Failure> {
        let this = self.this.get().map(WeakHandle::upgrade).unwrap_or_default();
        if this.is_null() {
            return Err(exceptions::null_object(
                "CDM_Document: the document has no owning handle; expected bind_self after creation",
            ));
        }
        Ok(this)
    }
    /// The protected `myResourcesAreLoaded` field; `base.resources_are_loaded()`.
    pub fn resources_are_loaded(&self) -> bool {
        self.state.read().resources_are_loaded
    }
    /// Assigns the protected `myResourcesAreLoaded` field (derived documents reset it).
    pub fn set_resources_are_loaded(&self, loaded: bool) {
        self.state.write().resources_are_loaded = loaded;
    }
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, DocumentState> {
        self.state.read()
    }
    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, DocumentState> {
        self.state.write()
    }
    /// `Modifications`: the modification counter.
    pub fn modifications(&self) -> i32 {
        self.state.read().version
    }
    /// `IsOpened()`: an application opened the document.
    pub fn is_opened(&self) -> bool {
        self.state.read().application.is_some()
    }
    /// `IsStored()`: the document has meta-data.
    pub fn is_stored(&self) -> bool {
        self.state.read().meta_data.is_some()
    }
    /// `IsReadOnly()`: the read-only flag of the meta-data of a stored document.
    pub fn is_read_only(&self) -> bool {
        let meta_data = self.state.read().meta_data.clone();
        meta_data.get().is_some_and(CdmMetaData::is_read_only)
    }
    /// Private `AddToReference`: appends a reference from this document.
    pub(crate) fn add_to_reference(&self, reference: &Handle<CdmReference>) {
        self.state.write().to_references.push(reference.clone());
    }
    /// Private `AddFromReference`: appends a reference to this document.
    pub(crate) fn add_from_reference(&self, reference: &Handle<CdmReference>) {
        self.state.write().from_references.push(reference.clone());
    }
    /// Private `RemoveFromReference`: removes the first reference to this document with
    /// that identifier.
    pub(crate) fn remove_from_reference(&self, reference_identifier: i32) {
        let mut state = self.state.write();
        let found = state
            .from_references
            .iter()
            .position(|reference| identifier_of(reference) == Some(reference_identifier));
        if let Some(index) = found {
            state.from_references.remove(index);
        }
    }
    /// A copy of the references from this document, so a caller can call into them
    /// without holding the lock.
    pub(crate) fn references_from_this(&self) -> Vec<Handle<CdmReference>> {
        self.state.read().to_references.clone()
    }
    /// A copy of the references to this document.
    pub(crate) fn references_to_this(&self) -> Vec<Handle<CdmReference>> {
        self.state.read().from_references.clone()
    }
}

impl Drop for CdmDocumentBase {
    /// `~CDM_Document`: the meta-data no longer has a retrieved document.
    fn drop(&mut self) {
        let meta_data = self.state.read().meta_data.clone();
        if let Some(meta_data) = meta_data.get() {
            meta_data.unset_document();
        }
    }
}

/// The identifier of a reference handle (`None` for a null handle, which the lists never hold).
pub(crate) fn identifier_of(reference: &Handle<CdmReference>) -> Option<i32> {
    reference.get().map(CdmReference::reference_identifier)
}
