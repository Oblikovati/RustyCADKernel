use super::super::copy_construction::tests::fixture;
use super::{Sequence, native_sequence_copy_offset};
use crate::ncollection::BaseAllocator;
use fixture::{CopyItem, CopyTrace, events, live_allocations, source_state};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[path = "splice_max_tests/native_rows.rs"]
mod native_rows;

#[test]
fn max_splice_native_rows_preserve_allocator_copy_and_failure_order() {
    assert_eq!(
        native_rows::all_rows(),
        include_str!("../../../../../tools/oracle/tkernel_sequence_splice_max.expected")
    );
}
#[test]
fn copy_offset_normalizes_only_the_defined_distinct_allocator_wrap() {
    for index in [0, 1, 2, usize::MAX - 1, usize::MAX] {
        assert_eq!(native_sequence_copy_offset(index, true), index);
        let expected = if index == usize::MAX { 0 } else { index };
        assert_eq!(native_sequence_copy_offset(index, false), expected);
    }
}
#[test]
fn undefined_native_oversized_indices_remain_checked_without_copy() {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let trace = make_trace(&arena, 1);
    let mut source = fixture::make_source(&trace, 1);
    let mut target = Sequence::default();
    for index in [1, usize::MAX - 1] {
        assert!(target.insert_sequence_after(index, &mut source).is_err());
        assert!(trace.events.borrow().is_empty());
        assert_eq!(source.len(), 1);
        assert!(target.is_empty());
    }
}

fn make_trace(arena: &BaseAllocator, failure: i32) -> CopyTrace<'_> {
    CopyTrace {
        arena,
        events: Default::default(),
        failing: Cell::new(failure),
    }
}
