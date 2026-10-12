//! A flat physical-policy lease avoids native-call scratch allocation and logical receiver ownership.
#![allow(unsafe_code)] // Nullable release forwards the actual allocator's original-pointer contract.
use super::{AllocatorKind, BaseAllocator, heap_round, locked};
use crate::standard::Failure;
use crate::standard::memory::raw_heap::{
    MALLOC_ALIGNMENT, heap_allocate, heap_free, mm_malloc_alignment,
};
use std::{ptr::NonNull, sync::Arc};

/// Independent shared physical policy; `invocation.parent_policy()?.allocate_optimal(8)`.
/// Cloning this backing retains no native allocator reference and allocates no façade.
pub struct AllocatorPolicyInvocation {
    inner: Arc<AllocatorKind>,
}
impl BaseAllocator {
    pub(in crate::ncollection) fn policy_invocation(&self) -> AllocatorPolicyInvocation {
        // The engine remains shared and mutable; this is not a copied allocator receiver.
        AllocatorPolicyInvocation {
            inner: self.fields.inner.clone(),
        }
    }
}
impl AllocatorPolicyInvocation {
    /// Allocate from actual shared backing; `policy.allocate(8, true)`.
    pub fn allocate(&self, size: usize, zeroed: bool) -> Option<NonNull<u8>> {
        match &*self.inner {
            AllocatorKind::Common => heap_allocate(size, MALLOC_ALIGNMENT, zeroed),
            AllocatorKind::Heap => heap_allocate(heap_round(size)?, MALLOC_ALIGNMENT, zeroed),
            AllocatorKind::Incremental(engine) => locked(engine).allocate(size),
            AllocatorKind::Accumulating(engine) => locked(engine).allocate(size),
            AllocatorKind::Aligned(alignment) => {
                heap_allocate(size, mm_malloc_alignment(*alignment)?, zeroed)
            }
        }
    }
    /// Preserve native common-allocator uninitialized optimal allocation; `policy.allocate_optimal(8)`.
    pub fn allocate_optimal(&self, size: usize) -> Option<NonNull<u8>> {
        let zeroed = !matches!(&*self.inner, AllocatorKind::Common);
        self.allocate(size, zeroed)
    }
    /// Release accepted original backing; `unsafe { policy.free_nullable(pointer) }?`.
    /// # Safety
    /// A non-null pointer must be live and accepted by this actual physical policy.
    pub unsafe fn free_nullable(&self, pointer: *mut u8) -> Result<(), Failure> {
        match &*self.inner {
            AllocatorKind::Common | AllocatorKind::Heap | AllocatorKind::Aligned(_) => {
                if let Some(pointer) = NonNull::new(pointer) {
                    // SAFETY: the caller supplies this backing's original live allocation.
                    unsafe { heap_free(pointer) };
                }
                Ok(())
            }
            AllocatorKind::Incremental(_) => Ok(()),
            AllocatorKind::Accumulating(engine) => locked(engine).free(pointer as usize),
        }
    }
}
