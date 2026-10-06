//! Buckets use Standard's optimal raw policy; nodes retain independent global ownership.
use crate::ncollection::BaseAllocator;
use std::alloc::{Layout, handle_alloc_error};
use std::ptr::NonNull;
#[cfg(test)]
pub(in super::super) mod observation;

pub(super) fn reserve(layout: Layout) -> NonNull<u8> {
    // Pinned Standard::AllocateOptimal/Free use the native heap; keep this allocation
    // separate from global node Box allocation while preserving malloc alignment.
    #[cfg(test)]
    observation::allocation(layout.size());
    BaseAllocator::common()
        .allocate_optimal_raw(layout.size())
        .unwrap_or_else(|| handle_alloc_error(layout))
}

/// The caller supplies this policy's live allocation, or the native null table.
pub(super) unsafe fn release(address: *mut u8) {
    #[cfg(test)]
    observation::release(address.is_null());
    // Standard::Free(nullptr) must not lazily allocate a common-allocator handle.
    if address.is_null() {
        return;
    }
    // SAFETY: the table retains the exact pointer returned by reserve until this final release.
    unsafe { BaseAllocator::common().free_raw_nullable(address) }
        .expect("common bucket allocation: expected matching raw release");
}
