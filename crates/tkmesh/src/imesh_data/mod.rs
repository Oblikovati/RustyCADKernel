//! IMeshData discrete model interfaces. Indices start at zero.
//!
//! Native mutable-reference getters are value getters plus explicit setters, so
//! shared handles observe writes without holding locks across calls to other objects.
//! Native non-owning model pointers use `WeakHandle` to avoid ownership cycles.
mod parameters;
pub(crate) mod shape;
mod status;
mod topology;
pub use parameters::{MeshCurve, MeshPCurve, ParametersList, ParametersListArrayAdaptor};
use rustycad_tkernel::standard::{Handle, WeakHandle};
pub use shape::{MeshShape, TessellatedShape};
pub use status::{Status, StatusOwner};
pub use topology::{MeshEdge, MeshFace, MeshModel, MeshWire};
/// Native allocator block size on this platform; `MEMORY_BLOCK_SIZE_HUGE >= 524288`.
pub const MEMORY_BLOCK_SIZE_HUGE: usize = if cfg!(all(windows, target_pointer_width = "64")) {
    1024 * 1024
} else {
    512 * 1024
};
/// Owning discrete edge; `EdgeHandle::default()`.
pub type EdgeHandle = Handle<dyn MeshEdge>;
/// Owning discrete face; `FaceHandle::default()`.
pub type FaceHandle = Handle<dyn MeshFace>;
/// Owning ordered wire; `WireHandle::default()`.
pub type WireHandle = Handle<dyn MeshWire>;
/// Owning 3D discretization; `CurveHandle::default()`.
pub type CurveHandle = Handle<dyn MeshCurve>;
/// Owning 2D discretization; `PCurveHandle::default()`.
pub type PCurveHandle = Handle<dyn MeshPCurve>;
/// Non-owning edge pointer; `EdgePtr::default()`.
pub type EdgePtr = WeakHandle<dyn MeshEdge>;
/// Non-owning face pointer; `FacePtr::default()`.
pub type FacePtr = WeakHandle<dyn MeshFace>;
