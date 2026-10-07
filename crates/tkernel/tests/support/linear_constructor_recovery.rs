//! Test-only typed repair; no constructor invokes this as normal recovery.
#![allow(unsafe_code)]
use super::item::{CopyItem, CopyTrace};

pub(crate) fn repair(
    name: &str,
    expected_capacity: usize,
    trace: &CopyTrace,
    allocation: (*mut CopyItem<'_>, usize),
    assert_native: impl Fn(&str),
) {
    let (pointer, capacity) = allocation;
    assert_eq!(capacity, expected_capacity);
    let count = trace.completed.get();
    // SAFETY: the failed construction observer captured fresh storage provenance. Exactly
    // count copies returned successfully; the failed next slot never became a live CopyItem.
    let values: String = unsafe { std::slice::from_raw_parts(pointer, count) }
        .iter()
        .map(|item| format!("{},", item.value))
        .collect();
    assert_native(&format!(
        "unpublished:{}:{capacity}:{count}:{values}:{}",
        name,
        trace.resources.get()
    ));
    // SAFETY: native state is asserted first. This test-only owner reclaims the exact
    // leaked Vec allocation and known live prefix once; production has no recovery owner.
    unsafe { drop(Vec::from_raw_parts(pointer, count, capacity)) };
}
