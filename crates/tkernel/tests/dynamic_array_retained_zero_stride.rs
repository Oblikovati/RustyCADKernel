//! Defined first-slot accesses survive overflowing stride when physical blocks are retained.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::DynamicArray;
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
mod dynamic_array_retained_zero_stride {
    pub mod contracts;
    pub mod native_rows;
}
struct RetainedPayload<'a> {
    value: i32,
    destroyed: &'a Cell<usize>,
}
impl Drop for RetainedPayload<'_> {
    fn drop(&mut self) {
        self.destroyed.set(self.destroyed.get() + 1);
    }
}
fn retained_payload_array(destroyed: &Cell<usize>) -> DynamicArray<RetainedPayload<'_>> {
    let mut array = DynamicArray::with_increment(2);
    for value in 1..=3 {
        array.emplace_append(|| RetainedPayload { value, destroyed });
    }
    array.clear(false);
    destroyed.set(0);
    array.set_increment(usize::MAX);
    array
}
#[test]
fn first_retained_zero_stride_slot_can_be_constructed_and_erased() {
    let destroyed = Cell::new(0);
    let mut array = retained_payload_array(&destroyed);
    array.emplace_append(|| RetainedPayload {
        value: 11,
        destroyed: &destroyed,
    });
    assert_eq!(array.value(0_usize).unwrap().value, 11);
    array.change_value(0_usize).unwrap().value = 12;
    assert_eq!(array.first().unwrap().value, 12);
    assert_eq!(array.last().unwrap().value, 12);
    array.erase_last();
    assert!(array.is_empty());
    assert_eq!(destroyed.get(), 1);
}

#[test]
fn retained_zero_stride_native_lifecycle_rows_match() {
    assert_eq!(
        dynamic_array_retained_zero_stride::native_rows::trace(),
        include_str!("../../../tools/oracle/tkernel_dynamic_retained_zero_stride.expected")
    );
}
fn retained_integers() -> DynamicArray<i32> {
    let mut array = DynamicArray::with_increment(2);
    for value in 1..=3 {
        array.append(value);
    }
    array.clear(false);
    array.set_increment(usize::MAX);
    array
}
fn block_count<T>(array: &mut DynamicArray<T>) -> usize {
    array.storage_extension().block_count()
}

fn append_payload<'a>(
    array: &mut DynamicArray<RetainedPayload<'a>>,
    value: i32,
    destroyed: &'a Cell<usize>,
) -> *mut RetainedPayload<'a> {
    std::ptr::from_mut(array.emplace_append(|| RetainedPayload { value, destroyed }))
}
