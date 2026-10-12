//! Rejected and unwinding native virtual allocations do not fabricate ownership or arena leases.
#![allow(unsafe_code)]
use allocator_api2::alloc::Allocator;
use rustycad_tkernel::{
    ncollection::{AllocatorObject, AllocatorSource, BaseAllocator, IncrementalAllocatorObject},
    standard::Handle,
};
use std::{alloc::Layout, ptr::NonNull};
#[derive(Default)]
struct NullAllocationProbe {
    parent: AllocatorObject,
}
rustycad_tkernel::impl_transient!(NullAllocationProbe, "Martensite_NullAllocationProbe",
    AllocatorObject, parent, interfaces: [dyn AllocatorSource]);
// SAFETY: NULL never exposes a range; inherited Optimal/Free retain Standard's valid contract.
unsafe impl AllocatorSource for NullAllocationProbe {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // Projection borrows this original native count base.
        &self.parent
    }
    fn allocate(&self, _size: usize) -> Option<NonNull<u8>> {
        // Native derived allocators may reject allocation without constructing payload storage.
        None
    }
}
#[derive(Default)]
struct ThrowingIncrementalProbe {
    parent: IncrementalAllocatorObject,
}
rustycad_tkernel::impl_transient!(ThrowingIncrementalProbe, "Martensite_ThrowingIncrementalProbe",
    IncrementalAllocatorObject, parent, interfaces: [dyn AllocatorSource]);
// SAFETY: failure exposes no region and cannot leave live storage in the parent engine.
unsafe impl AllocatorSource for ThrowingIncrementalProbe {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // The incremental forwarding body reads its actual embedded base policy.
        self.parent.allocator_object_base()
    }
    fn allocate_optimal(&self, size: usize) -> Option<NonNull<u8>> {
        // Inject the native virtual failure before allocation; the typed construction guard unwinds.
        panic!("requested allocation={size}; expected successful virtual allocator")
    }
}
#[test]
fn null_source_rejects_typed_and_bytes_without_inventing_ownership() {
    let owner = Handle::new(NullAllocationProbe::default());
    let resource = BaseAllocator::from_handle(&owner);
    assert!(resource.allocate_raw(8).is_none());
    assert!(resource.allocate(8).is_none());
    assert!(Allocator::allocate(&resource, Layout::new::<u64>()).is_err());
    assert_eq!(owner.ref_count(), 2);
    let pointer = resource.allocate_optimal_raw(8).unwrap();
    // SAFETY: default parent Optimal allocated this exact live range independently of Allocate.
    unsafe { resource.free_raw(pointer).unwrap() };
}
#[test]
fn virtual_unwind_releases_pending_incremental_lease_without_payload() {
    let owner = Handle::new(ThrowingIncrementalProbe::default());
    let resource = BaseAllocator::from_handle(&owner);
    let failure = std::panic::catch_unwind(|| resource.allocate_value(5_u64));
    assert!(failure.is_err());
    assert!(resource.reset(false).is_ok());
    assert_eq!(owner.ref_count(), 2);
}
