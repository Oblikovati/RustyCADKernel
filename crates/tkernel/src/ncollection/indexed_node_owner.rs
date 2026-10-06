//! Indexed node families share the existing native allocation/unwind owner.
use super::{AllocatedBox, BaseAllocator, node_owner::NativeNodeOwner};
use std::ops::{Deref, DerefMut};
/// Allocator-owned indexed node; obtained through `IndexedMapNode::allocate_with`.
pub struct IndexedNodeAllocation<N>(NativeNodeOwner<N>);
impl<N> IndexedNodeAllocation<N> {
    pub(super) fn from_allocated(owner: AllocatedBox<N>) -> Self {
        Self(NativeNodeOwner::from_allocated(owner))
    }
    pub(super) fn into_parts(self) -> (*mut N, BaseAllocator) {
        self.0.into_parts()
    }
}
impl<N> Deref for IndexedNodeAllocation<N> {
    type Target = N;
    fn deref(&self) -> &N {
        &self.0
    }
}
impl<N> DerefMut for IndexedNodeAllocation<N> {
    fn deref_mut(&mut self) -> &mut N {
        &mut self.0
    }
}
