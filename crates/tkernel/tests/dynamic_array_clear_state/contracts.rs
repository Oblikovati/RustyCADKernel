use super::payload::*;
use rustycad_tkernel::ncollection::DynamicArray;
use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};
#[derive(Debug, PartialEq)]
struct RepairPanic;
#[test]
fn repair_panic_preserves_progress_and_both_original_payloads() {
    let (mut state, probe) = fixture();
    probe.borrow_mut().fail = 3;
    let original = catch_unwind(AssertUnwindSafe(|| clear(&mut state, &probe, false))).unwrap_err();
    let repair_panic = catch_unwind(AssertUnwindSafe(|| {
        state.repair_destroyed_with(|index| repair_item(index, &probe))
    }))
    .unwrap_err();
    assert_eq!(original.downcast_ref::<ClearPanic>(), Some(&ClearPanic(3)));
    assert_eq!(
        repair_panic.downcast_ref::<RepairPanic>(),
        Some(&RepairPanic)
    );
    assert_eq!(state.dead_slots(), 1..3);
    assert_eq!(state.live_value(0).unwrap().value, 1);
    finish_repair(state, &probe);
}
#[test]
fn noncanonical_prefix_is_returned_unchanged_before_transfer() {
    let mut array = DynamicArray::with_increment(2);
    array.append(1);
    array[1_usize] = 2;
    let (array, failure) = array
        .into_clear_state()
        .err()
        .expect("unpublished initialized tail");
    assert_eq!(array.size(), 1);
    assert_eq!(array[1_usize], 2);
    assert!(failure.to_string().contains("coherent registered prefixes"));
}
#[test]
fn original_live_array_can_be_extracted_without_destruction() {
    let (state, probe) = fixture();
    assert_eq!(state.block_count(), 3);
    let array = state
        .try_into_array()
        .unwrap_or_else(|_| panic!("no holes expected"));
    assert_eq!(array.size(), 5);
    assert!(probe.borrow().ended.is_empty());
    assert_eq!(array.first().unwrap() as *const _, probe.borrow().slots[0]);
}
#[test]
fn empty_and_zero_stride_states_skip_destruction_callbacks() {
    let empty = DynamicArray::<i32>::with_increment(usize::MAX);
    let mut state = empty
        .into_clear_state()
        .unwrap_or_else(|(_, e)| panic!("{e}"));
    unsafe {
        state.clear_with_destruction(true, |_, _| panic!("empty callback"));
    }
    assert_eq!(state.block_count(), 0);
}
#[test]
fn retained_zero_stride_omission_allows_explicit_stride_reset() {
    let mut array = DynamicArray::with_increment(2);
    array.append(1);
    array.clear(false);
    array.set_increment(usize::MAX);
    array.append(7);
    let mut state = array
        .into_clear_state()
        .unwrap_or_else(|(_, e)| panic!("{e}"));
    unsafe {
        state.clear_with_destruction(false, |_, _| panic!("zero stride callback"));
    }
    let mut array = state
        .try_into_array()
        .unwrap_or_else(|_| panic!("completed zero stride"));
    assert_eq!(array.size(), 0);
    array.set_increment(2);
    array.append(8);
    assert_eq!(array.first().unwrap(), &8);
}
fn repair_item(index: usize, probe: &SharedProbe) -> ClearItem {
    if index == 1 {
        panic_any(RepairPanic);
    }
    item(index as i32 + 1, probe)
}
fn finish_repair(
    state: rustycad_tkernel::ncollection::DynamicArrayClearState<ClearItem>,
    probe: &SharedProbe,
) {
    let mut state = state
        .try_into_array()
        .err()
        .expect("unrepaired holes must stay contained");
    repair(&mut state, probe);
    let array = state
        .try_into_array()
        .unwrap_or_else(|_| panic!("repair expected complete"));
    assert_eq!(array.size(), 5);
}
