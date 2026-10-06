//! Native NCollection_Iterator positions compare pointers, not slice origins and offsets.
use rustycad_tkernel::ncollection::{Array1, CollectionCursor, CollectionCursorMut};

#[test]
fn array_construction_keeps_default_null_and_nonempty_borrowed_origins() {
    let mut empty = Array1::<i32>::default();
    assert!(CollectionCursor::from_array(&empty) == CollectionCursor::default());
    let mut writable = CollectionCursorMut::from_array(&mut empty);
    assert!(writable.value_iter_ptr().is_null());
    assert!(writable.change_end_iter_ptr().is_null());
    let mut array = Array1::<i32>::with_bounds(-4, -3).unwrap();
    let origin = array.data_ptr();
    let readonly = CollectionCursor::from_array(&array);
    assert_eq!(readonly.value_iter_ptr(), origin);
    assert_eq!(readonly.end_iter_ptr(), origin.wrapping_add(2));
    let mut writable = CollectionCursorMut::from_array(&mut array);
    *writable.change_value().unwrap() = 11;
    assert_eq!(writable.value(), Ok(&11));
}

#[test]
fn default_and_copy_need_no_element_default_or_clone_bound() {
    struct Element;
    let first = CollectionCursor::<Element>::default();
    let second = first;
    assert!(first == second);
    assert!(first.value_iter_ptr().is_null());
    assert!(first.end_iter_ptr().is_null());
    let mut writable = CollectionCursorMut::<Element>::default();
    assert!(writable.change_value_iter_ptr().is_null());
    assert!(writable.change_end_iter_ptr().is_null());
    assert!(first == writable);
}

#[test]
fn pointer_pair_equality_matches_overlapping_array_views() {
    let items = [10, 20, 30];
    let cursor = CollectionCursor::with_position(&items, 1);
    let suffix = CollectionCursor::new(&items[1..]);
    assert!(cursor == suffix);
    assert_eq!(cursor.value_iter_ptr(), items.as_ptr().wrapping_add(1));
    assert_eq!(cursor.end_iter_ptr(), items.as_ptr().wrapping_add(3));
    let shorter = CollectionCursor::with_position(&items[..2], 1);
    assert!(cursor != shorter);
    let empty_borrow = CollectionCursor::new(&items[3..]);
    assert!(empty_borrow != CollectionCursor::default());
}

#[test]
fn read_only_assignment_preserves_pointers_and_returns_destination() {
    let items = [10, 20];
    let source = CollectionCursor::with_position(&items, 1);
    let mut target = CollectionCursor::default();
    let identity = std::ptr::from_mut(&mut target);
    assert_eq!(std::ptr::from_mut(target.assign(&source)), identity);
    assert_eq!(std::ptr::from_mut(target.assign_self()), identity);
    assert!(target == source);
    target.initialize(&items);
    assert_eq!(target.value(), Ok(&10));
    assert_eq!(target.value_iter_ptr(), items.as_ptr());
}

#[test]
fn mutable_conversion_assignment_and_initialization_preserve_native_positions() {
    let mut first = [10, 20];
    let mut second = [30, 40, 50];
    let origin = second.as_mut_ptr();
    let mut cursor = CollectionCursorMut::new(&mut first);
    cursor.initialize(&mut second);
    cursor.next();
    let identity = std::ptr::from_mut(&mut cursor);
    assert_eq!(std::ptr::from_mut(cursor.assign_self()), identity);
    assert_eq!(cursor.change_value_iter_ptr(), origin.wrapping_add(1));
    assert_eq!(cursor.change_end_iter_ptr(), origin.wrapping_add(3));
    let mut readonly = CollectionCursor::default();
    readonly.assign_from_mutable(&cursor);
    assert!(readonly == cursor);
    assert_eq!(readonly.value(), Ok(&40));
    assert_eq!(cursor.value_iter_ptr(), origin.wrapping_add(1).cast_const());
    assert_eq!(cursor.end_iter_ptr(), origin.wrapping_add(3).cast_const());
}
