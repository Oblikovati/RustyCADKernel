//! Builtin Incremental arena retirement forwards actual current no-op Free and preserves its failures.
#![allow(unsafe_code)] // Test ranges originate from the real arena; no-op Free preserves their raw storage.
use super::super::BaseAllocator;
use crate::ncollection::{
    AllocatorDispatchCode, AllocatorInvocation, AllocatorObject, AllocatorSource,
    IncrementalAllocatorObject,
};
use crate::standard::{Failure, Handle, Transient, catch_native_failure};
use std::{
    alloc::Layout,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr::NonNull,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicUsize, Ordering},
    },
};
#[derive(Default)]
struct TypedIncFreeTrace {
    outcome: AtomicU8,
    calls: AtomicUsize,
    received: AtomicUsize,
    parent_completed: AtomicBool,
    owners: AtomicI32,
}
struct TypedIncFreeProbe {
    parent: IncrementalAllocatorObject,
    trace: Arc<TypedIncFreeTrace>,
}
crate::impl_transient!(TypedIncFreeProbe,"Martensite_TypedIncFreeProbe",IncrementalAllocatorObject,parent,interfaces:[dyn AllocatorSource]);
// SAFETY: allocations use the genuine incremental parent; Free is its native no-op for arbitrary ranges.
unsafe impl AllocatorSource for TypedIncFreeProbe {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // This projection reaches the real owned superclass policy, never Standard heap Free.
        self.parent.allocator_object_base()
    }
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            free: typed_inc_probe_free,
            ..Default::default()
        }
    }
}
fn capture_typed_inc_trace(source: &TypedIncFreeProbe) -> (Arc<TypedIncFreeTrace>, i32) {
    // Immutable field metadata capture has no callbacks and retains no logical allocator owner.
    (source.trace.clone(), source.transient_base().ref_count())
}
unsafe fn typed_inc_probe_free(
    invocation: &AllocatorInvocation,
    pointer: *mut u8,
) -> Result<(), Failure> {
    // SAFETY: pure capture ends before executing the virtual callback body.
    let (trace, owners) = unsafe { invocation.capture(capture_typed_inc_trace) }?;
    // SAFETY: the actual incremental qualified parent intentionally ignores this pointer.
    unsafe { invocation.free_base(pointer) }?;
    trace.parent_completed.store(true, Ordering::Relaxed);
    trace.received.store(pointer as usize, Ordering::Relaxed);
    trace.calls.fetch_add(1, Ordering::Relaxed);
    trace.owners.store(owners, Ordering::Relaxed);
    typed_inc_probe_outcome(trace.outcome.load(Ordering::Relaxed))
}
fn typed_inc_probe_outcome(outcome: u8) -> Result<(), Failure> {
    match outcome {
        1 => Err(Failure::with_message(
            "Standard_ConstructionError",
            "incremental Free outcome=1; expected successful qualified no-op completion",
        )),
        2 => std::panic::panic_any(73_u32),
        _ => Ok(()),
    }
}
fn typed_inc_probe(outcome: u8) -> (Handle<TypedIncFreeProbe>, BaseAllocator) {
    let trace = Arc::new(TypedIncFreeTrace::default());
    trace.outcome.store(outcome, Ordering::Relaxed);
    let owner = Handle::new(TypedIncFreeProbe {
        parent: IncrementalAllocatorObject::default(),
        trace,
    });
    let current = BaseAllocator::from_handle(&owner);
    (owner, current)
}
fn actual_builtin_inc_pointer(original: &BaseAllocator) -> NonNull<u8> {
    let pointer = original.allocate_typed(Layout::new::<i32>()).unwrap();
    assert!(!original.has_virtual_allocation(pointer));
    assert!(original.allocate_calls_optimal());
    // SAFETY: the arena supplies aligned writable i32 storage; payload destruction precedes Free.
    unsafe {
        pointer.cast::<i32>().as_ptr().write(17);
        std::ptr::drop_in_place(pointer.cast::<i32>().as_ptr());
    }
    pointer
}
fn retire_actual_inc_pointer(
    original: &BaseAllocator,
    current: &BaseAllocator,
    pointer: NonNull<u8>,
) -> Result<Result<(), Failure>, Box<dyn std::any::Any + Send>> {
    catch_unwind(AssertUnwindSafe(|| {
        catch_native_failure(|| {
            // SAFETY: original arena remains live; current's genuine incremental Free accepts any pointer without freeing it.
            unsafe { original.deallocate_using(pointer, Layout::new::<i32>(), current) };
            Ok(())
        })
    }))
}
fn assert_typed_inc_outcome(
    outcome: u8,
    result: Result<Result<(), Failure>, Box<dyn std::any::Any + Send>>,
) {
    match (outcome, result) {
        (0, Ok(Ok(()))) => (),
        (1, Ok(Err(failure))) => {
            assert_eq!(failure.exception_type(), "Standard_ConstructionError");
            assert_eq!(
                failure.message(),
                "incremental Free outcome=1; expected successful qualified no-op completion"
            );
        }
        (2, Err(payload)) => assert_eq!(payload.downcast_ref::<u32>(), Some(&73)),
        (actual, _) => panic!(
            "incremental Free outcome={actual}; expected exact success, typed cause or original foreign payload"
        ),
    }
}
fn check_actual_inc_retirement(outcome: u8) -> Vec<Vec<i32>> {
    let original = BaseAllocator::incremental(4096);
    let pointer = actual_builtin_inc_pointer(&original);
    // SAFETY: read only physical arena bytes; the destroyed typed payload is not revived.
    let original_byte = unsafe { pointer.as_ptr().read() };
    let (owner, current) = typed_inc_probe(outcome);
    record_initial_inc_retirement(outcome, &original, &current, pointer);
    let mut rows = initial_inc_retirement_rows(outcome, &owner, pointer, original_byte);
    recover_actual_inc_retirement(outcome, &original, &current, pointer, &owner);
    if outcome != 0 {
        rows.push(retry_inc_retirement_row(
            outcome,
            &owner,
            pointer,
            original_byte,
        ));
    }
    original.reset(true).unwrap();
    rows
}
fn retry_inc_retirement_row(
    outcome: u8,
    owner: &TypedIncFreeProbe,
    pointer: NonNull<u8>,
    original_byte: u8,
) -> Vec<i32> {
    // Phase one records only successful raw no-op retry, while original storage remains alive.
    actual_inc_retirement_row(outcome, 1, 0, owner, pointer, original_byte)
}
fn record_initial_inc_retirement(
    outcome: u8,
    original: &BaseAllocator,
    current: &BaseAllocator,
    pointer: NonNull<u8>,
) {
    // The explicit no-virtual-provenance assertions in setup select only the second retirement site.
    assert_typed_inc_outcome(
        outcome,
        retire_actual_inc_pointer(original, current, pointer),
    );
}
fn initial_inc_retirement_rows(
    outcome: u8,
    owner: &TypedIncFreeProbe,
    pointer: NonNull<u8>,
    original_byte: u8,
) -> Vec<Vec<i32>> {
    // Exact native rows retain actual traces, not guessed second-site eligibility.
    vec![actual_inc_retirement_row(
        outcome,
        0,
        outcome,
        owner,
        pointer,
        original_byte,
    )]
}
fn recover_actual_inc_retirement(
    outcome: u8,
    original: &BaseAllocator,
    current: &BaseAllocator,
    pointer: NonNull<u8>,
    owner: &TypedIncFreeProbe,
) {
    if outcome != 0 {
        assert!(original.reset(false).is_err());
        owner.trace.outcome.store(0, Ordering::Relaxed);
        // Native raw Inc Free may safely repeat its no-op; no payload/node destruction is retried.
        assert_typed_inc_outcome(0, retire_actual_inc_pointer(original, current, pointer));
        assert_eq!(owner.trace.calls.load(Ordering::Relaxed), 2);
    }
}
fn actual_inc_retirement_row(
    outcome: u8,
    phase: i32,
    fault: u8,
    owner: &TypedIncFreeProbe,
    pointer: NonNull<u8>,
    original_byte: u8,
) -> Vec<i32> {
    let mut row = vec![outcome.into(), phase, fault.into()];
    row.extend(actual_inc_retirement_trace(owner, pointer, original_byte));
    row
}
fn actual_inc_retirement_trace(
    owner: &TypedIncFreeProbe,
    pointer: NonNull<u8>,
    original_byte: u8,
) -> [i32; 5] {
    // SAFETY: actual incremental no-op Free preserved original arena bytes until the caller's Reset.
    let unchanged = unsafe { pointer.as_ptr().read() } == original_byte;
    [
        owner.trace.calls.load(Ordering::Relaxed) as i32,
        owner.trace.parent_completed.load(Ordering::Relaxed).into(),
        i32::from(owner.trace.received.load(Ordering::Relaxed) == pointer.as_ptr() as usize),
        unchanged.into(),
        owner.trace.owners.load(Ordering::Relaxed),
    ]
}
fn native_inc_retirement_rows() -> Vec<Vec<i32>> {
    include_str!("../../../tests/fixtures/incremental-current-free-failure-native.txt")
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|value| value.parse().unwrap())
                .collect()
        })
        .collect()
}
#[test]
fn builtin_incremental_current_noop_free_success_typed_foreign_and_raw_retry_release_match() {
    let rows: Vec<Vec<i32>> = (0..3).flat_map(check_actual_inc_retirement).collect();
    assert_eq!(rows, native_inc_retirement_rows());
}
