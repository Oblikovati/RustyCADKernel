//! OCCT's `FSD` package: file drivers of the `Storage` framework (text, compatible text
//! and binary formats) and the Base64 codec. Every driver opens its file through an
//! injected [`DriverFileSystem`].
mod base64;
mod binary_file;
mod c_file;
mod file_header;
mod file_system;
mod text_file;

pub use base64::Base64;
pub use binary_file::BinaryFile;
pub(crate) use c_file::CFile;
pub use file_header::FileHeader;
pub use file_system::{DriverFileSystem, FileOpenMode, MemoryFileSystem, NativeDriverFileSystem};
pub use text_file::{CmpDialect, CmpFile, File, FsdDialect, TextDialect, TextFile};
