//! Native BaseList iterator state, conversion, editing, and safe detached access.
use rustycad_tkernel::ncollection::{BaseListIterator, List, ListCursor};

#[test]
fn iterator_conversion_preserves_current_and_previous_links() {
    let list: List<i32> = [1, 2].into_iter().collect();
    let first = list.base_iterator::<true>();
    let mut cursor = first;
    let old = cursor.post_advance(&list);
    let readonly = cursor.to_const();
    assert_eq!(old.cursor().previous_node(), None);
    assert_eq!(
        cursor.cursor().previous_node(),
        first.cursor().current_node()
    );
    assert_eq!(readonly.cursor(), cursor.cursor());
    assert_eq!(
        readonly.cursor().previous_node(),
        cursor.cursor().previous_node()
    );
    assert_eq!(readonly.value(&list), Some(&2));
    assert!(cursor == readonly);
    let restored = BaseListIterator::<i32, true>::from_cursor(readonly.cursor());
    assert!(restored.is_equal(&readonly));
}
#[test]
fn end_equality_ignores_previous_but_increment_preserves_it() {
    let list: List<i32> = [1, 2].into_iter().collect();
    let mut cursor = list.base_iterator::<false>();
    cursor.advance(&list).advance(&list);
    let previous = cursor.cursor().previous_node();
    assert!(previous.is_some());
    assert!(cursor == BaseListIterator::<i32>::default());
    assert_eq!(cursor.cursor(), ListCursor::default());
    cursor.next(&list);
    assert!(!cursor.more());
    assert_eq!(cursor.cursor().previous_node(), previous);
    assert_eq!(cursor.value(&list), None);
    assert_eq!(cursor.value_ptr(&list), None);
}
fn check_returning_accessors(cursor: &mut BaseListIterator<i32, true>) {
    let address = std::ptr::from_ref(cursor);
    assert_eq!(std::ptr::from_ref(cursor.iterator()), address);
    let editable = std::ptr::from_mut(cursor.change_iterator()).cast_const();
    assert_eq!(editable, address);
    let assigned_self = std::ptr::from_mut(cursor.assign_self()).cast_const();
    assert_eq!(assigned_self, address);
    let assigned = cursor.assign(&BaseListIterator::default());
    assert_eq!(std::ptr::from_mut(assigned).cast_const(), address);
    assert!(!cursor.more());
}
#[test]
fn assignment_and_native_self_accessors_return_destination() {
    let list: List<i32> = [1, 2].into_iter().collect();
    let mut cursor = list.base_iterator::<true>();
    check_returning_accessors(&mut cursor);
    cursor.init(&list);
    assert_eq!(cursor.value(&list), Some(&1));
    cursor.next(&list);
    cursor.initialize(&list);
    assert_eq!(cursor.cursor().previous_node(), None);
    let mut readonly = BaseListIterator::default();
    let destination = std::ptr::from_mut(&mut readonly);
    assert_eq!(
        std::ptr::from_mut(readonly.assign_mutable(&cursor)),
        destination
    );
    assert!(readonly == cursor);
}
#[test]
fn inserted_previous_node_survives_remove_and_append() {
    let mut list: List<i32> = [1, 2].into_iter().collect();
    let mut cursor = list.base_iterator::<true>();
    cursor.next(&list);
    assert_eq!(
        *list.insert_before_at(3, cursor.change_cursor()).unwrap(),
        3
    );
    let before = cursor.cursor().previous_node();
    list.remove(cursor.change_cursor()).unwrap();
    assert!(!cursor.more());
    assert_eq!(cursor.cursor().previous_node(), before);
    assert_eq!(list.last(), Some(&3));
    list.append_at(4, cursor.change_cursor());
    assert_eq!(cursor.value(&list), Some(&4));
    assert_eq!(cursor.cursor().previous_node(), before);
    let mut empty = List::default();
    empty.append_at(5, cursor.change_cursor());
    assert_eq!(cursor.value(&empty), Some(&5));
    assert_eq!(cursor.cursor().previous_node(), None);
}
#[test]
fn standard_iterators_preserve_previous_in_legacy_conversion() {
    let mut list: List<i32> = [1, 2].into_iter().collect();
    let head = list.cursor().current_node();
    let mut iterator = list.iter();
    iterator.next();
    let cursor = ListCursor::from_iterator(&iterator);
    assert_eq!(cursor.previous_node(), head);
    assert_eq!(cursor.value(&list), Some(&2));
    let mut iterator = list.iter_mut();
    iterator.next();
    let cursor = ListCursor::from_mut_iterator(&iterator);
    assert_eq!(cursor.previous_node(), head);
    assert_eq!(cursor.value(&list), Some(&2));
}
#[test]
fn mutable_access_requires_owner_and_tracks_transferred_node() {
    let mut list: List<i32> = [1].into_iter().collect();
    let cursor = list.base_iterator::<true>();
    let pointer = cursor.value_ptr(&list).unwrap();
    assert_eq!(
        cursor.change_value_ptr(&mut list).unwrap().cast_const(),
        pointer
    );
    *cursor.change_value(&mut list).unwrap() = 7;
    let mut moved = List::default();
    moved.append_list(&mut list);
    assert_eq!(cursor.value(&list), None);
    assert_eq!(cursor.value(&moved), Some(&7));
    moved.clear();
    assert!(cursor.change_value(&mut moved).is_none());
    assert!(cursor.change_value_ptr(&mut moved).is_none());
}
#[test]
fn iterator_copy_does_not_require_copyable_items() {
    struct NonCopy;
    let list: List<NonCopy> = [NonCopy].into_iter().collect();
    let cursor = list.base_iterator::<false>();
    let mut copy = cursor;
    copy.advance(&list);
    assert!(cursor.more());
    assert!(!copy.more());
    assert!(copy == BaseListIterator::<NonCopy>::default());
}
#[test]
fn mutable_emplacement_updates_previous_without_moving_current() {
    let mut list: List<i32> = [1].into_iter().collect();
    let mut cursor = list.cursor();
    let original = cursor.current_node();
    list.emplace_before_at(&mut cursor, || 2).unwrap();
    assert_eq!(cursor.current_node(), original);
    assert_eq!(cursor.previous_node(), list.cursor().current_node());
    list.remove(&mut cursor).unwrap();
    assert!(!cursor.more());
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [2]);
}
