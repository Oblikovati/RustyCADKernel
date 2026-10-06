//! Typed List uses its actual native base, link nodes and cursor transitions.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    BaseListBasicIterator, List, ListIter, ListIterMut, ListValueNode,
};
#[path = "list_topology_contracts/transitions.rs"]
mod transitions;
#[test]
fn canonical_list_topology_matches_1090_native_transition_rows() {
    let rows = transitions::all_rows();
    let expected = include_str!("../../../tools/oracle/tkernel_list_topology_transitions.expected");
    assert_eq!(rows.len(), 1090);
    for (index, (actual, expected)) in rows.iter().zip(expected.lines()).enumerate() {
        assert_eq!(actual, expected, "native List transition row {index}");
    }
    assert_eq!(rows.join("\n") + "\n", expected);
}
fn require_send_sync<T: Send + Sync>() {}
fn require_unpin<T: Unpin>() {}
fn require_unwind<T: std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
#[test]
fn list_and_iterators_preserve_public_auto_traits() {
    require_send_sync::<List<i32>>();
    require_send_sync::<ListIter<'_, i32>>();
    require_send_sync::<ListIterMut<'_, i32>>();
    require_unpin::<List<std::marker::PhantomPinned>>();
    require_unwind::<List<i32>>();
}
#[test]
fn raw_generic_iterator_reads_actual_typed_node_identity_and_value() {
    let list: List<i32> = [1, 2, 3].into_iter().collect();
    let base = list.base_storage();
    let mut raw = BaseListBasicIterator::<ListValueNode<i32>>::new(base);
    assert_eq!(raw.raw.current.cast_const(), base.first());
    assert_eq!(list.cursor().current_node(), Some(base.first() as usize));
    let mut values = Vec::new();
    while raw.more() {
        values.push(unsafe { *raw.value() });
        unsafe { raw.next() };
    }
    assert_eq!(values, [1, 2, 3]);
    assert_eq!(raw.raw.previous.cast_const(), base.last());
}
#[test]
fn mutable_link_iteration_keeps_all_yielded_items_disjoint() {
    let mut list: List<i32> = [1, 2, 3].into_iter().collect();
    let mut iterator = list.iter_mut();
    let first = iterator.next().unwrap();
    let second = iterator.next().unwrap();
    let third = iterator.next().unwrap();
    assert!(iterator.next().is_none());
    *first += 10;
    *second += 20;
    *third += 30;
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [11, 22, 33]);
}

#[path = "list_topology_contracts/destruction.rs"]
mod destruction;

#[test]
fn checked_removal_preserves_member_error_and_detached_previous_state() {
    let mut list: List<i32> = List::default();
    assert!(
        list.remove_first()
            .unwrap_err()
            .message()
            .contains("RemoveFirst")
    );
    list.append(1);
    let mut cursor = list.cursor();
    list.prepend(9);
    list.remove(&mut cursor).unwrap();
    assert!(!cursor.more());
    assert_eq!(cursor.previous_node(), None);
    assert_eq!(list.first(), Some(&9));
    assert_eq!(list.base_storage().size(), 1);
}
