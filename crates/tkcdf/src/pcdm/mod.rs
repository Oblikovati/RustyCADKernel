//! The PCDM package: the persistent-document interfaces of the framework (readers, writers,
//! storage and retrieval drivers, reader filters). Concrete native-format drivers are not
//! ported (ocaf-design.md section 1).
mod reader_filter;
pub use reader_filter::{PcdmReaderFilter, PcdmReaderFilterAppendMode};
