//! OCCT's Standard foundation package.
mod guid;
pub use guid::{GUID_SIZE, GUID_SIZE_ALLOC, Guid, Uuid};
mod array_stream_buffer;
mod array_stream_hooks;
mod array_stream_state;
pub use array_stream_buffer::ArrayStreamBuffer;
pub use array_stream_hooks::{ArrayStreamBufferHooks, NativeArrayStreamBufferHooks};
pub use array_stream_state::{ArrayStreamBufferState, ArrayStreamSeekDirection};
mod failure;
pub use failure::Failure;
pub mod exceptions;
mod strtod;
pub use strtod::{strtod, strtod_bytes};

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
mod shared_state;
mod transient;
pub use handle::{Handle, HandleView, WeakHandle};
pub use runtime_type::{
    InterfaceCaster, InterfaceEntry, RuntimeType, RuntimeTyped, project_concrete, project_interface,
};
pub use shared_state::SharedState;
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
mod fread_stream;
mod istream_stream;
mod read_buffer;
mod read_line_buffer;
mod stream_fill;
pub use fread_stream::FreadStream;
pub use istream_stream::IstreamStream;
pub use read_buffer::ReadBuffer;
pub use read_line_buffer::ReadLineBuffer;
