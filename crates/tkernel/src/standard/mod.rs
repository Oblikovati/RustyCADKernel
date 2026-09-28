//! OCCT's Standard foundation package.
mod guid;
pub use guid::{GUID_SIZE, GUID_SIZE_ALLOC, Guid, Uuid};
mod array_stream_buffer;
pub use array_stream_buffer::ArrayStreamBuffer;
mod failure;
pub use failure::Failure;
mod strtod;
pub use strtod::strtod;

pub mod character;

pub mod hashing;
pub use hashing::hash_bytes;

mod handle;
mod runtime_type;
mod shared_state;
mod transient;
pub use handle::{Handle, HandleView};
pub use runtime_type::{
    InterfaceCaster, InterfaceEntry, RuntimeType, RuntimeTyped, project_interface,
};
pub use shared_state::SharedState;
pub use transient::{StandardTransient, Transient};
pub mod dump;
pub mod integer;
pub mod real;
