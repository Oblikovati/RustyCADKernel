//! Typed allocation borrows native sources while physical leases preserve safe Rust storage.
use super::raw_heap::heap_free;
use super::{AllocatorKind, BaseAllocator, Layout, locked};
use std::ptr::NonNull;
impl BaseAllocator {
    /// Typed allocation for RAII owners and collections, honouring the layout alignment.
    pub(super) fn allocate_typed(&self, layout: Layout) -> Option<NonNull<u8>> {
        if self.uses_custom_source()
            || matches!(
                &*self.fields.inner,
                AllocatorKind::Common | AllocatorKind::Heap | AllocatorKind::Aligned(_)
            )
        {
            return self.allocate_virtual_typed(layout);
        }
        self.allocate_builtin_arena(layout)
    }
    fn allocate_builtin_arena(&self, layout: Layout) -> Option<NonNull<u8>> {
        match &*self.fields.inner {
            AllocatorKind::Incremental(engine) => {
                let mut engine = locked(engine);
                let address = engine.allocate_aligned(layout.size(), layout.align())?;
                engine.acquire();
                Some(address)
            }
            AllocatorKind::Accumulating(engine) => {
                locked(engine).allocate_aligned(layout.size(), layout.align())
            }
            _ => unreachable!("heap typed allocation uses virtual provenance records"),
        }
    }

    /// Releases a raw-policy owner or a typed arena allocation.
    ///
    /// # Safety
    /// `address` must come from this allocator's raw policy or typed arena and not be freed.
    /// Layout-bearing heap releases restore the separately recorded original allocation pointer.
    pub(super) unsafe fn free_typed(&self, address: NonNull<u8>) {
        if self.storage_source_retired() || self.uses_custom_source() {
            // SAFETY: live virtual sources or independent backing accept original raw byte storage.
            unsafe { self.free_raw(address) }
                .expect("owned raw storage must be accepted by its live provider");
            self.release_virtual_lease();
            return;
        }
        // SAFETY: original arena leases and heap raw provenance remain valid.
        unsafe { self.free_builtin_typed(address) };
    }
    unsafe fn free_builtin_typed(&self, address: NonNull<u8>) {
        match &*self.fields.inner {
            AllocatorKind::Incremental(engine) => locked(engine).release(),
            AllocatorKind::Accumulating(engine) => {
                // Owned layouts can exceed native lookup windows; actual block extents retain them.
                locked(engine)
                    .free_owned(address.as_ptr().addr())
                    .expect("owned accumulating allocation must retain its counted block");
            }
            // SAFETY: forwarded from this function's contract.
            _ => unsafe { heap_free(address) },
        }
    }
}
