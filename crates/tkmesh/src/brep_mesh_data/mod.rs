//! BRepMeshData default discrete model implementations.
//! Shared mutation follows `Standard_Transient`; allocation uses the supplied arena.
mod containers;
mod curves;
mod edge;
mod samples;
use allocator_api2::vec::Vec as ArenaVec;
pub use containers::{DiscreteFace, DiscreteModel, DiscreteWire};
pub use curves::{DiscreteCurve, DiscretePCurve};
pub use edge::DiscreteEdge;
use rustycad_tkernel::{
    ncollection::{BaseAllocator, IncrementalAllocatorObject},
    standard::{Failure, Handle},
};
/// Shared incremental allocation resource; `MeshAllocator::default()`.
pub type MeshAllocator = Handle<IncrementalAllocatorObject>;
pub(crate) fn arena_vector<T>(allocator: &MeshAllocator) -> ArenaVec<T, BaseAllocator> {
    let resource = allocator
        .get()
        .map(|a| a.resource().clone())
        .unwrap_or_else(BaseAllocator::common);
    ArenaVec::new_in(resource)
}
pub(crate) fn mesh_index(index: i32, length: usize, member: &str) -> Result<usize, Failure> {
    if index < 0 || index as usize >= length {
        return Err(Failure::out_of_range(&format!(
            "{member}: index {index}; expected 0..{length}"
        )));
    }
    Ok(index as usize)
}
