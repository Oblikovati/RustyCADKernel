// Rust port of ApplicationFramework/TKCDF/CDM/CDM_Reference.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CDM_Reference`: a link from one document to another, in session or stored.
use super::{CdmApplication, CdmDocument, CdmDocumentLifecycle, CdmMetaData};
use crate::json_dump::{class_begin, field_dumped, field_integer};
use crate::live;
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle, dump::DumpStream},
};
use std::io;

#[derive(Clone, Default)]
struct ReferenceState {
    to_document: Handle<dyn CdmDocument>,
    // Native `CDM_Document* myFromDocument`: a raw pointer, so a weak handle (section 6).
    from_document: WeakHandle<dyn CdmDocument>,
    reference_identifier: i32,
    application: Handle<dyn CdmApplication>,
    meta_data: Handle<CdmMetaData>,
    document_version: i32,
    use_storage_configuration: bool,
}

/// `CDM_Reference`: created by the documents only (private constructors, friend
/// `CDM_Document`); `document.reference(id)` returns one.
pub struct CdmReference {
    transient: StandardTransient,
    state: SharedState<ReferenceState>,
}
impl_transient!(CdmReference, "CDM_Reference", StandardTransient, transient);

impl CdmReference {
    fn create(state: ReferenceState) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }
    /// Private constructor to a document in session.
    pub(crate) fn to_document_in_session(
        from_document: &Handle<dyn CdmDocument>,
        to_document: &Handle<dyn CdmDocument>,
        reference_identifier: i32,
        to_document_version: i32,
    ) -> Handle<Self> {
        Self::create(ReferenceState {
            to_document: to_document.clone(),
            from_document: from_document.downgrade(),
            reference_identifier,
            document_version: to_document_version,
            ..ReferenceState::default()
        })
    }
    /// Private constructor to a stored document that is not retrieved yet.
    pub(crate) fn to_stored_document(
        from_document: &Handle<dyn CdmDocument>,
        meta_data: &Handle<CdmMetaData>,
        reference_identifier: i32,
        application: &Handle<dyn CdmApplication>,
        to_document_version: i32,
        use_storage_configuration: bool,
    ) -> Handle<Self> {
        Self::create(ReferenceState {
            from_document: from_document.downgrade(),
            reference_identifier,
            application: application.clone(),
            meta_data: meta_data.clone(),
            document_version: to_document_version,
            use_storage_configuration,
            ..ReferenceState::default()
        })
    }
    /// `FromDocument`: the referencing document, null when it is gone.
    pub fn from_document(&self) -> Handle<dyn CdmDocument> {
        self.state.read().from_document.upgrade()
    }
    /// `ToDocument`: the referenced document; a stored one is retrieved through the
    /// application on first access, which is then forgotten. A null application of an
    /// unretrieved reference is `Standard_NullObject` (a crash natively).
    pub fn to_document(&self) -> Result<Handle<dyn CdmDocument>, Failure> {
        let state = self.state.snapshot();
        if state.to_document.is_some() {
            return Ok(state.to_document);
        }
        let application = live(&state.application, "CDM_Reference::ToDocument")?;
        let retrieved = application.retrieve(
            &state.meta_data,
            state.use_storage_configuration,
            &Handle::default(),
            &ProgressRange::default(),
        )?;
        let mut written = self.state.write();
        written.to_document = retrieved.clone();
        written.application = Handle::default();
        Ok(retrieved)
    }
    /// `ReferenceIdentifier`: unique in the referencing document.
    pub fn reference_identifier(&self) -> i32 {
        self.state.read().reference_identifier
    }
    /// Private `Update`: attaches the document of `meta_data` when it is the one this
    /// unretrieved reference waits for, and registers this reference on it.
    pub(crate) fn update(
        this: &Handle<Self>,
        meta_data: &Handle<CdmMetaData>,
    ) -> Result<(), Failure> {
        let own = live(this, "CDM_Reference::Update")?;
        let document = {
            let state = own.state.read();
            if state.to_document.is_some() || state.meta_data != *meta_data {
                return Ok(());
            }
            live(&state.meta_data, "CDM_Reference::Update")?.document()
        };
        live(&document, "CDM_Reference::Update")?
            .document_base()
            .add_from_reference(this);
        let mut state = own.state.write();
        state.to_document = document;
        state.application = Handle::default();
        Ok(())
    }
    /// The actual version of the referenced document: its counter in session, else the
    /// one its meta-data reports.
    fn actual_document_version(&self) -> Result<i32, Failure> {
        let state = self.state.snapshot();
        match state.to_document.get() {
            Some(document) => Ok(document.document_base().modifications()),
            None => CdmMetaData::document_version(&state.meta_data, &state.application),
        }
    }
    /// Private `IsUpToDate`: the stored version equals the actual one.
    pub(crate) fn is_up_to_date(&self) -> Result<bool, Failure> {
        let actual = self.actual_document_version()?;
        Ok(self.state.read().document_version == actual)
    }
    /// Private `SetIsUpToDate`: takes the actual version, unless it is -1 (unknown).
    pub(crate) fn set_is_up_to_date(&self) -> Result<(), Failure> {
        let actual = self.actual_document_version()?;
        if actual != -1 {
            self.state.write().document_version = actual;
        }
        Ok(())
    }
    /// Private `UnsetToDocument`: forgets the document in session, keeping how to retrieve it.
    pub(crate) fn unset_to_document(
        &self,
        meta_data: &Handle<CdmMetaData>,
        application: &Handle<dyn CdmApplication>,
    ) {
        let mut state = self.state.write();
        state.to_document = Handle::default();
        state.application = application.clone();
        state.meta_data = meta_data.clone();
    }
    /// `DocumentVersion`: the referenced version at creation or last update.
    pub fn document_version(&self) -> i32 {
        self.state.read().document_version
    }
    /// Private `IsOpened`: the referenced document is in session and opened.
    pub(crate) fn is_opened(&self) -> bool {
        let document = self.document();
        document
            .get()
            .is_some_and(|document| document.document_base().is_opened())
    }
    /// `IsReadOnly`: of the document in session, else of its meta-data
    /// (`Standard_NullObject` when neither exists, a crash natively).
    pub fn is_read_only(&self) -> Result<bool, Failure> {
        let state = self.state.snapshot();
        if let Some(document) = state.to_document.get() {
            return Ok(document.document_base().is_read_only());
        }
        Ok(live(&state.meta_data, "CDM_Reference::IsReadOnly")?.is_read_only())
    }
    /// Private `Document`: the referenced document without retrieval (may be null).
    pub(crate) fn document(&self) -> Handle<dyn CdmDocument> {
        self.state.read().to_document.clone()
    }
    /// Private `MetaData`; `reference.meta_data()`.
    pub(crate) fn meta_data(&self) -> Handle<CdmMetaData> {
        self.state.read().meta_data.clone()
    }
    /// Private `Application`; `reference.application()`.
    pub(crate) fn application(&self) -> Handle<dyn CdmApplication> {
        self.state.read().application.clone()
    }
    /// Private `UseStorageConfiguration`.
    pub(crate) fn use_storage_configuration(&self) -> bool {
        self.state.read().use_storage_configuration
    }
    /// Private `IsInSession`: the referenced document is in session.
    pub(crate) fn is_in_session(&self) -> bool {
        self.state.read().to_document.is_some()
    }
    /// Private `IsStored`: the reference has meta-data.
    pub(crate) fn is_stored(&self) -> bool {
        self.state.read().meta_data.is_some()
    }
    /// `DumpJson`: the native fields in native order; `reference.dump_json(&mut stream, 1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.snapshot();
        let from_document = state.from_document.upgrade();
        class_begin(stream, "CDM_Reference")?;
        dump_document(stream, depth, "myToDocument.get()", &state.to_document)?;
        dump_document(stream, depth, "myFromDocument", &from_document)?;
        field_integer(stream, "myReferenceIdentifier", state.reference_identifier)?;
        let application = state.application.get();
        let dump_application = |s: &mut DumpStream, d: i32| -> io::Result<()> {
            application.map_or(Ok(()), |application| application.dump_json(s, d))
        };
        let application_child: Option<crate::json_dump::DumpChild<'_>> =
            application.map(|_| &dump_application as crate::json_dump::DumpChild<'_>);
        field_dumped(stream, depth, "myApplication.get()", application_child)?;
        let meta = state.meta_data.get();
        let dump_meta = |s: &mut DumpStream, d: i32| meta.map_or(Ok(()), |m| m.dump_json(s, d));
        let meta_child: Option<crate::json_dump::DumpChild<'_>> =
            meta.map(|_| &dump_meta as crate::json_dump::DumpChild<'_>);
        field_dumped(stream, depth, "myMetaData.get()", meta_child)?;
        field_integer(stream, "myDocumentVersion", state.document_version)?;
        field_integer(
            stream,
            "myUseStorageConfiguration",
            i32::from(state.use_storage_configuration),
        )
    }
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED` of a document pointer.
pub(crate) fn dump_document(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    document: &Handle<dyn CdmDocument>,
) -> io::Result<()> {
    let Some(document) = document.get() else {
        return Ok(());
    };
    let child = |s: &mut DumpStream, d: i32| document.dump_json(s, d);
    field_dumped(stream, depth, field, Some(&child))
}
