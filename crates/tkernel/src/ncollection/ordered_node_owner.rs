//! Both ordered node classes share the existing allocator ownership and unwind guard.
use super::{AllocatedBox, BaseAllocator, node_owner::NativeNodeOwner};
use std::ops::{Deref, DerefMut};
/// Owns one native ordered node; obtain it through `OrderedMapNode::allocate_with`.
pub struct OrderedNodeOwner<N>(NativeNodeOwner<N>);
impl<N> OrderedNodeOwner<N> {
    pub(super) fn from_allocated(owner: AllocatedBox<N>) -> Self {
        Self(NativeNodeOwner::from_allocated(owner))
    }
    pub(super) fn into_parts(self) -> (*mut N, BaseAllocator) {
        self.0.into_parts()
    }
}
impl<N> Deref for OrderedNodeOwner<N> {
    type Target = N;
    fn deref(&self) -> &N {
        &self.0
    }
}
impl<N> DerefMut for OrderedNodeOwner<N> {
    fn deref_mut(&mut self) -> &mut N {
        &mut self.0
    }
}
