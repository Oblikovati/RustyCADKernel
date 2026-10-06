//! A typed node owner preserves unwind deallocation across the native map node families.
#![allow(unsafe_code)]
use super::{AllocatedBox, BaseAllocator, node_release::release_typed_node};
use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
pub(super) struct NativeNodeOwner<N> {
    pointer: Option<NonNull<N>>,
    allocator: BaseAllocator,
}
impl<N> NativeNodeOwner<N> {
    pub(super) fn from_allocated(owner: AllocatedBox<N>) -> Self {
        let (pointer, allocator) = owner.into_raw_with_allocator();
        Self {
            pointer: Some(NonNull::new(pointer).unwrap()),
            allocator,
        }
    }
    pub(super) fn into_parts(mut self) -> (*mut N, BaseAllocator) {
        let pointer = self.pointer.take().unwrap().as_ptr();
        (pointer, self.allocator.clone())
    }
}
impl<N> Deref for NativeNodeOwner<N> {
    type Target = N;
    fn deref(&self) -> &N {
        // SAFETY: an intact owner exclusively owns one initialized node.
        unsafe { self.pointer.unwrap().as_ref() }
    }
}
impl<N> DerefMut for NativeNodeOwner<N> {
    fn deref_mut(&mut self) -> &mut N {
        // SAFETY: exclusive owner borrow prohibits competing safe node accesses.
        unsafe { self.pointer.as_mut().unwrap().as_mut() }
    }
}
impl<N> Drop for NativeNodeOwner<N> {
    fn drop(&mut self) {
        if let Some(pointer) = self.pointer.take() {
            // SAFETY: pointer is the unique live allocation paired with this allocator.
            unsafe { release_typed_node(pointer.as_ptr(), &self.allocator) };
        }
    }
}
