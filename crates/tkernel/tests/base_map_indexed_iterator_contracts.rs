//! Shared native BaseMap::IndexedIterator contracts for both indexed map types.
use rustycad_tkernel::ncollection::{BaseMapIndexedIterator, IndexedDataMap, IndexedMap};
use std::cmp::Ordering;
type KeyCursor = BaseMapIndexedIterator<IndexedMap<i32>>;
type ValueCursor = BaseMapIndexedIterator<IndexedDataMap<i32, i32>, true>;

fn key_map() -> IndexedMap<i32> {
    let mut map = IndexedMap::default();
    for key in [10, 20, 30] {
        map.add(key);
    }
    map
}

#[test]
fn detached_default_and_unchecked_index_match_native_states() {
    let map = key_map();
    let default = KeyCursor::default();
    assert_eq!(default.index(), 1);
    assert!(!default.more(&map).unwrap());
    assert!(default.value(&map).is_err());
    let zero = KeyCursor::new(&map, 0);
    assert!(zero.more(&map).unwrap());
    assert!(zero.value(&map).is_err());
    let past = KeyCursor::new(&map, 4);
    assert!(default.is_equal(&map, &past, &map).unwrap());
}

#[test]
fn dereference_key_arrow_and_subscript_keep_one_based_indices() {
    let map = key_map();
    let cursor = KeyCursor::new(&map, 2);
    assert_eq!(cursor.key(&map).unwrap(), &20);
    assert_eq!(cursor.value(&map).unwrap(), &20);
    assert_eq!(
        cursor.value_ptr(&map).unwrap(),
        std::ptr::from_ref(map.find_key(2).unwrap())
    );
    assert_eq!(cursor.value_at(&map, -1).unwrap(), &10);
    assert_eq!(cursor.value_at(&map, 1).unwrap(), &30);
    assert_eq!(cursor.index(), 2);
}

#[test]
fn prefix_postfix_and_native_wrapper_returns_preserve_identity() {
    let map = key_map();
    let mut cursor = KeyCursor::new(&map, 1);
    let identity = std::ptr::from_mut(&mut cursor);
    assert_eq!(std::ptr::from_mut(cursor.advance_returning()), identity);
    assert_eq!(cursor.advance_post().index(), 2);
    assert_eq!(std::ptr::from_mut(cursor.retreat_returning()), identity);
    assert_eq!(cursor.retreat_post().index(), 2);
    assert_eq!(cursor.index(), 1);
    assert_eq!(std::ptr::from_ref(cursor.iterator()), identity.cast_const());
    assert_eq!(std::ptr::from_mut(cursor.change_iterator()), identity);
}

#[test]
fn offset_arithmetic_distance_and_position_comparisons_match_native() {
    let map = key_map();
    let mut cursor = KeyCursor::new(&map, 1);
    let first = cursor;
    assert_eq!(cursor.offset_returning(2).index(), 3);
    assert_eq!(cursor.offset_sub_returning(1).index(), 2);
    assert_eq!(cursor.shifted(1).index(), 3);
    assert_eq!(cursor.shifted_back(1).index(), 1);
    assert_eq!(cursor.distance(&first), 1);
    assert_eq!(first.compare_position(&cursor), Ordering::Less);
    assert_eq!(cursor.compare_position(&first), Ordering::Greater);
    assert_eq!(cursor.compare_position(&cursor), Ordering::Equal);
}

#[test]
fn unsigned_prefix_and_postfix_wrap_at_native_size_t_boundaries() {
    let map = key_map();
    let mut cursor = KeyCursor::new(&map, 0);
    assert_eq!(cursor.retreat_post().index(), 0);
    assert_eq!(cursor.index(), usize::MAX);
    cursor.advance_returning();
    assert_eq!(cursor.index(), 0);
}

#[test]
fn exhausted_equality_is_independent_of_index_and_owner_but_order_is_not() {
    let first = key_map();
    let second = key_map();
    let left = KeyCursor::new(&first, 4);
    let right = KeyCursor::new(&second, 5);
    assert!(left.is_equal(&first, &right, &second).unwrap());
    assert_eq!(left.compare_position(&right), Ordering::Less);
    let left_live = KeyCursor::new(&first, 1);
    let right_live = KeyCursor::new(&second, 1);
    assert!(!left_live.is_equal(&first, &right_live, &second).unwrap());
    assert_eq!(left_live.compare_position(&right_live), Ordering::Equal);
}

#[test]
fn live_size_is_read_again_after_container_growth_and_clear() {
    let mut map = IndexedMap::<i32>::default();
    map.add(10);
    let cursor = KeyCursor::new(&map, 2);
    assert!(!cursor.more(&map).unwrap());
    map.add(20);
    assert!(cursor.more(&map).unwrap());
    assert_eq!(cursor.value(&map).unwrap(), &20);
    map.clear(false);
    assert!(!cursor.more(&map).unwrap());
}

#[test]
fn mutable_values_conversion_and_assignment_share_the_live_container() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    map.add(10, 100);
    map.add(20, 200);
    let cursor = ValueCursor::new(&map, 1);
    *cursor.change_value(&mut map).unwrap() = 101;
    *cursor.change_value_at(&mut map, 1).unwrap() = 202;
    let pointer = cursor.change_value_ptr(&mut map).unwrap();
    assert_eq!(pointer.cast_const(), cursor.value_ptr(&map).unwrap());
    let mut readonly = BaseMapIndexedIterator::<_, false>::default();
    readonly.assign_from_mutable(&cursor);
    assert!(readonly.is_equal(&map, &cursor, &map).unwrap());
    assert_eq!(readonly.key(&map).unwrap(), &10);
    assert_eq!(readonly.value(&map).unwrap(), &101);
    assert_eq!(readonly.value_at(&map, 1).unwrap(), &202);
}

#[test]
fn copy_assignment_and_self_assignment_return_the_destination() {
    let map = key_map();
    let original = KeyCursor::new(&map, 2);
    let mut cursor = KeyCursor::default();
    let identity = std::ptr::from_mut(&mut cursor);
    assert_eq!(std::ptr::from_mut(cursor.assign(&original)), identity);
    assert_eq!(std::ptr::from_mut(cursor.assign_self()), identity);
    assert!(cursor.is_equal(&map, &original, &map).unwrap());
    assert_eq!(cursor.as_const().value(&map).unwrap(), &20);
}

#[test]
fn explicit_owner_validation_rejects_foreign_maps_without_dereferencing_tokens() {
    let map = key_map();
    let other = key_map();
    let cursor = KeyCursor::new(&map, 1);
    let error = cursor.value(&other).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert!(error.message().contains("expected supplied map"));
    assert!(cursor.more(&other).is_err());
    assert!(cursor.key(&other).is_err());
    assert!(cursor.is_equal(&other, &cursor, &map).is_err());
}
