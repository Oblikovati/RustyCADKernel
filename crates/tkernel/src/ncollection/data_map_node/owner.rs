//! Safe node ownership uses the shared unwind deallocation guard, unlike allocator-api2 Box Drop.
use super::{AllocatedBox, BaseAllocator, DataMapNode, MapLinkNode};
use crate::ncollection::node_release::release_typed_node;
use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
/// Allocator-owned DataMapNode; `let node = DataMapNode::allocate_with(&arena, next, || 1, || 7);`.
pub struct DataMapNodeOwner<K, V> {
    pointer: Option<NonNull<DataMapNode<K, V>>>,
    allocator: BaseAllocator,
}
impl<K, V> DataMapNodeOwner<K, V> {
    pub(super) fn from_allocated(owner: AllocatedBox<DataMapNode<K, V>>) -> Self {
        let (pointer, allocator) = owner.into_raw_with_allocator();
        Self {
            pointer: Some(NonNull::new(pointer).unwrap()),
            allocator,
        }
    }
    pub(super) fn into_parts(mut self) -> (*mut MapLinkNode, BaseAllocator) {
        let pointer = self.pointer.take().unwrap().as_ptr().cast();
        (pointer, self.allocator.clone())
    }
}
impl<K, V> Deref for DataMapNodeOwner<K, V> {
    type Target = DataMapNode<K, V>;
    fn deref(&self) -> &Self::Target {
        // SAFETY: an intact owner exclusively owns one initialized node.
        unsafe { self.pointer.unwrap().as_ref() }
    }
}
impl<K, V> DerefMut for DataMapNodeOwner<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: exclusive owner borrow prohibits competing safe node accesses.
        unsafe { self.pointer.as_mut().unwrap().as_mut() }
    }
}
impl<K, V> Drop for DataMapNodeOwner<K, V> {
    fn drop(&mut self) {
        if let Some(pointer) = self.pointer.take() {
            // SAFETY: pointer is the unique live allocation paired with this allocator.
            unsafe { release_typed_node(pointer.as_ptr(), &self.allocator) };
        }
    }
}
