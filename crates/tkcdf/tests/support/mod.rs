//! Named fakes of the abstract CDM classes for the tests.
#![allow(dead_code)]
use rustycad_tkcdf::{
    SharedResourceManager,
    cdm::{CdmApplication, CdmApplicationBase, CdmDocument, CdmDocumentBase, CdmMetaData},
    pcdm::PcdmReaderFilter,
};
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    resource::{ResourceHost, ResourceManager},
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::ExtendedString,
};
use std::sync::{Arc, Mutex};

/// A document of format "TestFormat" that records its update calls.
#[derive(Default)]
pub struct FakeDocument {
    transient: StandardTransient,
    base: CdmDocumentBase,
    pub updates: Mutex<Vec<i32>>,
    pub refuse_close: Mutex<bool>,
}
impl_transient!(FakeDocument, "FakeDocument", dyn CdmDocument, transient, interfaces: [dyn CdmDocument]);
impl CdmDocument for FakeDocument {
    fn document_base(&self) -> &CdmDocumentBase {
        &self.base
    }
    fn storage_format(&self) -> ExtendedString {
        ExtendedString::new("TestFormat")
    }
    fn update_reference(&self, _to: &Handle<dyn CdmDocument>, id: i32, _context: *const ()) {
        self.updates.lock().unwrap().push(id);
    }
    fn can_close_reference(&self, _document: &Handle<dyn CdmDocument>, _id: i32) -> bool {
        !*self.refuse_close.lock().unwrap()
    }
}

/// A new bound fake document and its concrete handle.
pub fn new_document() -> (Handle<dyn CdmDocument>, Handle<FakeDocument>) {
    let concrete = Handle::new(FakeDocument::default());
    let document: Handle<dyn CdmDocument> = concrete.cast();
    document.document_base().bind_self(&document);
    (document, concrete)
}

/// An application with an in-memory resource manager that retrieves a fixed document.
pub struct FakeApplication {
    transient: StandardTransient,
    base: CdmApplicationBase,
    pub resources: SharedResourceManager,
    pub retrieved: Mutex<Handle<dyn CdmDocument>>,
    pub stored_version: i32,
}
impl_transient!(FakeApplication, "FakeApplication", dyn CdmApplication, transient, interfaces: [dyn CdmApplication]);
impl CdmApplication for FakeApplication {
    fn application_base(&self) -> &CdmApplicationBase {
        &self.base
    }
    fn resources(&self) -> SharedResourceManager {
        self.resources.clone()
    }
    fn retrieve(
        &self,
        _meta_data: &Handle<CdmMetaData>,
        _use_storage_configuration: bool,
        _filter: &Handle<PcdmReaderFilter>,
        _range: &ProgressRange<'_>,
    ) -> Result<Handle<dyn CdmDocument>, Failure> {
        Ok(self.retrieved.lock().unwrap().clone())
    }
    fn document_version(&self, _meta_data: &Handle<CdmMetaData>) -> Result<i32, Failure> {
        Ok(self.stored_version)
    }
}

/// A new bound fake application.
pub fn new_application(
    stored_version: i32,
) -> (Handle<dyn CdmApplication>, Handle<FakeApplication>) {
    let manager = ResourceManager::empty(ResourceHost::native());
    let concrete = Handle::new(FakeApplication {
        transient: StandardTransient::default(),
        base: CdmApplicationBase::default(),
        resources: Arc::new(SharedState::new(manager)),
        retrieved: Mutex::new(Handle::default()),
        stored_version,
    });
    let application: Handle<dyn CdmApplication> = concrete.cast();
    application.application_base().bind_self(&application);
    (application, concrete)
}

/// `ExtendedString::new` shorthand.
pub fn text(value: &str) -> ExtendedString {
    ExtendedString::new(value)
}
