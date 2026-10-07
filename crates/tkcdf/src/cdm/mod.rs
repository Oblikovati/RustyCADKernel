//! The CDM package: applicative documents, references between documents, meta-data of
//! stored documents and the abstract application (ocaf-design.md section 6).
mod application;
mod can_close_status;
mod document;
mod document_lifecycle;
mod document_references;
mod document_storage;
mod meta_data;
mod reference;
mod reference_iterator;
pub use application::{
    CdmApplication, CdmApplicationBase, MetaDataLookUpTable, SharedMetaDataLookUpTable,
};
pub use can_close_status::CdmCanCloseStatus;
pub use document::{CdmDocument, CdmDocumentBase, ModifContext};
pub use document_lifecycle::CdmDocumentLifecycle;
pub use document_references::CdmDocumentReferences;
pub use document_storage::CdmDocumentStorage;
pub use meta_data::CdmMetaData;
pub use reference::CdmReference;
pub use reference_iterator::CdmReferenceIterator;
