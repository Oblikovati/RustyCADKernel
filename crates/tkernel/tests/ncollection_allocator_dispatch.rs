//! Real derived allocator dispatch and native noncounting storage contracts.
#![allow(unsafe_code)]
use rustycad_tkernel::{
    ncollection::{
        AllocatorDispatchCode, AllocatorInvocation, AllocatorObject, AllocatorSource, BaseAllocator,
    },
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
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            allocate: probe_allocate,
            allocate_optimal: probe_optimal,
            free: probe_free,
        }
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
    assert_eq!(
        std::mem::size_of::<BaseAllocator>(),
        std::mem::size_of::<usize>()
    );
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

fn capture_probe_events(source: &AllocatorProbe) -> (Arc<SharedState<Vec<String>>>, i32) {
    (
        source.events.clone(),
        source.parent.transient_base().ref_count(),
    )
}
fn record_probe_event(invocation: &AllocatorInvocation, operation: &str) {
    // SAFETY: the selector captures independently anchored trace storage and an immediate native count.
    let (events, count) = rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(capture_probe_events)
    });
    events.write().push(format!("{operation} {count}"));
}
fn probe_allocate(invocation: &AllocatorInvocation, size: usize) -> Option<NonNull<u8>> {
    record_probe_event(invocation, &format!("allocate {size}"));
    invocation.allocate_base(size)
}
fn probe_optimal(invocation: &AllocatorInvocation, size: usize) -> Option<NonNull<u8>> {
    record_probe_event(invocation, &format!("optimal {size}"));
    invocation.allocate_optimal_base(size)
}
unsafe fn probe_free(invocation: &AllocatorInvocation, pointer: *mut u8) -> Result<(), Failure> {
    record_probe_event(invocation, "free");
    // SAFETY: the original live pointer is passed through unchanged.
    unsafe { invocation.free_base(pointer) }
}

#[test]
fn invocation_and_parent_policy_preserve_noncounting_identity() {
    let owner = Handle::new(AllocatorProbe::default());
    let invocation = AllocatorInvocation::from_handle(&owner).unwrap();
    let policy = invocation.parent_policy().unwrap();
    assert_eq!(owner.ref_count(), 1);
    let pointer = policy.allocate_optimal(8).unwrap();
    drop(owner);
    // Physical backing survives logical destruction without making the receiver callable again.
    unsafe { policy.free_nullable(pointer.as_ptr()).unwrap() };
    assert!(invocation.parent_policy().is_err());
}

#[test]
fn invocation_capture_rejects_expired_and_wrong_actual_receivers() {
    let owner = Handle::new(AllocatorProbe::default());
    let invocation = AllocatorInvocation::from_handle(&owner).unwrap();
    // SAFETY: these selectors only inspect actual immutable receiver fields.
    let actual = unsafe { invocation.capture::<AllocatorProbe, _>(probe_count) }.unwrap();
    assert_eq!(actual, 1);
    assert!(unsafe { invocation.capture::<rustycad_tkernel::ncollection::HeapAllocatorObject, _>(|_| 0) }.is_err());
    drop(owner);
    assert!(unsafe { invocation.capture::<AllocatorProbe, _>(probe_count) }.is_err());
}
fn probe_count(probe: &AllocatorProbe) -> i32 {
    // Count reads do not invoke native callbacks or retain the actual receiver.
    probe.transient_base().ref_count()
}
