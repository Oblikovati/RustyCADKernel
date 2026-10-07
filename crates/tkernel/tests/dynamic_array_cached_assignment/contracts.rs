use super::super::*;
struct UniquePayload(i32);
#[test]
fn assignment_and_initializer_preserve_non_clone_payload_support() {
    let mut source = DynamicArray::with_increment(2);
    source.append(UniquePayload(7));
    let saved = DynamicArrayCachedIterator::from_mutable(&mut source, 0).unwrap();
    let mut assigned = DynamicArrayCachedIterator::<UniquePayload, true>::default();
    assigned.assign_from(&saved);
    // Source retains the slot and no competing reference exists during this write.
    unsafe {
        assigned.change_value().0 = 9;
    }
    let mut shared = saved.as_const_cursor();
    shared.assign_self();
    assert!(shared == assigned);
    shared.initialize_with(&source).unwrap();
    assert_eq!(source.value(0).unwrap().0, 9);
    assert!(shared.at_least(&assigned).unwrap());
}
#[test]
fn assignment_copies_end_snapshot_and_default_clears_cached_state() {
    let source = source_array();
    let saved = SharedCursor::from_shared(&source, 2).unwrap();
    let mut assigned = SharedCursor::default();
    assigned.assign_from(&saved);
    assert_eq!(assigned.pointer(), saved.pointer());
    assert!(assigned == saved);
    assigned.assign_from(&SharedCursor::default());
    assert!(!assigned.more());
    assert!(assigned == SharedCursor::default());
}
#[test]
fn failed_initialization_preserves_both_specialized_previous_owners() {
    let mut empty = DynamicArray::<()>::with_increment(2);
    let mut unsupported = DynamicArray::with_increment(2);
    unsupported.append(());
    let mut shared = DynamicArrayCachedIterator::from_shared(&empty, 0).unwrap();
    let before = shared;
    assert!(shared.initialize_with(&unsupported).is_err());
    assert!(shared == before);
    let mut mutable = DynamicArrayCachedIterator::from_mutable(&mut empty, 0).unwrap();
    let before = mutable;
    assert!(mutable.initialize_with(&mut unsupported).is_err());
    assert!(mutable == before);
    assert!(mutable.pointer().is_null());
}
#[test]
fn initialization_switches_required_navigation_owner() {
    let mut source = source_array();
    let mut other = source_array();
    let mut cursor = MutableCursor::from_mutable(&mut source, 0).unwrap();
    cursor.initialize_with(&mut other).unwrap();
    assert!(cursor.advance_with(&mut source).is_err());
    cursor.advance_with(&mut other).unwrap();
    assert_eq!(cached_value(&cursor), 2);
    let mut constant = cursor.as_const_cursor();
    constant.initialize_with(&source).unwrap();
    assert!(constant.advance_with(&other).is_err());
    constant.advance_with(&source).unwrap();
    assert_eq!(cached_value(&constant), 2);
}
#[test]
fn native_comparison_direction_controls_signed_minimum_overflow() {
    let source = source_array();
    let zero = SharedCursor::from_shared(&source, 0).unwrap();
    let negative = SharedCursor::from_shared(&source, isize::MIN as usize).unwrap();
    assert!(negative.less_than(&zero).unwrap());
    assert!(!negative.at_least(&zero).unwrap());
    assert!(zero.greater_than(&negative).unwrap());
    assert!(!zero.at_most(&negative).unwrap());
    assert!(negative.greater_than(&zero).is_err());
    assert!(negative.at_most(&zero).is_err());
    assert!(zero.less_than(&negative).is_err());
    assert!(zero.at_least(&negative).is_err());
}
#[test]
fn assignment_and_comparison_never_dereference_destroyed_slot_pointers() {
    let saved = {
        let source = source_array();
        SharedCursor::from_shared(&source, 1).unwrap()
    };
    let mut assigned = SharedCursor::default();
    assigned.assign_from(&saved);
    assigned.assign_self();
    assert!(assigned == saved);
    assert!(!assigned.less_than(&saved).unwrap());
    assert!(assigned.at_least(&saved).unwrap());
    // More observes the cached address only, so it must not be interpreted as a lifetime check.
    assert!(assigned.more());
}
