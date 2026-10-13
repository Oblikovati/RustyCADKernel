//! Preserve the original virtual allocation pointer across typed reborrows and alignment padding.
use super::BaseAllocator;
use crate::ncollection::{
    AccumulatingAllocatorObject, AlignedAllocatorObject, AllocatorObject, HeapAllocatorObject,
    IncrementalAllocatorObject,
};
use crate::standard::RuntimeTyped;
use std::{
    alloc::Layout,
    collections::HashMap,
    ptr::NonNull,
    sync::{
        Mutex,
        atomic::{AtomicPtr, Ordering},
    },
};
pub(super) type VirtualAllocationRecords = Mutex<HashMap<usize, AtomicPtr<u8>>>;
impl BaseAllocator {
    pub(super) fn uses_custom_source(&self) -> bool {
        let Some(source) = self.source_observer() else {
            return false;
        };
        let descriptor = source
            .dynamic_type()
            .expect("non-null allocator descriptor");
        !builtin_source_descriptor(descriptor)
    }
    pub(super) fn allocate_virtual_typed(&self, layout: Layout) -> Option<NonNull<u8>> {
        let request = virtual_allocation_size(layout)?;
        let original = self.allocate_leased_virtual_raw(request)?;
        let Some(result) = aligned_virtual_address(original, layout) else {
            // SAFETY: reject ranges unsuitable for native placement and return original provenance.
            let _ = unsafe { self.free_raw(original) };
            self.release_virtual_lease();
            return None;
        };
        super::locked(&self.fields.virtual_allocations)
            .insert(result.as_ptr().addr(), AtomicPtr::new(original.as_ptr()));
        Some(result)
    }
    pub(super) fn has_virtual_allocation(&self, pointer: NonNull<u8>) -> bool {
        let records = super::locked(&self.fields.virtual_allocations);
        records.contains_key(&pointer.as_ptr().addr())
    }
    pub(super) unsafe fn free_virtual_typed(&self, pointer: NonNull<u8>) {
        // SAFETY: the allocation's original source accepts its recorded original pointer.
        unsafe { self.free_virtual_typed_using(pointer, self) };
    }
    pub(super) unsafe fn free_virtual_typed_using(&self, pointer: NonNull<u8>, current: &Self) {
        let original = super::locked(&self.fields.virtual_allocations)
            .remove(&pointer.as_ptr().addr())
            .expect("typed allocation address must retain original virtual allocation provenance");
        // SAFETY: caller guarantees current allocator accepts the original raw allocation.
        // Native virtual Free may raise a typed exception after payload destruction; keep its cause.
        crate::standard::raise_native_failure(unsafe {
            current.free_raw_nullable(original.load(Ordering::Relaxed))
        });
        self.release_virtual_lease();
    }
    /// Release through the current native collection resource; `unsafe { original.deallocate_using(p, layout, &current) }`.
    /// # Safety
    /// Pointer/layout must denote this resource's live typed allocation; current must accept
    /// its original raw allocation. The caller must already have destroyed its payload.
    pub unsafe fn deallocate_using(&self, pointer: NonNull<u8>, layout: Layout, current: &Self) {
        if layout.size() == 0 {
            return;
        }
        if self.has_virtual_allocation(pointer) {
            // SAFETY: caller supplies live provenance and compatible actual current Free.
            unsafe { self.free_virtual_typed_using(pointer, current) };
            return;
        }
        // SAFETY: caller's accepted current resource is forwarded for the original live arena.
        unsafe { self.deallocate_arena_using(pointer, current) };
    }
    unsafe fn deallocate_arena_using(&self, pointer: NonNull<u8>, current: &Self) {
        if self.allocate_calls_optimal() {
            // SAFETY: current Free accepts this raw arena range, according to the caller contract.
            // The same native Free exception contract applies to raw arena retirement.
            crate::standard::raise_native_failure(unsafe { current.free_raw(pointer) });
            self.release_virtual_lease();
            return;
        }
        assert_eq!(
            self.identity(),
            current.identity(),
            "accumulating resource identities differ; expected same live arena"
        );
        // SAFETY: same arena identity preserves its original typed lease and layout.
        unsafe { self.free_typed(pointer) };
    }
}

fn builtin_source_descriptor(descriptor: &crate::standard::RuntimeType) -> bool {
    [
        AllocatorObject::static_type(),
        HeapAllocatorObject::static_type(),
        IncrementalAllocatorObject::static_type(),
        AccumulatingAllocatorObject::static_type(),
        AlignedAllocatorObject::static_type(),
    ]
    .into_iter()
    .any(|known| std::ptr::eq(descriptor, known))
}

fn virtual_allocation_size(layout: Layout) -> Option<usize> {
    let padding = if layout.align() > super::MALLOC_ALIGNMENT {
        layout.align() - 1
    } else {
        0
    };
    layout.size().checked_add(padding)
}
fn aligned_virtual_address(original: NonNull<u8>, layout: Layout) -> Option<NonNull<u8>> {
    let aligned = original.as_ptr().addr().checked_add(layout.align() - 1)? & !(layout.align() - 1);
    if layout.align() <= super::MALLOC_ALIGNMENT && aligned != original.as_ptr().addr() {
        return None;
    }
    NonNull::new(original.as_ptr().with_addr(aligned))
}

#[cfg(test)]
#[path = "typed_inc_free_failure_tests.rs"]
mod typed_inc_free_failure_tests;
