// Rust port of ApplicationFramework/TKCDF/CDF/CDF_StoreList.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CDF_StoreList`: a document and the modified documents it references, stored together.
use super::{CdfApplication, CdfApplicationServices, CdfMetaDataDriver};
use crate::cdm::{
    CdmDocument, CdmDocumentLifecycle, CdmDocumentStorage, CdmMetaData, CdmReferenceIterator,
};
use crate::live;
use crate::pcdm::PcdmStoreStatus;
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    ncollection::Map,
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::ExtendedString,
};

#[derive(Default)]
struct StoreListState {
    items: Map<Handle<dyn CdmDocument>>,
    /// Native `myStack`, a list with the last added document first.
    stack: Vec<Handle<dyn CdmDocument>>,
    /// The documents of `items` in map order and the position of the iterator.
    iteration: Vec<Handle<dyn CdmDocument>>,
    position: usize,
}

/// `CDF_StoreList`; `CdfStoreList::new(&document)?`.
pub struct CdfStoreList {
    transient: StandardTransient,
    main_document: Handle<dyn CdmDocument>,
    state: SharedState<StoreListState>,
}
impl_transient!(CdfStoreList, "CDF_StoreList", StandardTransient, transient);

impl CdfStoreList {
    /// `CDF_StoreList(aDocument)`: the document and, recursively, its modified references.
    pub fn new(document: &Handle<dyn CdmDocument>) -> Result<Self, Failure> {
        let list = Self {
            transient: StandardTransient::default(),
            main_document: document.clone(),
            state: SharedState::default(),
        };
        list.add(document)?;
        Ok(list)
    }
    /// Private `Add`: the document (once, except the main one) on the stack, then its
    /// modified referenced documents.
    fn add(&self, document: &Handle<dyn CdmDocument>) -> Result<(), Failure> {
        {
            let mut state = self.state.write();
            if !state.items.contains(document) && *document != self.main_document {
                state.items.add(document.clone());
            }
            state.stack.insert(0, document.clone());
        }
        let mut references = CdmReferenceIterator::new(document)?;
        while references.more() {
            let referenced = references.document()?;
            if live(&referenced, "CDF_StoreList::Add")?.is_modified() {
                self.add(&references.document()?)?;
            }
            references.next();
        }
        Ok(())
    }
    /// `IsConsistent`: every document has a requested folder.
    pub fn is_consistent(&self) -> Result<bool, Failure> {
        let items: Vec<_> = self.state.read().items.iter().cloned().collect();
        for item in &items {
            if !live(item, "CDF_StoreList::IsConsistent")?.has_requested_folder() {
                return Ok(false);
            }
        }
        Ok(live(&self.main_document, "CDF_StoreList::IsConsistent")?.has_requested_folder())
    }
    /// `Init`: restarts the iteration over the referenced documents.
    pub fn init(&self) {
        let mut state = self.state.write();
        state.iteration = state.items.iter().cloned().collect();
        state.position = 0;
    }
    /// `More`.
    pub fn more(&self) -> bool {
        let state = self.state.read();
        state.position < state.iteration.len()
    }
    /// `Next`.
    pub fn next(&self) {
        self.state.write().position += 1;
    }
    /// `Value`: the current document, null past the end.
    pub fn value(&self) -> Handle<dyn CdmDocument> {
        let state = self.state.read();
        state
            .iteration
            .get(state.position)
            .cloned()
            .unwrap_or_default()
    }
    /// `Store(aMetaData, aStatusAssociatedText, theRange)`: writes the main document and the
    /// modified ones of the stack through their applications' storage drivers.
    pub fn store(
        &self,
        meta_data: &mut Handle<CdmMetaData>,
        text: &mut ExtendedString,
        range: &ProgressRange<'_>,
    ) -> Result<PcdmStoreStatus, Failure> {
        let main = live(&self.main_document, "CDF_StoreList::Store")?;
        let application: Handle<dyn CdfApplication> = main.application()?.cast();
        let driver = live(&application, "CDF_StoreList::Store")?.meta_data_driver()?;
        let mut status = PcdmStoreStatus::Ok;
        loop {
            let Some(document) = self.state.read().stack.first().cloned() else {
                break;
            };
            let target = live(&document, "CDF_StoreList::Store")?;
            if document == self.main_document || target.is_modified() {
                status = match store_one(&document, &driver, meta_data, text, range) {
                    Ok(status) => status,
                    Err(failure) => {
                        text.assign_cat(ExtendedString::new("driver failed; reason: "));
                        text.assign_cat(ExtendedString::from_c_string(failure.what(), false));
                        PcdmStoreStatus::DriverFailure
                    }
                };
            }
            self.state.write().stack.remove(0);
        }
        Ok(status)
    }
}

/// The body of the native store loop for one document.
fn store_one(
    document: &Handle<dyn CdmDocument>,
    driver: &Handle<dyn CdfMetaDataDriver>,
    meta_data: &mut Handle<CdmMetaData>,
    text: &mut ExtendedString,
    range: &ProgressRange<'_>,
) -> Result<PcdmStoreStatus, Failure> {
    let target = live(document, "CDF_StoreList::Store")?;
    let application: Handle<dyn CdfApplication> = target.application()?.cast();
    let Some(application) = application.get() else {
        *text =
            ExtendedString::new("driver failed; reason: document has no application, cannot save!");
        return Ok(PcdmStoreStatus::Failure);
    };
    let format = target.storage_format();
    let writer = application.writer_from_format(&format)?;
    let Some(writer) = writer.get() else {
        *text = ExtendedString::new(
            "driver not found; reason: no storage driver does exist for this format: ",
        )
        .cat(&format);
        return Ok(PcdmStoreStatus::UnrecognizedFormat);
    };
    // Reset the store status, which matters when the driver is shared between threads.
    writer.set_store_status(PcdmStoreStatus::Ok);
    let driver = live(driver, "CDF_StoreList::Store")?;
    let folder = target.requested_folder()?;
    if !driver.find_folder(&folder) {
        *text = ExtendedString::new("driver not found; reason: could not find the active dbunit ")
            .cat(&folder);
        return Ok(PcdmStoreStatus::UnrecognizedFormat);
    }
    let name = driver.build_file_name(document)?;
    writer.write_file(document, &name, range)?;
    let status = writer.get_store_status();
    *meta_data = driver.create_meta_data(document, &name)?;
    target.set_meta_data(meta_data)?;
    create_references(document, driver, meta_data)?;
    Ok(status)
}

/// The meta-data references of a stored document to its referenced documents.
fn create_references(
    document: &Handle<dyn CdmDocument>,
    driver: &dyn CdfMetaDataDriver,
    meta_data: &Handle<CdmMetaData>,
) -> Result<(), Failure> {
    let mut references = CdmReferenceIterator::new(document)?;
    while references.more() {
        let referenced = references.document()?;
        let referenced_meta = live(&referenced, "CDF_StoreList::Store")?.meta_data()?;
        driver.create_reference(
            meta_data,
            &referenced_meta,
            references.reference_identifier()?,
            references.document_version()?,
        );
        references.next();
    }
    Ok(())
}
