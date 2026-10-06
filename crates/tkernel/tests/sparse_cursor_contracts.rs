//! OCCT sparse cursors retain capacity as the exhausted key and restart on the same array.
use rustycad_tkernel::ncollection::{SparseArray, SparseArrayCursor, SparseArrayCursorMut};

fn sparse_cursor_fixture() -> SparseArray<i32> {
    let mut array = SparseArray::new(4);
    for (key, value) in [(9, 90), (0, 10), (5, 50)] {
        array.set_value(key, value);
    }
    array
}

#[test]
fn default_sparse_cursor_initializes_and_restarts() {
    let array = sparse_cursor_fixture();
    let mut cursor = SparseArrayCursor::<i32>::default();
    assert!(!cursor.more());
    assert_eq!(cursor.key(), 0);
    cursor.next();
    cursor.restart();
    assert_eq!(cursor.key(), 0);
    cursor.initialize(&array);
    assert_eq!((cursor.key(), cursor.value().unwrap()), (0, &10));
    cursor.next();
    assert_eq!(cursor.key(), 5);
    cursor.restart();
    assert_eq!(cursor.key(), 0);
}

#[test]
fn sparse_const_cursor_skips_holes_and_stabilizes_at_capacity() {
    let array = sparse_cursor_fixture();
    let mut cursor = array.cursor();
    let mut observed = Vec::new();
    while cursor.more() {
        observed.push((cursor.key(), *cursor.value().unwrap()));
        cursor.next();
    }
    assert_eq!(observed, [(0, 10), (5, 50), (9, 90)]);
    assert_eq!(cursor.key(), array.capacity());
    cursor.next();
    assert_eq!(cursor.key(), array.capacity());
    let failure = cursor.value().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
    assert!(failure.message().contains("key 16"));
    assert!(failure.message().contains("expected"));
}

#[test]
fn sparse_end_cursor_restarts_to_the_first_occupied_slot() {
    let array = sparse_cursor_fixture();
    let mut cursor = array.end_cursor();
    assert!(!cursor.more());
    assert_eq!(cursor.key(), array.capacity());
    cursor.restart();
    assert_eq!(cursor.key(), 0);
    assert_eq!(cursor.value().unwrap(), &10);
}

#[test]
fn sparse_mutable_cursor_changes_values_and_restarts() {
    let mut array = sparse_cursor_fixture();
    let mut cursor = array.cursor_mut();
    while cursor.more() {
        *cursor.change_value().unwrap() += cursor.key() as i32;
        cursor.next();
    }
    assert!(cursor.change_value().is_err());
    cursor.restart();
    assert_eq!(cursor.value().unwrap(), &10);
    assert_eq!(array.value(5), Some(&55));
    assert_eq!(array.value(9), Some(&99));
}

#[test]
fn mutable_default_and_end_cursors_initialize_without_cloning_values() {
    let mut array = sparse_cursor_fixture();
    let capacity = array.capacity();
    let mut cursor = SparseArrayCursorMut::default();
    assert!(!cursor.more());
    assert_eq!(cursor.key(), 0);
    cursor.next();
    cursor.restart();
    cursor.initialize(&mut array);
    assert_eq!(cursor.value().unwrap(), &10);
    let mut ending = array.end_cursor_mut();
    assert_eq!(ending.key(), capacity);
    assert!(!ending.more());
    ending.restart();
    *ending.change_value().unwrap() = 11;
    assert_eq!(array.value(0), Some(&11));
}

#[test]
fn empty_sparse_cursor_has_no_value_even_with_retained_capacity() {
    let mut array = SparseArray::new(4);
    array.set_value(9, 90);
    array.unset_value(9);
    let mut cursor = array.cursor();
    assert!(!cursor.more());
    assert_eq!(cursor.key(), array.capacity());
    cursor.restart();
    assert_eq!(cursor.key(), array.capacity());
    assert!(cursor.value().is_err());
}

#[test]
fn sparse_cursor_reinitialization_adopts_the_new_array_capacity() {
    let first = sparse_cursor_fixture();
    let mut other = SparseArray::new(2);
    other.set_value(2, 20);
    let mut cursor = first.cursor();
    cursor.next();
    cursor.initialize(&other);
    assert_eq!((cursor.key(), cursor.value().unwrap()), (2, &20));
    cursor.next();
    assert_eq!(cursor.key(), 4);
    assert!(!cursor.more());
}

struct SparseBorrowedValue(i32);

#[test]
fn sparse_cursor_borrows_nonclone_values_across_advancement() {
    let mut array = SparseArray::new(4);
    array.set_value(0, SparseBorrowedValue(10));
    array.set_value(5, SparseBorrowedValue(50));
    let mut cursor = array.cursor();
    let first = cursor.value().unwrap();
    cursor.next();
    assert_eq!(first.0, 10);
    assert_eq!(cursor.value().unwrap().0, 50);
}

