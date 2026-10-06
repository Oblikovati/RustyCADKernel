use super::super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[derive(Copy)]
struct CloneObservable<'a> {
    value: i32,
    calls: &'a Cell<usize>,
}
#[allow(clippy::non_canonical_clone_impl)]
impl Clone for CloneObservable<'_> {
    fn clone(&self) -> Self {
        self.calls.set(self.calls.get() + 1);
        *self
    }
}
#[test]
fn trivial_policy_does_not_call_an_observable_clone_implementation() {
    let calls = Cell::new(0);
    let mut source = DynamicArray::with_increment(2);
    source.append(CloneObservable {
        value: 4,
        calls: &calls,
    });
    let mut destination = DynamicArray::with_increment(2);
    destination
        .storage_extension()
        .copy_date_trivial(&source)
        .unwrap();
    assert_eq!(calls.get(), 0);
    assert_eq!(destination.value(0).unwrap().value, 4);
    let cloned = source.clone();
    assert_eq!(calls.get(), 1);
    assert_eq!(cloned.value(0).unwrap().value, 4);
}
fn assert_domain_failure(error: DynamicArrayCopyFailure<CopyDateItem>) {
    assert!(!error.has_incomplete_block());
    assert!(error.incomplete_values().is_empty());
    assert!(
        error
            .failure()
            .message()
            .contains("expected unallocated destination")
    );
    assert!(format!("{error:?}").contains("initialized_values: 0"));
    assert_eq!(error.to_string(), error.failure().to_string());
    let source = std::error::Error::source(&error).unwrap();
    assert_eq!(source.to_string(), error.failure().to_string());
}
#[test]
fn mismatched_stride_rejects_before_allocation_or_copying() {
    let (source, observations) = source_items(3);
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let mut target = DynamicArray::with_allocator(4, Some(allocator.clone()));
    let error = target
        .storage_extension()
        .copy_date_with(&source, |item| item.copy_construct(0))
        .unwrap_err();
    assert_domain_failure(error);
    assert_eq!(observations.attempts.get(), 0);
    assert_eq!(live_allocations(&allocator), 0);
}
#[test]
fn retained_destination_rejects_without_changing_owned_values() {
    let (source, observations) = source_items(3);
    let (mut target, allocator) = target_items();
    target.append(CopyDateItem {
        value: 9,
        observations: Rc::clone(&observations),
    });
    let error = target
        .storage_extension()
        .copy_date_with(&source, |item| item.copy_construct(0))
        .unwrap_err();
    assert_domain_failure(error);
    assert_eq!(target.value(0).unwrap().value, 9);
    assert_eq!(observations.attempts.get(), 0);
    assert_eq!(live_allocations(&allocator), 1);
}
#[test]
fn failed_block_remains_owned_and_readable_after_destination_drop() {
    let (source, observations) = source_items(3);
    let (mut target, allocator) = target_items();
    let error = target
        .storage_extension()
        .copy_date_with(&source, |item| item.copy_construct(2))
        .unwrap_err();
    drop(target);
    assert_eq!(live_allocations(&allocator), 1);
    assert_eq!(observations.drops.get(), 0);
    assert_eq!(error.incomplete_values()[0].value, 1);
    drop(error);
    assert_eq!(observations.drops.get(), 1);
    assert_eq!(live_allocations(&allocator), 0);
}
#[test]
fn rust_callback_panic_cleans_unpublished_block_as_documented() {
    let (source, observations) = source_items(3);
    let (mut target, allocator) = target_items();
    let result = catch_unwind(AssertUnwindSafe(|| {
        target
            .storage_extension()
            .copy_date_with(&source, |item| {
                assert_ne!(item.value, 2, "copy value2: expected injected Rust panic");
                item.copy_construct(0)
            })
            .unwrap();
    }));
    assert!(result.is_err());
    assert_eq!(target.size(), 0);
    assert_eq!(observations.drops.get(), 1);
    assert_eq!(live_allocations(&allocator), 0);
}
#[test]
fn trivial_policy_reports_storage_failure_without_allocating() {
    let source = DynamicArray::<i32>::with_increment(2);
    let mut target = DynamicArray::<i32>::with_increment(4);
    let error = target
        .storage_extension()
        .copy_date_trivial(&source)
        .unwrap_err();
    assert!(error.message().contains("stride 4"));
    assert_eq!(target.storage_extension().block_count(), 0);
}
