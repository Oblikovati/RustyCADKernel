//! Custom virtual callbacks must not rewind an incremental arena while typed storage is in flight.
use super::{AllocatorKind, BaseAllocator};
impl BaseAllocator {
    pub(super) fn acquire_virtual_lease(&self) {
        if let AllocatorKind::Incremental(engine) = &*self.fields.inner {
            // Acquire before arbitrary Allocate callbacks can invoke Reset on the same resource.
            super::locked(engine).acquire();
        }
    }
    pub(super) fn release_virtual_lease(&self) {
        if let AllocatorKind::Incremental(engine) = &*self.fields.inner {
            // The real payload is gone before its physical arena lease is retired.
            super::locked(engine).release();
        }
    }
}
struct VirtualLeaseConstruction<'a> {
    resource: &'a BaseAllocator,
    armed: bool,
}
impl Drop for VirtualLeaseConstruction<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.resource.release_virtual_lease();
        }
    }
}
impl BaseAllocator {
    pub(super) fn allocate_leased_virtual_raw(&self, size: usize) -> Option<std::ptr::NonNull<u8>> {
        self.acquire_virtual_lease();
        let mut pending = VirtualLeaseConstruction {
            resource: self,
            armed: true,
        };
        let result = self.allocate_raw(size)?;
        pending.armed = false;
        Some(result)
    }
}
