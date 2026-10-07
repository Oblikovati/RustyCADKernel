use super::{DynamicArray, DynamicArrayCachedIterator, retained_integers};
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};
fn rejected_emplace(array: &mut DynamicArray<i32>, index: usize) {
    let invoked = Cell::new(false);
    let size = array.size();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array
                .emplace_value(index, || {
                    invoked.set(true);
                    7
                })
                .unwrap();
        }))
        .is_err()
    );
    assert!(!invoked.get());
    assert_eq!(array.size(), size);
}
#[test]
fn absent_freed_and_later_zero_slots_reject_before_callbacks() {
    let mut missing = DynamicArray::with_increment(usize::MAX);
    rejected_emplace(&mut missing, 0);
    let mut array = retained_integers();
    rejected_emplace(&mut array, 1);
    array.append(7);
    rejected_emplace(&mut array, 1);
    array.clear(true);
    rejected_emplace(&mut array, 0);
    array.set_increment(2);
    array.emplace_value(0_usize, || 8).unwrap();
    assert_eq!(array.first().unwrap(), &8);
}
struct NonClone(i32);
impl Default for NonClone {
    fn default() -> Self {
        panic!("default construction: expected an existing registered slot");
    }
}
#[test]
fn registered_zero_slot_indexing_never_calls_default_or_requires_clone() {
    let mut array = DynamicArray::with_increment(2);
    array.append(NonClone(1));
    array.clear(false);
    array.set_increment(usize::MAX);
    assert_uninitialized_index_rejected(&mut array);
    array.emplace_value(0_i32, || NonClone(7)).unwrap();
    array[0_usize].0 = 8;
    let mut cursor = DynamicArrayCachedIterator::from_mutable(&mut array, 0).unwrap();
    unsafe {
        cursor.change_value().0 = 9;
    }
    assert_eq!(array.first().unwrap().0, 9);
    array.erase_last();
    assert!(cursor.offset_with(&mut array, 0).is_err());
}
#[test]
fn cached_dead_slot_and_wrong_owner_reload_reject_without_changing_cache() {
    let mut array = retained_integers();
    array.append(7);
    let mut cursor = DynamicArrayCachedIterator::from_shared(&array, 0).unwrap();
    let saved = cursor.pointer();
    let other = retained_integers();
    assert!(cursor.offset_with(&other, 0).is_err());
    array.clear(true);
    assert!(cursor.offset_with(&array, 0).is_err());
    assert_eq!(cursor.pointer(), saved);
    assert!(
        DynamicArrayCachedIterator::from_shared(&array, 0)
            .unwrap()
            .pointer()
            .is_null()
    );
}
fn assert_uninitialized_index_rejected(array: &mut DynamicArray<NonClone>) {
    let rejected = catch_unwind(AssertUnwindSafe(|| {
        array[0_usize].0 = 2;
    }))
    .is_err();
    assert!(rejected);
    assert_eq!(array.size(), 0);
}
