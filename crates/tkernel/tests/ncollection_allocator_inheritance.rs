//! Native inherited Allocate/AllocateOptimal bodies retain virtual dispatch.
#![allow(unsafe_code)]
use rustycad_tkernel::{
    ncollection::{
        AllocatorObject, AllocatorSource, BaseAllocator, HeapAllocatorObject,
        IncrementalAllocatorObject,
    },
    standard::{Handle, SharedState, Transient},
};
use std::ptr::NonNull;
#[derive(Default)]
struct HeapForwardingProbe {
    parent: HeapAllocatorObject,
    events: SharedState<Vec<String>>,
}
rustycad_tkernel::impl_transient!(HeapForwardingProbe, "Martensite_HeapForwardingProbe",
    HeapAllocatorObject, parent, interfaces: [dyn AllocatorSource]);
// SAFETY: allocation delegates to the genuine embedded heap policy.
unsafe impl AllocatorSource for HeapForwardingProbe {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // Preserve the exact native inherited forwarding base and reference-count identity.
        self.parent.allocator_object_base()
    }
    fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        self.events.write().push(format!(
            "heap-allocate {size} {}",
            self.transient_base().ref_count()
        ));
        self.allocator_object_base().allocate_base(size)
    }
}
#[derive(Default)]
struct IncForwardingProbe {
    parent: IncrementalAllocatorObject,
    events: SharedState<Vec<String>>,
}
rustycad_tkernel::impl_transient!(IncForwardingProbe, "Martensite_IncForwardingProbe",
    IncrementalAllocatorObject, parent, interfaces: [dyn AllocatorSource]);
// SAFETY: allocation delegates to the genuine embedded incremental policy.
unsafe impl AllocatorSource for IncForwardingProbe {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // Preserve the exact native inherited forwarding base and reference-count identity.
        self.parent.allocator_object_base()
    }
    fn allocate_optimal(&self, size: usize) -> Option<NonNull<u8>> {
        self.events.write().push(format!(
            "inc-optimal {size} {}",
            self.transient_base().ref_count()
        ));
        self.allocator_object_base().allocate_optimal_base(size)
    }
}
#[test]
fn derived_parent_forwarding_matches_actual_native_calls() {
    let heap = Handle::new(HeapForwardingProbe::default());
    let resource = BaseAllocator::from_handle(&heap);
    let pointer = resource.allocate_optimal_raw(16).unwrap();
    // SAFETY: original live heap range is released once.
    unsafe { resource.free_raw(pointer).unwrap() };
    let inc = Handle::new(IncForwardingProbe::default());
    let resource = BaseAllocator::from_handle(&inc);
    let pointer = resource.allocate_raw(16).unwrap();
    // SAFETY: incremental Free ignores its still-live range.
    unsafe { resource.free_raw(pointer).unwrap() };
    let rows = [heap.events.read().join("\n"), inc.events.read().join("\n")].join("\n") + "\n";
    assert_eq!(
        rows,
        include_str!("fixtures/allocator-forwarding-native.txt")
    );
}
#[test]
fn custom_incremental_typed_lease_blocks_reset_until_payload_release() {
    let owner = Handle::new(IncForwardingProbe::default());
    let resource = BaseAllocator::from_handle(&owner);
    let payload = resource.allocate_value(8_u64);
    assert_eq!(owner.ref_count(), 2);
    assert!(resource.reset(false).is_err());
    drop(payload);
    assert!(resource.reset(false).is_ok());
    assert_eq!(owner.ref_count(), 2);
}
