// Rust port of ApplicationFramework/TKCDF/CDM/CDM_Application.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CDM_Application`: the abstract application that retrieves documents and reports
//! their updates.
use super::{CdmDocument, CdmDocumentLifecycle, CdmMetaData};
use crate::json_dump::class_begin;
use crate::{SharedResourceManager, live};
use rustycad_tkernel::{
    impl_transient_interface,
    message::{Gravity, Messenger, ProgressRange},
    ncollection::DataMap,
    standard::{
        Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle, dump::DumpStream,
        exceptions,
    },
    tcollection::{AsciiString, ExtendedString},
};
use std::io;
use std::sync::{Arc, OnceLock, RwLockWriteGuard};

use crate::pcdm::PcdmReaderFilter;

/// `NCollection_DataMap<TCollection_ExtendedString, occ::handle<CDM_MetaData>>`: the
/// meta-data of an application by conventional path.
pub type MetaDataLookUpTable = DataMap<ExtendedString, Handle<CdmMetaData>>;

/// The application's table shared with its meta-data driver (`CDF_FWOSDriver` keeps a
/// pointer to it natively).
pub type SharedMetaDataLookUpTable = Arc<SharedState<MetaDataLookUpTable>>;

/// The private fields of `CDM_Application`; `CdmApplicationBase::default()` is the
/// protected constructor (it creates the default messenger).
pub struct CdmApplicationBase {
    this: OnceLock<WeakHandle<dyn CdmApplication>>,
    messenger: Arc<Messenger>,
    meta_data_look_up_table: SharedMetaDataLookUpTable,
}
impl Default for CdmApplicationBase {
    fn default() -> Self {
        Self {
            this: OnceLock::new(),
            messenger: Arc::new(Messenger::new()),
            meta_data_look_up_table: Arc::new(SharedState::new(MetaDataLookUpTable::default())),
        }
    }
}

impl CdmApplicationBase {
    /// The shared table, for the drivers that keep a pointer to it;
    /// `base.shared_meta_data_look_up_table()`.
    pub fn shared_meta_data_look_up_table(&self) -> SharedMetaDataLookUpTable {
        self.meta_data_look_up_table.clone()
    }
    /// Records the owning handle of this application, once (ocaf-design.md section 2);
    /// `application.application_base().bind_self(&application)`.
    pub fn bind_self(&self, application: &Handle<dyn CdmApplication>) {
        let _ = self.this.set(application.downgrade());
    }
    /// The handle `this` converts to natively; `Standard_NullObject` before `bind_self`.
    pub fn this(&self) -> Result<Handle<dyn CdmApplication>, Failure> {
        let this = self.this.get().map(WeakHandle::upgrade).unwrap_or_default();
        if this.is_null() {
            return Err(exceptions::null_object(
                "CDM_Application: the application has no owning handle; expected bind_self after creation",
            ));
        }
        Ok(this)
    }
}

/// `CDM_Application`'s members; a concrete application embeds a [`CdmApplicationBase`].
pub trait CdmApplication: Transient {
    /// The embedded base with the native private fields.
    fn application_base(&self) -> &CdmApplicationBase;
    /// `Resources` (pure virtual): the manager searched for `<format>.*` resources
    /// (`Result`, because an implementation may load a resource file).
    fn resources(&self) -> Result<SharedResourceManager, Failure>;
    /// `MessageDriver`: the default messenger (`occ::handle<Message_Messenger>` is an
    /// `Arc<Messenger>` in the TKernel port).
    fn message_driver(&self) -> Arc<Messenger> {
        self.application_base().messenger.clone()
    }
    /// `BeginOfUpdate`: writes `"Updating: Document"` by default.
    fn begin_of_update(&self, _document: &Handle<dyn CdmDocument>) {
        let mut updating = ExtendedString::new("Updating: ");
        updating.assign_cat(ExtendedString::new("Document"));
        self.write(updating.as_utf16());
    }
    /// `EndOfUpdate`: writes `"Updated: Document"` or `"Error during updating: Document"`.
    fn end_of_update(
        &self,
        _document: &Handle<dyn CdmDocument>,
        status: bool,
        _error_string: &ExtendedString,
    ) {
        let prefix = if status {
            "Updated: "
        } else {
            "Error during updating: "
        };
        let mut message = ExtendedString::new(prefix);
        message.assign_cat(ExtendedString::new("Document"));
        self.write(message.as_utf16());
    }
    /// `Write`: sends the text (up to its first NUL) to the message driver as a warning,
    /// the native default gravity of `Message_Messenger::Send`.
    fn write(&self, text: &[u16]) {
        let messenger = self.message_driver();
        messenger.send_extended(&ExtendedString::from_utf16(text), Gravity::Warning);
    }
    /// `Name`: empty by default.
    fn name(&self) -> ExtendedString {
        ExtendedString::default()
    }
    /// `Version`: empty by default.
    fn version(&self) -> AsciiString {
        AsciiString::default()
    }
    /// `MetaDataLookUpTable`: the application's table, lent through its write guard
    /// (ocaf-design.md section 6); drop the guard before calling back into the application.
    fn meta_data_look_up_table(&self) -> RwLockWriteGuard<'_, MetaDataLookUpTable> {
        self.application_base().meta_data_look_up_table.write()
    }
    /// `DumpJson`: the class name only.
    fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        class_begin(stream, "CDM_Application")
    }
    /// `Retrieve` (pure virtual, private; called by `CDM_Reference`): the document a
    /// meta-data describes.
    fn retrieve(
        &self,
        meta_data: &Handle<CdmMetaData>,
        use_storage_configuration: bool,
        filter: &Handle<PcdmReaderFilter>,
        range: &ProgressRange<'_>,
    ) -> Result<Handle<dyn CdmDocument>, Failure>;
    /// `DocumentVersion` (pure virtual, private): the stored modification counter of a
    /// meta-data, -1 when it has none.
    fn document_version(&self, meta_data: &Handle<CdmMetaData>) -> Result<i32, Failure>;
    /// Protected `SetDocumentVersion`: the document's counter becomes the stored one.
    fn set_document_version(
        &self,
        document: &Handle<dyn CdmDocument>,
        meta_data: &Handle<CdmMetaData>,
    ) -> Result<(), Failure> {
        let application = self.application_base().this()?;
        let version = CdmMetaData::document_version(meta_data, &application)?;
        live(document, "CDM_Application::SetDocumentVersion")?.set_modifications(version);
        Ok(())
    }
    /// Protected `SetReferenceCounter`: the document's last reference identifier.
    fn set_reference_counter(
        &self,
        document: &Handle<dyn CdmDocument>,
        reference_counter: i32,
    ) -> Result<(), Failure> {
        let document = live(document, "CDM_Application::SetReferenceCounter")?;
        super::CdmDocumentReferences::set_reference_counter(document, reference_counter);
        Ok(())
    }
}
impl_transient_interface!(dyn CdmApplication, "CDM_Application", StandardTransient);
