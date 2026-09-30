//! OCCT's `Storage` package: the legacy persistence framework of drivers, schemas and
//! data sets. Drivers read and write through an [`IoStream`] over an injected device.
mod base_driver;
mod bucket;
mod callback;
mod data;
mod header_data;
mod internal_data;
mod kinds;
mod numbers;
mod root;
mod root_data;
mod schema;
mod status;
mod stream;
mod type_data;

pub use base_driver::{
    BaseDriver, DataSections, DriverBase, DriverPrimitives, DriverValue, HeaderSections,
    TableSections,
};
pub use bucket::{Bucket, BucketIterator, BucketOfPersistent};
pub(crate) use callback::CallBackTable;
pub use callback::{CallBack, DefaultCallBack, TypedCallBack};
pub use data::Data;
pub use header_data::{HeaderData, HeaderFields};
pub use internal_data::{InternalData, ReadArray};
pub(crate) use kinds::STORAGE_HIERARCHY;
pub use kinds::{
    OpenMode, Position, SolveMode, StorageError, StreamException, stream_ext_char_parity_error,
    stream_format_error, stream_mode_error, stream_read_error, stream_type_mismatch_error,
    stream_unknown_type_error, stream_write_error,
};
pub(crate) use numbers::PersistentNumbers;
pub use root::Root;
pub use root_data::RootData;
pub use schema::{
    CreationDateSource, MigrationFile, MigrationSource, NativeCreationDate, NativeMigrationSource,
    Schema, TypeMigration,
};
pub use status::SectionStatus;
pub use stream::{IoStream, StreamDevice, format_general, state};
pub use type_data::TypeData;

use crate::tcollection::AsciiString;

/// `Storage::Version`: the version of Storage's read/write routines, `"1.3"`.
///
/// ```
/// assert_eq!(rustycad_tkernel::storage::version().as_bytes(), b"1.3");
/// ```
pub fn version() -> AsciiString {
    AsciiString::new("1.3")
}
