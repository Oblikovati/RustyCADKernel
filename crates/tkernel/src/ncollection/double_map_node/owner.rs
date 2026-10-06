//! Safe node ownership uses the shared unwind deallocation guard, unlike allocator-api2 Box Drop.
use super::{AllocatedBox, BaseAllocator, DoubleMapNode, MapLinkNode};
use crate::ncollection::node_owner::NativeNodeOwner;
use std::ops::{Deref, DerefMut};
/// Allocator-owned DoubleMapNode; `let node = DoubleMapNode::allocate_with(&arena, next, next2, || 1, || 7);`.
pub struct DoubleMapNodeOwner<K, V>(NativeNodeOwner<DoubleMapNode<K, V>>);
impl<K, V> DoubleMapNodeOwner<K, V> {
    pub(super) fn from_allocated(owner: AllocatedBox<DoubleMapNode<K, V>>) -> Self {
        Self(NativeNodeOwner::from_allocated(owner))
    }
    pub(super) fn into_parts(self) -> (*mut MapLinkNode, BaseAllocator) {
        let (pointer, allocator) = self.0.into_parts();
        (pointer.cast(), allocator)
    }
}
impl<K, V> Deref for DoubleMapNodeOwner<K, V> {
    type Target = DoubleMapNode<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<K, V> DerefMut for DoubleMapNodeOwner<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
