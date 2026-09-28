//! OCCT's Standard foundation package.
mod guid;
pub use guid::{GUID_SIZE, GUID_SIZE_ALLOC, Guid, Uuid};
mod array_stream_buffer;
pub use array_stream_buffer::ArrayStreamBuffer;
mod failure;
pub use failure::Failure;
pub mod exceptions;
mod strtod;
pub use strtod::strtod;

pub mod character;

pub(crate) mod hashing;
pub use hashing::hash_bytes;

mod handle;
mod runtime_type;
mod transient;
pub use handle::{Handle, HandleView};
pub use runtime_type::{RuntimeType, RuntimeTyped};
pub use transient::{StandardTransient, Transient};
pub mod dump;
pub mod integer;
pub mod memory;
mod mmgr_opt;
mod mmgr_root;
pub mod real;
pub use mmgr_opt::{MMgrOpt, OptCallBack};
pub use mmgr_root::{MMgrRoot, StorageAddress};
pub mod condition;
pub mod memory_utils;
pub mod mutex;
pub use condition::StandardCondition;
pub use mutex::StandardMutex;
pub mod c_locale_sentry;
mod read_buffer;
mod read_line_buffer;
mod stream_fill;
pub use read_buffer::ReadBuffer;
pub use read_line_buffer::ReadLineBuffer;
