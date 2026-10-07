use super::super::*;
#[test]
fn retained_first_append_still_rejects_missing_storage_before_construction() {
    let mut array = DynamicArray::<i32>::with_increment(usize::MAX);
    let invoked = Cell::new(false);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.emplace_append(|| {
                invoked.set(true);
                5
            });
        }))
        .is_err()
    );
    assert!(!invoked.get());
    assert!(array.is_empty());
}
#[test]
fn later_sparse_zero_stride_slots_are_rejected_before_callback_or_state_change() {
    let mut array = retained_integers();
    array.append(7);
    let pointer = array.storage_extension().block_at(0).unwrap();
    let invoked = Cell::new(false);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.emplace_append(|| {
                invoked.set(true);
                8
            });
        }))
        .is_err()
    );
    assert!(!invoked.get());
    assert_eq!(array.size(), 1);
    assert_eq!(*array.first().unwrap(), 7);
    assert_eq!(array.storage_extension().block_at(0).unwrap(), pointer);
    assert!(array.value(1).is_err());
}
#[test]
fn zero_stride_erasure_callback_observes_original_slot_and_old_size() {
    let destroyed = Cell::new(0);
    let mut array = retained_payload_array(&destroyed);
    append_payload(&mut array, 8, &destroyed);
    let original = array.storage_extension().change_block_at(0).unwrap();
    // The scoped adapter transfers exactly this live slot's destructor obligation to the callback.
    unsafe {
        array
            .erase_last_with_destruction(|slot, size| {
                assert_eq!(slot, original);
                assert_eq!(*size, 1);
                slot.drop_in_place();
            })
            .unwrap();
    }
    assert_eq!(destroyed.get(), 1);
    assert!(array.is_empty());
}
#[test]
fn omitted_clear_does_not_allow_safe_borrowing_of_unowned_payloads() {
    let destroyed = Cell::new(0);
    let mut array = retained_payload_array(&destroyed);
    append_payload(&mut array, 8, &destroyed);
    array.clear(false);
    assert!(array.value(0).is_err());
    assert!(array.change_value(0).is_err());
    assert_eq!(destroyed.get(), 0);
    let pointer = array.storage_extension().change_block_at(0).unwrap();
    // The object remains live after the native omission; repair once before physical deallocation.
    unsafe {
        pointer.drop_in_place();
    }
    assert_eq!(destroyed.get(), 1);
}
#[test]
fn release_then_append_requires_reset_before_any_constructor_callback() {
    let mut array = retained_integers();
    array.append(7);
    array.clear(true);
    let invoked = Cell::new(false);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.emplace_append(|| {
                invoked.set(true);
                8
            });
        }))
        .is_err()
    );
    assert!(!invoked.get());
    assert_eq!(block_count(&mut array), 0);
    array.set_increment(2);
    array.append(9);
    assert_eq!(array.first().unwrap(), &9);
}
#[test]
fn cached_cursor_accepts_only_the_registered_retained_first_slot() {
    let mut array = retained_integers();
    array.append(7);
    assert!(
        rustycad_tkernel::ncollection::DynamicArrayCachedIterator::from_shared(&array, 0).is_ok()
    );
    assert!(
        rustycad_tkernel::ncollection::DynamicArrayCachedIterator::from_mutable(&mut array, 0)
            .is_ok()
    );
}
#[test]
fn zero_stride_erasure_unwind_keeps_destroyed_slot_out_of_automatic_drop() {
    let destroyed = Cell::new(0);
    let mut array = retained_payload_array(&destroyed);
    append_payload(&mut array, 8, &destroyed);
    let panicked = catch_unwind(AssertUnwindSafe(|| erase_and_panic(&mut array))).is_err();
    assert!(panicked);
    assert_eq!(array.size(), 0);
    array.clear(true);
    assert_eq!(destroyed.get(), 1);
}

fn erase_and_panic(array: &mut DynamicArray<RetainedPayload<'_>>) {
    // The callback fulfills its lifetime duty before exercising Rust-only unwind recovery.
    unsafe {
        array
            .erase_last_with_destruction(|slot, _| {
                slot.drop_in_place();
                panic!("completed destruction");
            })
            .unwrap();
    }
}
