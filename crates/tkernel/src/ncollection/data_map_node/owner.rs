//! Safe node ownership uses the shared unwind deallocation guard, unlike allocator-api2 Box Drop.
use super::{AllocatedBox, BaseAllocator, DataMapNode, MapLinkNode};
use crate::ncollection::node_owner::NativeNodeOwner;
use std::ops::{Deref, DerefMut};
/// Allocator-owned DataMapNode; `let node = DataMapNode::allocate_with(&arena, next, || 1, || 7);`.
pub struct DataMapNodeOwner<K, V>(NativeNodeOwner<DataMapNode<K, V>>);
impl<K, V> DataMapNodeOwner<K, V> {
    pub(super) fn from_allocated(owner: AllocatedBox<DataMapNode<K, V>>) -> Self {
        Self(NativeNodeOwner::from_allocated(owner))
    }
    pub(super) fn into_parts(self) -> (*mut MapLinkNode, BaseAllocator) {
        let (pointer, allocator) = self.0.into_parts();
        (pointer.cast(), allocator)
    }
}
impl<K, V> Deref for DataMapNodeOwner<K, V> {
    type Target = DataMapNode<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<K, V> DerefMut for DataMapNodeOwner<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
