//! Native collection Free selects the current resource after payload destruction callbacks.
#![allow(unsafe_code)]
use super::super::node_release::release_node_payload;
use super::BaseAllocator;
use std::{alloc::Layout, ptr::NonNull};
#[cfg(test)]
#[path = "current_release_tests.rs"]
mod tests;

pub(super) unsafe fn release<T>(
    pointer: *mut T,
    original: BaseAllocator,
    current: impl FnOnce() -> BaseAllocator,
) {
    let allocation = NonNull::new(pointer.cast()).expect("owned node is nonnull");
    let layout = Layout::new::<T>();
    // SAFETY: the owner holds provenance; caller guarantees current Free accepts it.
    unsafe {
        release_node_payload(pointer, || {
            let current = current();
            original.deallocate_using(allocation, layout, &current);
        });
    }
}
