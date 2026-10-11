use super::super::*;
#[test]
fn const_cursor_crosses_blocks_and_returns_from_end() {
    let source = source_array();
    let mut cursor = DynamicArrayCachedIterator::from_shared(&source, 0).unwrap();
    for expected in 1..=6 {
        assert_eq!(cached_value(&cursor), expected);
        cursor.advance_with(&source).unwrap();
    }
    assert!(!cursor.more());
    assert!(cursor.pointer().is_null());
    cursor.retreat_with(&source).unwrap();
    assert_eq!(cached_value(&cursor), 6);
    cursor.offset_with(&source, -3).unwrap();
    assert_eq!(cached_value(&cursor), 3);
}
#[test]
fn wrong_owner_and_missing_reload_leave_cached_pointer_unchanged() {
    let mut source = source_array();
    let mut cursor = MutableCursor::from_mutable(&mut source, 1).unwrap();
    let mut target = DynamicArray::from_moved(&mut source);
    let before = cursor.pointer();
    assert!(cursor.advance_with(&mut target).is_err());
    assert_eq!(cursor.pointer(), before);
    assert!(cursor.advance_with(&mut source).is_err());
    assert_eq!(cursor.pointer(), before);
    assert_eq!(cached_value(&cursor), 2);
}
#[test]
fn empty_default_and_dead_slot_reload_are_checked() {
    let cursor = DynamicArrayCachedIterator::<i32>::default();
    assert!(!cursor.more());
    assert!(cursor.pointer().is_null());
    let mut source = source_array();
    let mut live = MutableCursor::from_mutable(&mut source, 0).unwrap();
    source.clear(false);
    assert!(live.offset_with(&mut source, 0).is_err());
    let end = MutableCursor::from_mutable(&mut source, 0).unwrap();
    assert!(!end.more());
    assert!(end.change_pointer().is_null());
}
#[test]
fn cached_cursor_copy_and_signed_overflow_preserve_state() {
    let source = source_array();
    let mut cursor = DynamicArrayCachedIterator::from_shared(&source, 0).unwrap();
    // Exercise the explicit Clone contract even though this cursor is also Copy.
    #[allow(clippy::clone_on_copy)]
    let copied = Clone::clone(&cursor);
    assert_eq!(copied.pointer(), cursor.pointer());
    cursor.offset_with(&source, isize::MAX).unwrap();
    assert!(!cursor.more());
    assert!(cursor.offset_with(&source, 1).is_err());
    assert!(!cursor.more());
}
struct NonCloneSlot(i32);
#[test]
fn mutable_raw_provenance_survives_transfer_without_clone_bounds() {
    let mut source = DynamicArray::with_increment(2);
    source.append(NonCloneSlot(1));
    source.append(NonCloneSlot(2));
    let mut cursor = DynamicArrayCachedIterator::from_mutable(&mut source, 0).unwrap();
    let target = DynamicArray::from_moved(&mut source);
    // The target keeps both slots alive; no element reference overlaps either cached write.
    unsafe {
        (*cursor.change_pointer()).0 = 11;
    }
    cursor.advance_with(&mut source).unwrap();
    unsafe {
        cursor.change_value().0 = 22;
    }
    assert_eq!(target.value(0).unwrap().0, 11);
    assert_eq!(target.value(1).unwrap().0, 22);
}
#[test]
fn shared_pointer_survives_move_and_refill_until_explicit_reload() {
    let mut source = source_array();
    let mut cursor = DynamicArrayCachedIterator::from_shared(&source, 1).unwrap();
    let target = DynamicArray::from_moved(&mut source);
    source.append(101);
    source.append(102);
    assert_eq!(cached_value(&cursor), target[1]);
    cursor.offset_with(&source, 0).unwrap();
    assert_eq!(cached_value(&cursor), 102);
}
#[test]
fn before_begin_increment_rejects_null_pointer_arithmetic() {
    let source = source_array();
    let mut cursor = DynamicArrayCachedIterator::from_shared(&source, 0).unwrap();
    cursor.retreat_with(&source).unwrap();
    assert!(cursor.pointer().is_null());
    assert!(cursor.advance_with(&source).is_err());
    assert!(cursor.pointer().is_null());
}
#[test]
fn mutable_end_retreat_reloads_writable_last_slot() {
    let mut source = source_array();
    let mut cursor = MutableCursor::from_mutable(&mut source, 6).unwrap();
    cursor.retreat_with(&mut source).unwrap();
    // Source keeps its initialized last slot alive and uniquely accessible through this cursor.
    unsafe {
        *cursor.change_value() = 66;
    }
    assert_eq!(source[5], 66);
}