#[test]
fn sparse_cursor_copy_preserves_position_without_cloning_elements() {
    let mut array = SparseArray::new(4);
    array.set_value(0, SparseBorrowedValue(10));
    array.set_value(5, SparseBorrowedValue(50));
    let first = array.cursor();
    let mut second = first.clone();
    assert!(std::ptr::eq(
        first.value().unwrap(),
        second.value().unwrap()
    ));
    second.next();
    assert_eq!((first.key(), second.key()), (0, 5));
}

#[test]
fn sparse_cursor_equality_distinguishes_owners_until_exhaustion() {
    let first = sparse_cursor_fixture();
    let second = sparse_cursor_fixture();
    let beginning = first.cursor();
    assert!(beginning == beginning.clone());
    assert!(beginning != second.cursor());
    let mut advanced = beginning.clone();
    advanced.next();
    assert!(beginning != advanced);
    assert!(first.end_cursor() == second.end_cursor());
    assert!(SparseArrayCursor::default() == first.end_cursor());
}

#[test]
fn sparse_cursor_increment_returns_native_old_and_new_positions() {
    let array = sparse_cursor_fixture();
    let mut cursor = array.cursor();
    let old = cursor.advance_post();
    assert_eq!((old.key(), cursor.key()), (0, 5));
    let before = &cursor as *const _;
    let returned = cursor.advance_returning();
    assert!(std::ptr::eq(before, returned));
    assert_eq!(returned.key(), 9);
}

#[test]
fn mutable_sparse_cursor_exposes_a_borrowed_const_snapshot() {
    let mut array = sparse_cursor_fixture();
    let mut cursor = array.cursor_mut();
    let view = cursor.as_const_cursor();
    assert_eq!((view.key(), view.value().unwrap()), (0, &10));
    cursor.next();
    *cursor.change_value().unwrap() = 55;
    let view = cursor.as_const_cursor();
    assert_eq!((view.key(), view.value().unwrap()), (5, &55));
}

#[test]
fn sparse_indexed_cursors_preserve_explicit_past_end_keys() {
    let mut array = sparse_cursor_fixture();
    for (start, expected, more) in [
        (0, 0, true),
        (3, 5, true),
        (6, 9, true),
        (16, 16, false),
        (20, 20, false),
    ] {
        let cursor = array.cursor_from(start);
        assert_eq!((cursor.key(), cursor.more()), (expected, more));
        let cursor = array.cursor_from_mut(start);
        assert_eq!((cursor.key(), cursor.more()), (expected, more));
    }
}

#[test]
fn past_end_sparse_cursor_next_preserves_key_but_restart_finds_first_value() {
    let array = sparse_cursor_fixture();
    let mut cursor = array.cursor_from(20);
    cursor.next();
    assert_eq!(cursor.key(), 20);
    assert!(cursor.value_ptr().is_err());
    cursor.restart();
    assert_eq!(cursor.key(), 0);
}

#[test]
#[allow(unsafe_code)]
fn sparse_cursor_pointers_reference_the_current_live_slot() {
    let mut array = sparse_cursor_fixture();
    let cursor = array.cursor_from(3);
    assert_eq!(
        cursor.value_ptr().unwrap(),
        std::ptr::from_ref(array.value(5).unwrap())
    );
    let mut cursor = array.cursor_from_mut(3);
    let pointer = cursor.value_mut_ptr().unwrap();
    // SAFETY: the cursor owns exclusive access to this live initialized slot; no references exist.
    unsafe { pointer.write(555) };
    assert_eq!(cursor.value().unwrap(), &555);
    cursor.next();
    assert_eq!(cursor.key(), 9);
}

#[test]
fn sparse_cursor_assignment_returns_destination_and_copies_only_position() {
    let array = sparse_cursor_fixture();
    let mut destination = array.cursor();
    let ending = array.end_cursor();
    let before = &destination as *const _;
    let returned = destination.assign_from(&ending);
    assert!(std::ptr::eq(before, returned));
    assert_eq!(returned.key(), array.capacity());
    assert!(!returned.more());
    destination.assign_self();
    destination.clone_from(&array.cursor());
    destination.next();
    assert_eq!(destination.key(), 5);
    assert_eq!(array.cursor().key(), 0);
}

#[test]
fn sparse_cursor_assignment_from_mutable_preserves_the_source_position() {
    let mut array = sparse_cursor_fixture();
    let mut source = array.cursor_mut();
    source.next();
    let mut destination = SparseArrayCursor::default();
    let before = &destination as *const _;
    let returned = destination.assign_from_mutable(&source);
    assert!(std::ptr::eq(before, returned));
    assert_eq!((returned.key(), returned.value().unwrap()), (5, &50));
    assert_eq!(source.key(), 5);
}
