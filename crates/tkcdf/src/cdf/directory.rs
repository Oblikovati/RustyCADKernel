// Rust port of ApplicationFramework/TKCDF/CDF/CDF_Directory and CDF_DirectoryIterator
// (.hxx and .cxx), CDF_TypeOfActivation.hxx and CDF_StoreSetNameStatus.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CDF_Directory`: the documents an application has opened, in opening order.
use crate::cdm::CdmDocument;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
};

/// `CDF_TypeOfActivation`: how a retrieved document is activated, in native order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CdfTypeOfActivation {
    /// `CDF_TOA_New`.
    New,
    /// `CDF_TOA_Modified`.
    Modified,
    /// `CDF_TOA_Unchanged`.
    Unchanged,
}

/// `CDF_StoreSetNameStatus`: the outcome of naming a document to store, in native order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CdfStoreSetNameStatus {
    /// `CDF_SSNS_OK`.
    Ok,
    /// `CDF_SSNS_ReplacingAnExistentDocument`.
    ReplacingAnExistentDocument,
    /// `CDF_SSNS_OpenDocument`.
    OpenDocument,
}

/// `CDF_Directory`; `CdfDirectory::new()` is empty.
#[derive(Default)]
pub struct CdfDirectory {
    transient: StandardTransient,
    documents: SharedState<Vec<Handle<dyn CdmDocument>>>,
}
impl_transient!(CdfDirectory, "CDF_Directory", StandardTransient, transient);

impl CdfDirectory {
    /// `CDF_Directory()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Add`: appends a document that is not there yet.
    pub fn add(&self, document: &Handle<dyn CdmDocument>) {
        if !self.contains(document) {
            self.documents.write().push(document.clone());
        }
    }
    /// `Remove`: removes the first occurrence of the document.
    pub fn remove(&self, document: &Handle<dyn CdmDocument>) {
        let mut documents = self.documents.write();
        if let Some(index) = documents.iter().position(|kept| kept == document) {
            documents.remove(index);
        }
    }
    /// `Contains`.
    pub fn contains(&self, document: &Handle<dyn CdmDocument>) -> bool {
        self.documents.read().iter().any(|kept| kept == document)
    }
    /// `Last`: the last document added; `Standard_NoSuchObject` when empty.
    pub fn last(&self) -> Result<Handle<dyn CdmDocument>, Failure> {
        self.documents.read().last().cloned().ok_or_else(|| {
            exceptions::no_such_object(
                "CDF_Directory::Last: the directory does not contain any document",
            )
        })
    }
    /// `Length`; `directory.length()`.
    pub fn length(&self) -> i32 {
        self.documents.read().len() as i32
    }
    /// `IsEmpty`; `directory.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.documents.read().is_empty()
    }
    /// Private `List` (friend `CDF_DirectoryIterator`): a copy of the documents.
    pub(crate) fn list(&self) -> Vec<Handle<dyn CdmDocument>> {
        self.documents.read().clone()
    }
}

/// `CDF_DirectoryIterator`: the documents of a directory, over a copy taken at creation.
#[derive(Default)]
pub struct CdfDirectoryIterator {
    documents: Vec<Handle<dyn CdmDocument>>,
    index: usize,
}
impl CdfDirectoryIterator {
    /// `CDF_DirectoryIterator()`: nothing to iterate.
    pub fn new() -> Self {
        Self::default()
    }
    /// `CDF_DirectoryIterator(aDirectory)`; `Standard_NullObject` for a null directory.
    pub fn with_directory(directory: &Handle<CdfDirectory>) -> Result<Self, Failure> {
        let directory = crate::live(directory, "CDF_DirectoryIterator")?;
        Ok(Self {
            documents: directory.list(),
            index: 0,
        })
    }
    /// `MoreDocument`.
    pub fn more_document(&self) -> bool {
        self.index < self.documents.len()
    }
    /// `NextDocument`.
    pub fn next_document(&mut self) {
        self.index += 1;
    }
    /// `Document`: the current document, null past the end.
    pub fn document(&self) -> Handle<dyn CdmDocument> {
        self.documents.get(self.index).cloned().unwrap_or_default()
    }
}
