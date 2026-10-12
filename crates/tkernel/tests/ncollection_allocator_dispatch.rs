//! Real derived allocator dispatch and native noncounting storage contracts.
#![allow(unsafe_code)]
use rustycad_tkernel::{
    ncollection::{AllocatorObject, AllocatorSource, BaseAllocator},
    standard::{Failure, Handle, RuntimeTyped, SharedState, Transient},
};
use std::{ptr::NonNull, sync::Arc};
#[derive(Default)]
struct AllocatorProbe {
    parent: AllocatorObject,
    events: Arc<SharedState<Vec<String>>>,
}
rustycad_tkernel::impl_transient!(AllocatorProbe, "Martensite_AllocatorProbe", AllocatorObject,
    parent, interfaces: [dyn AllocatorSource]);
// SAFETY: the probe forwards every range to its genuine Standard allocator parent.
unsafe impl AllocatorSource for AllocatorProbe {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // Borrow the genuine parent; native allocator counters belong to this actual object.
        &self.parent
    }
    fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        self.events.write().push(format!(
            "allocate {size} {}",
            self.parent.transient_base().ref_count()
        ));
        self.parent.allocate_base(size)
    }
    fn allocate_optimal(&self, size: usize) -> Option<NonNull<u8>> {
        self.events.write().push(format!(
            "optimal {size} {}",
            self.parent.transient_base().ref_count()
        ));
        self.parent.allocate_optimal_base(size)
    }
    unsafe fn free(&self, pointer: *mut u8) -> Result<(), Failure> {
        self.events
            .write()
            .push(format!("free {}", self.parent.transient_base().ref_count()));
        // SAFETY: the original live pointer is passed through unchanged.
        unsafe { self.parent.free_base(pointer) }
    }
}
#[test]
fn derived_virtuals_replay_native_count_and_payload_order() {
    let owner = Handle::new(AllocatorProbe::default());
    let resource = BaseAllocator::from_handle(&owner);
    let pointer = resource.allocate_optimal_raw(16).unwrap();
    // SAFETY: this is the probe's original live allocation.
    unsafe { resource.free_raw(pointer).unwrap() };
    let payload = resource.allocate_value(29_u64);
    assert_eq!((owner.ref_count(), *payload), (2, 29));
    drop(payload);
    let rows = owner.events.read().join("\n") + "\n";
    assert_eq!(rows, include_str!("fixtures/allocator-dispatch-native.txt"));
}
#[test]
fn repeated_registration_preserves_identity_without_self_cycles() {
    let owner = Handle::new(AllocatorProbe::default());
    let weak = owner.downgrade();
    assert_eq!(std::mem::size_of::<BaseAllocator>(), std::mem::size_of::<usize>());
    let first = BaseAllocator::from_handle(&owner);
    let second = BaseAllocator::from_handle(&owner);
    assert_eq!(first.identity(), owner.object_address());
    assert_eq!(first, second);
    assert_eq!(owner.ref_count(), 3);
    drop((first, second));
    assert_eq!(owner.ref_count(), 1);
    drop(owner);
    assert!(weak.observe().is_null());
}
#[test]
fn source_descriptor_and_cast_keep_actual_derived_identity() {
    let owner = Handle::new(AllocatorProbe::default());
    let resource = BaseAllocator::from_handle(&owner);
    let projected = owner.cast::<dyn AllocatorSource>();
    assert!(std::ptr::eq(
        <dyn AllocatorSource>::static_type(),
        AllocatorObject::static_type()
    ));
    assert_eq!(projected.object_address(), resource.identity());
    assert_eq!(owner.ref_count(), 3);
    drop(projected);
    assert_eq!(owner.ref_count(), 2);
}
#[test]
fn physical_storage_lease_and_clone_do_not_add_intrusive_ownership() {
    let owner = Handle::new(AllocatorProbe::default());
    let resource = BaseAllocator::from_handle(&owner);
    let lease = resource.storage_lease();
    let second = lease.clone();
    assert_eq!(
        (owner.ref_count(), lease.ref_count(), second.ref_count()),
        (2, 2, 2)
    );
    drop((lease, second));
    assert_eq!(owner.ref_count(), 2);
}
#[repr(align(128))]
struct OveralignedProbe(u8);
#[test]
fn virtual_typed_alignment_returns_original_free_pointer() {
    let owner = Handle::new(AllocatorProbe::default());
    let resource = BaseAllocator::from_handle(&owner);
    let payload = resource.allocate_value(OveralignedProbe(7));
    assert_eq!(std::ptr::from_ref(&*payload).addr() % 128, 0);
    assert_eq!(payload.0, 7);
    drop(payload);
    assert_eq!(&*owner.events.read(), &["allocate 255 2", "free 2"]);
}
