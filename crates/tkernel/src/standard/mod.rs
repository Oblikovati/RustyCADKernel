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

pub mod hashing;
pub use hashing::hash_bytes;
pub mod atomic;
pub mod c_string;
pub mod ext_character;
pub mod short_real;
pub mod time;
pub mod version_info;

mod handle;
mod runtime_type;
mod transient;
pub use handle::{Handle, HandleView};
pub use runtime_type::{RuntimeType, RuntimeTyped};
pub use transient::{StandardTransient, Transient};
pub mod assert;
pub mod error_handler;
pub use error_handler::{ErrorHandler, ErrorHandlerCallback};
pub mod occ;
mod persistent;
pub use persistent::StandardPersistent;
mod stack_trace;
pub use stack_trace::stack_trace;
pub mod dump;
pub mod integer;
pub mod real;
