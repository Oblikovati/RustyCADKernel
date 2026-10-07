//! The PCDM package: the persistent-document interfaces of the framework (readers, writers,
//! storage and retrieval drivers, reader filters). Concrete native-format drivers and the
//! native container probes are not ported (ocaf-design.md section 1).
mod native_header;
mod read_writer;
mod reader;
mod reader_filter;
mod reference;
mod reference_iterator;
mod statuses;
mod writer;
pub use read_writer::{PcdmReadWriter, PcdmReadWriterBase};
pub use reader::{PcdmReader, PcdmReaderBase, PcdmRetrievalDriver, PcdmRetrievalDriverBase};
pub use reader_filter::{PcdmReaderFilter, PcdmReaderFilterAppendMode};
pub use reference::PcdmReference;
pub use reference_iterator::PcdmReferenceIterator;
pub use statuses::{PcdmReaderStatus, PcdmStoreStatus, PcdmTypeOfFileDriver, pcdm_driver_error};
pub use writer::{PcdmStorageDriver, PcdmStorageDriverBase, PcdmWriter};
