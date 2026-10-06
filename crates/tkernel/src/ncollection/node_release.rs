//! Typed linked-node destruction must release allocation even when the item destructor unwinds.
use super::BaseAllocator;
use allocator_api2::alloc::Allocator;
use std::{alloc::Layout, ptr::NonNull};
struct NodeAllocationRelease<'a> {
    allocator: &'a BaseAllocator,
    pointer: NonNull<u8>,
    layout: Layout,
}
impl Drop for NodeAllocationRelease<'_> {
    fn drop(&mut self) {
        // SAFETY: the guard retains the original allocation and exact typed layout.
        unsafe { self.allocator.deallocate(self.pointer, self.layout) };
    }
}
pub(super) unsafe fn release_typed_node<T>(pointer: *mut T, allocator: &BaseAllocator) {
    let allocation = NodeAllocationRelease {
        allocator,
        pointer: NonNull::new(pointer.cast()).expect("owned node is nonnull"),
        layout: Layout::new::<T>(),
    };
    // allocator-api2 0.4.0 Box::drop lacks unwind deallocation. Retain the allocation
    // guard while invoking user destruction, with the original allocator and layout.
    unsafe { std::ptr::drop_in_place(pointer) };
    drop(allocation);
}
