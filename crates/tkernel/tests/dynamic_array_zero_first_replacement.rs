//! Retained first-slot replacement and cached pointer capture preserve native zero-stride phases.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{DynamicArray, DynamicArrayCachedIterator};
fn retained_integers() -> DynamicArray<i32> {
    let mut array = DynamicArray::with_increment(2);
    for value in 1..=3 {
        array.append(value);
    }
    array.clear(false);
    array.set_increment(usize::MAX);
    array
}
#[test]
fn zero_stride_emplacement_publishes_only_the_requested_first_slot() {
    let mut array = retained_integers();
    array.emplace_value(0_usize, || 7).unwrap();
    assert_eq!(array.size(), 1);
    assert_eq!(array.first().unwrap(), &7);
}
#[test]
fn zero_stride_existing_first_slot_uses_explicit_assignment_policy() {
    let mut array = retained_integers();
    array.append(7);
    array
        .emplace_value_with_assignment(0_i32, || 8, |slot, donor| *slot = *donor)
        .unwrap();
    assert_eq!(array.size(), 1);
    assert_eq!(array.first().unwrap(), &8);
}
#[test]
fn zero_stride_index_mutation_uses_the_registered_first_slot() {
    let mut array = retained_integers();
    array.append(7);
    array[0_usize] = 8;
    assert_eq!(array.first().unwrap(), &8);
}
#[test]
fn zero_stride_cached_capture_retains_the_actual_first_pointer() {
    let mut array = retained_integers();
    array.append(7);
    let cursor = DynamicArrayCachedIterator::from_shared(&array, 0).unwrap();
    assert_eq!(cursor.pointer(), std::ptr::from_ref(array.first().unwrap()));
    assert!(cursor.more());
}
#[path = "dynamic_array_zero_first_replacement/cached.rs"]
mod cached;
#[path = "dynamic_array_zero_first_replacement/contracts.rs"]
mod contracts;
#[path = "dynamic_array_zero_first_replacement/payload.rs"]
mod payload;
#[path = "dynamic_array_zero_first_replacement/phases.rs"]
mod phases;
#[test]
fn zero_stride_replacement_and_cached_trace_matches_pinned_native() {
    let mut rows = phases::emplace_rows();
    rows.extend(phases::borrowed_rows());
    rows.extend(phases::moved_rows());
    rows.extend(phases::internal_rows());
    rows.extend(cached::cached_rows());
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../../tools/oracle/tkernel_dynamic_zero_first_replacement.expected")
    );
}
