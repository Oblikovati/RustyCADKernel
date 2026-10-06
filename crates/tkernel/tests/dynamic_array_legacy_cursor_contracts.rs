//! Native cursor positions capture the end and survive appending across blocks.
use rustycad_tkernel::ncollection::{DynamicArray, DynamicArrayIterator};

fn legacy_cursor_fixture() -> DynamicArray<i32> {
    let mut array = DynamicArray::with_increment(2);
    for value in [10, 20, 30, 40] {
        array.append(value);
    }
    array
}

#[test]
fn legacy_cursor_captures_end_until_reinitialized() {
    let mut array = legacy_cursor_fixture();
    let mut cursor = array.legacy_cursor::<false>();
    for value in 50..80 {
        array.append(value);
    }
    let mut observed = Vec::new();
    while cursor.more() {
        observed.push(*cursor.value(&array).unwrap());
        cursor.next();
    }
    assert_eq!(observed, [10, 20, 30, 40]);
    assert!(cursor.value_ptr(&array).unwrap().is_null());
    assert!(cursor.value(&array).is_err());
    cursor.initialize(&array);
    assert_eq!(cursor.shifted(33).value(&array).unwrap(), &79);
}

#[test]
fn legacy_cursor_mutation_conversion_and_owner_identity() {
    let mut array = legacy_cursor_fixture();
    let cursor = array.legacy_cursor::<true>();
    *cursor.change_value(&mut array).unwrap() = 99;
    let constant = cursor.as_const_cursor();
    assert!(cursor == constant);
    assert_eq!(constant.value(&array).unwrap(), &99);
    assert_eq!(constant.value_ptr(&array).unwrap(), &array[0] as *const i32);
    let other = legacy_cursor_fixture();
    assert!(constant != other.legacy_cursor::<false>());
    assert!(
        constant
            .value(&other)
            .unwrap_err()
            .message()
            .contains("expected cursor owner")
    );
}

#[test]
fn legacy_cursor_prefix_and_postfix_positions_return_native_values() {
    let array = legacy_cursor_fixture();
    let mut cursor = array.legacy_cursor::<false>();
    assert_eq!(cursor.advance_post().value(&array).unwrap(), &10);
    assert_eq!(cursor.advance_returning().value(&array).unwrap(), &30);
    assert_eq!(cursor.retreat_post().value(&array).unwrap(), &30);
    assert_eq!(cursor.retreat_returning().value(&array).unwrap(), &10);
    let mut end = DynamicArrayIterator::<i32>::new(&array, 4);
    assert!(!end.more());
    assert_eq!(end.retreat_returning().value(&array).unwrap(), &40);
    end.offset_returning(20);
    assert_eq!(end.retreat_returning().value(&array).unwrap(), &40);
}

#[test]
fn legacy_cursor_offsets_distance_indexing_and_ordering() {
    let array = legacy_cursor_fixture();
    let begin = array.legacy_cursor::<false>();
    let mut position = begin.shifted(3);
    assert_eq!(position.distance(&begin), 3);
    assert!(position > begin && position >= begin && begin < position && begin <= position);
    assert_eq!(position.value_at(&array, -2).unwrap(), &20);
    assert_eq!(position.shifted_back(2).value(&array).unwrap(), &20);
    assert_eq!(position.offset_sub_returning(2).value(&array).unwrap(), &20);
    assert_eq!(position.offset_returning(-1).value(&array).unwrap(), &10);
    assert_eq!(begin.shifted(-1).distance(&begin), -1);
}

#[test]
fn legacy_cursor_assignment_preserves_captured_end_and_returns_receiver() {
    let mut array = legacy_cursor_fixture();
    let mutable = array.legacy_cursor::<true>();
    let mut constant = DynamicArrayIterator::<i32>::default();
    assert!(!constant.more());
    assert!(constant == DynamicArrayIterator::<i32>::default());
    let receiver = &constant as *const _;
    assert!(std::ptr::eq(
        receiver,
        constant.assign_from_mutable(&mutable)
    ));
    let copied = Clone::clone(&constant);
    array.append(50);
    assert!(!copied.shifted(4).more());
    assert!(std::ptr::eq(receiver, constant.assign_from(&copied)));
    assert!(std::ptr::eq(receiver, constant.assign_self()));
    assert!(constant == mutable);
}

#[test]
fn legacy_cursor_checks_exhausted_mutation_and_empty_retreat() {
    let mut array = DynamicArray::<i32>::default();
    let mut cursor = array.legacy_cursor::<true>();
    assert!(!cursor.retreat_returning().more());
    assert!(cursor.change_value(&mut array).is_err());
    cursor.next();
    assert!(!cursor.more());
    let other = legacy_cursor_fixture();
    assert!(cursor.value_ptr(&other).is_err());
    assert!(cursor.change_value(&mut legacy_cursor_fixture()).is_err());
}

#[test]
fn native_positional_comparison_is_separate_from_rust_owner_ordering() {
    let first = legacy_cursor_fixture();
    let second = legacy_cursor_fixture();
    let left = first.legacy_cursor::<false>();
    let right = second.legacy_cursor::<true>();
    assert!(left != right);
    assert_eq!(left.compare_position(&right), std::cmp::Ordering::Equal);
    assert_eq!(left.partial_cmp(&right), None);
}
