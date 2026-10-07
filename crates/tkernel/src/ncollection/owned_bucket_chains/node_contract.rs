//! Prefix casts and payload-derived thread bounds apply only to these audited native node layouts.
use crate::ncollection::{BaseAllocator, BasicMapNode};
pub(in crate::ncollection) mod sealed {
    pub trait Sealed {}
}
/// # Safety
/// Implementors have a real MapLinkNode prefix at offset zero and exact typed allocation.
/// Key/Value account for every owned payload and threading requirement; links are non-owning.
/// Bucket traversal reads only the link prefix. Key and Value may alias for key-only nodes.
pub(in crate::ncollection) unsafe trait CanonicalMapNode:
    BasicMapNode + sealed::Sealed
{
    /// Releases exactly this owned node, including allocator-dependent private payloads.
    /// # Safety
    /// Pointer must denote this allocator's live concrete node without outstanding borrows.
    unsafe fn release_owned(pointer: *mut Self, allocator: &BaseAllocator) {
        unsafe { crate::ncollection::node_release::release_typed_node(pointer, allocator) };
    }
}
