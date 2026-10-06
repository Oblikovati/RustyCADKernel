//! Existing allocator ownership is shared with other native node families.
#![allow(unsafe_code)]
use super::{PackedInteger, PackedMapNode};
use crate::ncollection::{AllocatedBox, BaseAllocator};

impl<T: PackedInteger> PackedMapNode<T> {
    /// Allocates native-sized storage before node construction; `PackedMapNode::allocate_with(&arena, || node)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        construct: impl FnOnce() -> Self,
    ) -> AllocatedBox<Self> {
        allocator.allocate_value_with(construct)
    }
    /// Transfers one node and its allocator without following Next; `PackedMapNode::into_raw(owner)`.
    pub fn into_raw(owner: AllocatedBox<Self>) -> (*mut Self, BaseAllocator) {
        owner.into_raw_with_allocator()
    }
    /// Restores ownership of one previously transferred node; `unsafe {PackedMapNode::from_raw(pointer, arena)}`.
    /// # Safety
    /// `pointer` must be an unreclaimed into_raw result paired with `allocator`, with no live borrows.
    pub unsafe fn from_raw(pointer: *mut Self, allocator: BaseAllocator) -> AllocatedBox<Self> {
        // SAFETY: the caller transfers the unique initialized allocation and matching resource.
        unsafe { AllocatedBox::from_raw_in(pointer, allocator) }
    }
}
