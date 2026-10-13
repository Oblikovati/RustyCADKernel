//! Current native Free resource receives the original allocation, with zero per-node ownership.
#![allow(unsafe_code)]
use allocator_api2::alloc::Allocator;
use rustycad_tkernel::{
    ncollection::{
        AllocatorDispatchCode, AllocatorInvocation, AllocatorObject, AllocatorSource, BaseAllocator,
    },
    standard::{Failure, Handle, SharedState, Transient},
};
use std::{alloc::Layout, ptr::NonNull};
#[derive(Default)]
struct CurrentFreeProbe {
    parent: AllocatorObject,
    events: SharedState<Vec<i32>>,
    failure_policy: u8,
}
rustycad_tkernel::impl_transient!(CurrentFreeProbe, "Martensite_CurrentFreeProbe", AllocatorObject,
    parent, interfaces: [dyn AllocatorSource]);
// SAFETY: both probes share Standard's process heap Free implementation.
unsafe impl AllocatorSource for CurrentFreeProbe {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // Borrow the genuine parent; native allocator counters belong to this actual object.
        &self.parent
    }
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            free: current_probe_free,
            ..Default::default()
        }
    }
}
#[test]
fn current_resource_receives_actual_original_pointer_without_node_count() {
    let old_owner = Handle::new(CurrentFreeProbe::default());
    let new_owner = Handle::new(CurrentFreeProbe::default());
    let old = BaseAllocator::from_handle(&old_owner);
    let current = BaseAllocator::from_handle(&new_owner);
    let lease = old.storage_lease();
    let layout = Layout::new::<u64>();
    let pointer = Allocator::allocate(&lease, layout).unwrap().cast::<u8>();
    assert_eq!((old_owner.ref_count(), new_owner.ref_count()), (2, 2));
    // SAFETY: no payload exists; both genuine allocators accept the original Standard pointer.
    unsafe { lease.deallocate_using(pointer, layout, &current) };
    assert!(old_owner.events.read().is_empty());
    assert_eq!(&*new_owner.events.read(), &[2]);
}
#[test]
fn native_static_const_allocator_reference_does_not_copy_ownership() {
    let original = AllocatorObject::common_base_allocator();
    let repeated = AllocatorObject::common_base_allocator();
    assert!(std::ptr::eq(original, repeated));
    let owner = BaseAllocator::from_handle(original);
    assert_eq!(owner.identity(), original.object_address());
    drop(owner);
}
#[test]
fn builtin_heap_records_restore_provenance_for_current_virtual_free() {
    let original = BaseAllocator::fresh_heap();
    let owner = Handle::new(CurrentFreeProbe::default());
    let current = BaseAllocator::from_handle(&owner);
    let layout = Layout::new::<u64>();
    let pointer = Allocator::allocate(&original, layout).unwrap().cast::<u8>();
    // SAFETY: both allocators use the same Standard physical heap allocation backend.
    unsafe { original.deallocate_using(pointer, layout, &current) };
    assert_eq!(&*owner.events.read(), &[2]);
}
#[test]
fn protected_base_construction_and_actual_source_alias_replay_native() {
    let stack = AllocatorObject::default();
    let mut rows = vec![format!("stack {}", stack.transient_base().ref_count())];
    let owner = Handle::new(AllocatorObject::default());
    rows.push(format!("owned {}", owner.ref_count()));
    let source = BaseAllocator::from_handle(&owner);
    rows.push(format!(
        "source {} {}",
        owner.ref_count(),
        i32::from(source.identity() == owner.object_address())
    ));
    drop(source);
    rows.push(format!("released {}", owner.ref_count()));
    let actual = rows.join("\n") + "\n";
    let native = include_str!("fixtures/allocator-construction-native.txt")
        .lines()
        .filter(|line| !line.starts_with("common "))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(actual, native);
}

fn record_current_probe(source: &CurrentFreeProbe) {
    source
        .events
        .write()
        .push(source.transient_base().ref_count());
}
unsafe fn current_probe_free(
    invocation: &AllocatorInvocation,
    pointer: *mut u8,
) -> Result<(), Failure> {
    // SAFETY: this step mutates only plain trace values, without callbacks or native retirement.
    let policy = rustycad_tkernel::standard::raise_native_failure(unsafe {
        invocation.capture(record_current_failure_policy)
    });
    current_free_failure_policy(policy)?;
    // SAFETY: the original Standard range is accepted by this actual parent.
    unsafe { invocation.free_base(pointer) }
}

fn record_current_failure_policy(source: &CurrentFreeProbe) -> u8 {
    record_current_probe(source);
    source.failure_policy
}
fn current_free_failure_policy(policy: u8) -> Result<(), Failure> {
    if policy == 1 {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            FREE_FAILURE_MESSAGE,
            "",
        ));
    }
    if policy == 2 {
        std::panic::panic_any(73_u32);
    }
    assert_eq!(
        policy, 0,
        "received Free policy={policy}; expected 0, 1 or 2"
    );
    Ok(())
}
type CurrentFreeOutcome = Result<Result<(), Failure>, Box<dyn std::any::Any + Send>>;
const FREE_FAILURE_MESSAGE: &str =
    "reset Free policy=1; expected successful original allocation release";

fn checked_current_free(
    original: &BaseAllocator,
    current: &BaseAllocator,
    pointer: NonNull<u8>,
) -> CurrentFreeOutcome {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rustycad_tkernel::standard::catch_native_failure(|| {
            // SAFETY: no payload exists and both sources use the actual Standard heap.
            unsafe { original.deallocate_using(pointer, Layout::new::<u64>(), current) };
            Ok(())
        })
    }))
}

fn assert_current_free_cause(outcome: CurrentFreeOutcome, policy: u8) {
    match (policy, outcome) {
        (0, Ok(Ok(()))) => {}
        (1, Ok(Err(failure))) => {
            assert_eq!(failure.exception_type(), "Standard_ConstructionError");
            assert_eq!(failure.message(), FREE_FAILURE_MESSAGE);
        }
        (2, Err(payload)) => assert_eq!(payload.downcast_ref::<u32>(), Some(&73)),
        (policy, _) => panic!(
            "received incompatible Free outcome for policy={policy}; expected success/typed ConstructionError/foreign payload73"
        ),
    }
}

fn replay_current_free_failure(policy: u8) {
    let owner = Handle::new(CurrentFreeProbe {
        failure_policy: policy,
        ..Default::default()
    });
    let current = BaseAllocator::from_handle(&owner);
    let original = current.storage_lease();
    let pointer = Allocator::allocate(&original, Layout::new::<u64>())
        .unwrap()
        .cast::<u8>();
    assert_eq!(owner.ref_count(), 2);
    let outcome = checked_current_free(&original, &current, pointer);
    recover_failed_current_free(&owner, pointer, policy);
    assert_eq!(&*owner.events.read(), &[2]);
    assert_current_free_cause(outcome, policy);
}

fn recover_failed_current_free(owner: &Handle<CurrentFreeProbe>, pointer: NonNull<u8>, policy: u8) {
    if policy == 0 {
        return;
    }
    let invocation = AllocatorInvocation::from_handle(owner).unwrap();
    // SAFETY: failed Free left the original raw allocation live; no payload destruction is repeated.
    unsafe { invocation.free_base(pointer.as_ptr()) }.unwrap();
}

#[test]
fn custom_current_free_preserves_typed_and_unrelated_failure_causes() {
    for policy in 0..3 {
        replay_current_free_failure(policy);
    }
}
