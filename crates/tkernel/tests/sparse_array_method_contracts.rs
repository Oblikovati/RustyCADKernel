//! SparseArray callback order and exception-state contracts observed in pinned OCCT.
#[path = "sparse_array_method_contracts/fixture.rs"]
mod fixture;
use fixture::{SparseTrace, SparseTrackedValue, assert_sparse_oracle_row, tracked_sparse_array};
use rustycad_tkernel::ncollection::{SparseArray, SparseArrayCursor, SparseArrayCursorMut};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

#[test]
fn zero_increment_preserves_defined_empty_operations() {
    let mut array = SparseArray::<i32>::new(0);
    assert_eq!((array.size(), array.extent(), array.capacity()), (0, 0, 0));
    assert!(array.is_empty());
    assert_sparse_oracle_row(&format!(
        "zero_constructor {} {} {}",
        array.size(),
        array.capacity(),
        u8::from(array.is_empty())
    ));
    assert!(!array.cursor().more());
    array.clear();
    let mut other = SparseArray::new(4);
    other.set_value(5, 50);
    array.exchange(&mut other);
    assert_eq!(array.value(5), Some(&50));
    assert_eq!(other.capacity(), 0);
}

#[test]
fn borrowed_insertion_constructs_then_assigns_at_a_stable_address() {
    let trace = Rc::new(RefCell::new(SparseTrace::default()));
    let first = SparseTrackedValue::new(7, &trace);
    let second = SparseTrackedValue::new(9, &trace);
    let mut array = SparseArray::new(4);
    let address = std::ptr::from_mut(array.set_value_ref(1, &first));
    assert_eq!(std::ptr::from_mut(array.bind_ref(1, &second)), address);
    assert_eq!(trace.borrow().events, "C7,A7>9,");
    assert_sparse_oracle_row(&format!(
        "borrowed_set {} 1 {}",
        trace.borrow().events,
        array.size()
    ));
    assert_eq!(array.size(), 1);
}

#[test]
fn assignment_interleaves_copy_and_destruction_by_ascending_block() {
    let trace = Rc::new(RefCell::new(SparseTrace::default()));
    let source = tracked_sparse_array(4, &[(0, 10), (8, 80)], &trace);
    let mut target = tracked_sparse_array(4, &[(0, 1), (4, 4), (8, 8), (12, 12)], &trace);
    let address = std::ptr::from_mut(&mut target);
    assert_eq!(std::ptr::from_mut(target.assign(&source)), address);
    assert_eq!(trace.borrow().events, "A1>10,D4,A8>80,D12,");
    assert_eq!(target.size(), 2);
    assert_sparse_oracle_row(&format!(
        "assign_order {} 1 {}",
        trace.borrow().events,
        target.size()
    ));
}

#[test]
fn failed_assignment_retains_completed_copy_and_untouched_later_slots() {
    let trace = Rc::new(RefCell::new(SparseTrace::default()));
    let source = tracked_sparse_array(4, &[(0, 10), (2, 20)], &trace);
    let mut target = tracked_sparse_array(4, &[(3, 33)], &trace);
    trace.borrow_mut().rejected = Some(20);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            target.assign(&source);
        }))
        .is_err()
    );
    assert_eq!(trace.borrow().events, "C10,C20,");
    assert_eq!(target.size(), 2);
    assert!(target.has_value(0));
    assert!(!target.has_value(2));
    assert!(target.has_value(3));
    assert_sparse_oracle_row(&format!(
        "assign_throw {} 1 {} {} {} {}",
        trace.borrow().events,
        target.size(),
        u8::from(target.has_value(0)),
        u8::from(target.has_value(2)),
        u8::from(target.has_value(3))
    ));
}

#[test]
fn failed_borrowed_set_retains_allocated_block_without_publishing_occupancy() {
    let trace = Rc::new(RefCell::new(SparseTrace::default()));
    let value = SparseTrackedValue::new(44, &trace);
    let mut array = SparseArray::new(4);
    trace.borrow_mut().rejected = Some(44);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.set_value_ref(9, &value);
        }))
        .is_err()
    );
    assert_eq!((array.size(), array.capacity()), (0, 16));
    assert!(!array.has_value(9));
    assert_sparse_oracle_row(&format!(
        "set_throw 1 {} {} {}",
        array.size(),
        array.capacity(),
        u8::from(array.has_value(9))
    ));
    assert_eq!(array.legacy_cursor().index(), 20);
    assert_eq!(array.cursor().index(), 16);
}

#[test]
fn assignment_self_adapter_is_a_noop_returning_this_array() {
    let trace = Rc::new(RefCell::new(SparseTrace::default()));
    let mut array = tracked_sparse_array(4, &[(1, 9)], &trace);
    let address = std::ptr::from_mut(&mut array);
    assert_eq!(std::ptr::from_mut(array.assign_self()), address);
    assert_eq!(trace.borrow().events, "");
    assert_sparse_oracle_row(&format!("assign_self 1 {}", trace.borrow().events.len()));
}

#[test]
fn assignment_with_changed_geometry_destroys_before_copying() {
    let trace = Rc::new(RefCell::new(SparseTrace::default()));
    let source = tracked_sparse_array(2, &[(0, 10)], &trace);
    let mut target = tracked_sparse_array(4, &[(1, 9)], &trace);
    target.assign(&source);
    assert_eq!(trace.borrow().events, "D9,C10,");
    assert_eq!((target.size(), target.capacity()), (1, 4));
}

