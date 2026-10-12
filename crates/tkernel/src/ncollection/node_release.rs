//! Typed linked-node destruction must release allocation even when the item destructor unwinds.
use super::BaseAllocator;
use allocator_api2::alloc::Allocator;
use std::{alloc::Layout, ptr::NonNull};
struct NodeAllocationRelease<F: FnOnce()> {
    release: Option<F>,
}
impl<F: FnOnce()> Drop for NodeAllocationRelease<F> {
    fn drop(&mut self) {
        // Exactly one cleanup consumes allocation provenance, including during unwinding.
        self.release.take().expect("owned node cleanup runs once")();
    }
}
pub(super) unsafe fn release_node_payload<T>(pointer: *mut T, release: impl FnOnce()) {
    let allocation = NodeAllocationRelease {
        release: Some(release),
    };
    // allocator-api2 0.4.0 Box::drop lacks unwind deallocation. Retain the allocation
    // guard while invoking user destruction, with the original allocator and layout.
    unsafe { std::ptr::drop_in_place(pointer) };
    drop(allocation);
}
pub(super) unsafe fn release_typed_node<T>(pointer: *mut T, allocator: &BaseAllocator) {
    let allocation = NonNull::new(pointer.cast()).expect("owned node is nonnull");
    let layout = Layout::new::<T>();
    // SAFETY: caller retains the original typed allocation and its exact layout.
    unsafe {
        release_node_payload(pointer, || allocator.deallocate(allocation, layout));
    }
}
