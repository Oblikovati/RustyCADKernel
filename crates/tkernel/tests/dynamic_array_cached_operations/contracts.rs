use super::super::*;
struct UniquePayload(i32);
#[test]
fn non_clone_payloads_support_both_specializations_and_subscript_borrows() {
    let mut source = DynamicArray::with_increment(2);
    source.append(UniquePayload(7));
    source.append(UniquePayload(9));
    let mut mutable = DynamicArrayCachedIterator::from_mutable(&mut source, 0).unwrap();
    let previous = mutable.postfix_advance_with(&mut source).unwrap();
    *previous.subscript_with(&mut source, 0).unwrap() = UniquePayload(17);
    let constant = mutable.as_const_cursor();
    assert_eq!(constant.subscript_with(&source, 0).unwrap().0, 9);
    assert_eq!(source.value(0).unwrap().0, 17);
    assert_eq!(constant.positional_difference(&previous).unwrap(), 1);
}
#[test]
fn checked_arithmetic_rejects_negation_and_difference_overflow() {
    let source = source_array();
    let mut cursor = DynamicArrayCachedIterator::from_shared(&source, 0).unwrap();
    let pointer = cursor.pointer();
    assert!(cursor.subtract_with(&source, isize::MIN).is_err());
    assert!(cursor.retreated_by_with(&source, isize::MIN).is_err());
    assert_eq!(cursor.pointer(), pointer);
    let positive = DynamicArrayCachedIterator::from_shared(&source, isize::MAX as usize).unwrap();
    let negative = DynamicArrayCachedIterator::from_shared(&source, usize::MAX).unwrap();
    assert!(positive.positional_difference(&negative).is_err());
    assert_eq!(negative.positional_difference(&cursor).unwrap(), -1);
    assert!(positive.advanced_by_with(&source, 1).is_err());
}
#[test]
fn failed_postfix_reload_preserves_the_transferred_cache() {
    let mut source = source_array();
    let mut cursor = DynamicArrayCachedIterator::from_mutable(&mut source, 1).unwrap();
    let target = DynamicArray::from_moved(&mut source);
    let pointer = cursor.pointer();
    assert!(cursor.postfix_advance_with(&mut source).is_err());
    assert_eq!(cursor.pointer(), pointer);
    assert_eq!(cached_value(&cursor), 2);
    assert!(cursor.advanced_by_with(&mut source, 0).is_err());
    assert!(cursor.subscript_with(&mut source, 0).is_err());
    assert_eq!(target.value(1).unwrap(), &2);
}
#[test]
fn invalid_subscripts_and_wrong_owners_fail_without_pointer_access() {
    let mut source = source_array();
    let mut other = source_array();
    let mut mutable = DynamicArrayCachedIterator::from_mutable(&mut source, 0).unwrap();
    let mut constant = mutable.as_const_cursor();
    assert!(constant.subscript_with(&source, -1).is_err());
    assert!(mutable.subscript_with(&mut source, 6).is_err());
    assert!(constant.postfix_advance_with(&other).is_err());
    assert!(constant.postfix_retreat_with(&other).is_err());
    assert!(mutable.postfix_advance_with(&mut other).is_err());
    assert!(mutable.postfix_retreat_with(&mut other).is_err());
    assert!(mutable.subscript_with(&mut other, 0).is_err());
    assert!(constant.subscript_with(&other, 0).is_err());
    assert!(constant.same_position(&mutable));
}
#[test]
fn empty_end_postfix_and_nonzero_displacement_remain_dereference_free() {
    let mut source = DynamicArray::<UniquePayload>::with_increment(2);
    let mut mutable = DynamicArrayCachedIterator::from_mutable(&mut source, 0).unwrap();
    let previous = mutable.postfix_retreat_with(&mut source).unwrap();
    assert!(!previous.more());
    assert!(!mutable.more());
    assert!(!mutable.retreated_by_with(&mut source, -2).unwrap().more());
    let mut constant = mutable.as_const_cursor();
    assert!(!constant.postfix_retreat_with(&source).unwrap().more());
    assert!(constant.subscript_with(&source, 0).is_err());
}