#[test]
fn legacy_base_cursor_preserves_distinct_exhausted_index_and_restarts() {
    let mut array = SparseArray::new(4);
    array.set_value(5, 50);
    let mut legacy = array.legacy_cursor();
    assert_eq!((legacy.index(), legacy.value()), (5, Some(&50)));
    legacy.next();
    assert!(!legacy.more());
    assert_eq!(legacy.index(), 20);
    assert_eq!(legacy.value(), None);
    assert_sparse_oracle_row(&format!(
        "legacy_end {} {} 0 {} {}",
        u8::from(legacy.more()),
        legacy.index(),
        array.end_cursor().index(),
        array.capacity()
    ));
    legacy.next();
    assert_eq!(legacy.index(), 20);
    legacy.restart();
    assert_eq!(legacy.index(), 5);
    assert_sparse_oracle_row(&format!(
        "legacy_restart {} {}",
        u8::from(legacy.more()),
        legacy.index()
    ));
}

#[test]
fn legacy_cursor_empty_retained_and_reinitialized_positions_match_native() {
    let mut array = SparseArray::new(4);
    array.set_value(5, 50);
    array.unset_value(5);
    let empty = SparseArray::new(8);
    let mut legacy = empty.legacy_cursor();
    legacy.initialize(&array);
    assert!(!legacy.more());
    assert_eq!(legacy.index(), 16);
    legacy.restart();
    assert_eq!(legacy.index(), 16);
    assert_sparse_oracle_row(&format!(
        "legacy_empty_retained {} {}",
        u8::from(legacy.more()),
        legacy.index()
    ));
}

#[test]
fn mutable_prefix_postfix_and_index_alias_preserve_native_positions() {
    let mut array = SparseArray::new(4);
    array.set_value(5, 50);
    let mut cursor = array.cursor_mut();
    let previous = cursor.advance_post();
    assert_eq!((previous.index(), previous.value().unwrap()), (5, &50));
    let previous_key = previous.index();
    let previous_value = *previous.value().unwrap();
    assert_eq!(cursor.index(), 16);
    assert_sparse_oracle_row(&format!(
        "mutable_postfix {previous_key} {} {previous_value}",
        cursor.index()
    ));
    assert_eq!(cursor.advance_returning().index(), 16);
    cursor.restart();
    assert_eq!(cursor.advance_returning().index(), 16);
}

#[test]
fn mutable_assignment_reborrows_position_and_returns_destination() {
    let mut array = SparseArray::new(4);
    array.set_value(5, 50);
    let mut source = array.cursor_mut();
    let mut target = SparseArrayCursorMut::default();
    let address = std::ptr::from_mut(&mut target);
    assert_eq!(std::ptr::from_mut(target.assign_from(&mut source)), address);
    assert_eq!(target.index(), 5);
    *target.change_value().unwrap() = 51;
    assert_eq!(std::ptr::from_mut(target.assign_self()), address);
}

#[test]
fn cursor_equality_supports_cross_const_and_mutable_noncomparable_elements() {
    struct Noncomparable;
    let mut array = SparseArray::new(4);
    array.set_value(5, Noncomparable);
    let mutable = array.cursor_mut();
    let readonly = mutable.as_const_cursor();
    assert!(mutable == readonly);
    assert!(readonly == mutable);
    assert!(mutable == mutable);
    assert!(SparseArrayCursorMut::<Noncomparable>::default() == SparseArrayCursor::default());
}

#[test]
fn const_assignment_from_mutable_matches_native_conversion_row() {
    let mut array = SparseArray::new(4);
    array.set_value(5, 50);
    let mutable = array.cursor_mut();
    let mut readonly = SparseArrayCursor::default();
    let address = std::ptr::from_mut(&mut readonly);
    let returned = std::ptr::from_mut(readonly.assign_from_mutable(&mutable)) == address;
    assert_sparse_oracle_row(&format!(
        "const_conversion {} {} {} {}",
        readonly.key(),
        readonly.index(),
        u8::from(returned),
        u8::from(readonly == mutable)
    ));
}

#[test]
fn shared_block_assignment_orders_slot_creation_assignment_and_removal() {
    let trace = Rc::new(RefCell::new(SparseTrace::default()));
    let source = tracked_sparse_array(4, &[(0, 10), (2, 20)], &trace);
    let mut target = tracked_sparse_array(4, &[(1, 11), (2, 22), (3, 33)], &trace);
    target.assign(&source);
    assert_eq!(trace.borrow().events, "C10,D11,A22>20,D33,");
    assert_eq!(target.size(), 2);
    assert_eq!(
        target.entries().map(|(index, _)| index).collect::<Vec<_>>(),
        [0, 2]
    );
}

#[test]
fn assignment_of_empty_arrays_preserves_target_table_without_growing_from_source() {
    let mut source = SparseArray::new(4);
    source.set_value(100, 10);
    source.unset_value(100);
    let mut target = SparseArray::new(4);
    target.set_value(0, 1);
    target.assign(&source);
    assert_eq!((target.size(), target.capacity()), (0, 16));
    assert!(source.capacity() > target.capacity());
    target.assign(&SparseArray::new(0));
    assert_eq!((target.size(), target.capacity()), (0, 0));
}

#[test]
fn failed_copy_into_new_block_publishes_only_preceding_successful_copies() {
    let trace = Rc::new(RefCell::new(SparseTrace::default()));
    let source = tracked_sparse_array(4, &[(0, 10), (2, 20)], &trace);
    let mut target = SparseArray::new(4);
    trace.borrow_mut().rejected = Some(20);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            target.assign(&source);
        }))
        .is_err()
    );
    assert_eq!((target.size(), target.capacity()), (1, 16));
    assert!(target.has_value(0));
    assert!(!target.has_value(2));
    assert_eq!(trace.borrow().events, "C10,C20,");
}
