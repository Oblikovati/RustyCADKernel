//! Canonical typed BaseSequence topology, shared lookup cache and exclusive item borrowing.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseSequenceBasicIterator, HSequence, Sequence, SequenceBaseAccess,
    SequenceIter, SequenceIterMut, SequenceNode,
};
use std::sync::Arc;
#[path = "sequence_topology_contracts/destruction.rs"]
mod destruction;
#[path = "sequence_topology_contracts/transitions.rs"]
mod transitions;

#[test]
fn typed_sequence_cache_and_topology_match_1520_native_transitions() {
    let actual = transitions::all_rows();
    let expected =
        include_str!("../../../tools/oracle/tkernel_sequence_topology_transitions.expected");
    assert_eq!(actual.len(), 1520);
    for (index, (actual, expected)) in actual.iter().zip(expected.lines()).enumerate() {
        assert_eq!(actual, expected, "native transition row {index}");
    }
    assert_eq!(actual.join("\n") + "\n", expected);
}
fn require_send<T: Send>() {}
fn require_sync<T: Sync>() {}
#[test]
fn sequence_and_handle_and_borrowed_iterators_preserve_send_sync_bounds() {
    require_send::<Sequence<i32>>();
    require_sync::<Sequence<i32>>();
    require_send::<HSequence<i32>>();
    require_sync::<HSequence<i32>>();
    require_send::<SequenceIter<'_, i32>>();
    require_sync::<SequenceIter<'_, i32>>();
    require_send::<SequenceIterMut<'_, i32>>();
    require_sync::<SequenceIterMut<'_, i32>>();
}
#[test]
fn shared_lookup_serializes_only_cache_writes_and_preserves_allocator_reference() {
    let sequence = Arc::new((1..=8).collect::<Sequence<i32>>());
    let allocator = sequence.allocator();
    std::thread::scope(|scope| {
        for index in 1..=8 {
            let sequence = &sequence;
            scope.spawn(move || {
                for _ in 0..16 {
                    assert_eq!(sequence.value(index).unwrap(), &(index as i32));
                }
            });
        }
    });
    assert_eq!(allocator, &BaseAllocator::common());
    let index = sequence.base_access().current_index();
    assert!((1..=8).contains(&index));
}
#[test]
fn mutable_double_ended_iteration_never_yields_a_node_twice() {
    let mut sequence: Sequence<i32> = [1, 2, 3].into_iter().collect();
    let mut iterator = sequence.iter_mut();
    let first = iterator.next().unwrap();
    let last = iterator.next_back().unwrap();
    let middle = iterator.next_back().unwrap();
    assert!(iterator.next().is_none());
    assert!(iterator.next_back().is_none());
    *first += 10;
    *last += 30;
    *middle += 20;
    assert_eq!(sequence.iter().copied().collect::<Vec<_>>(), [11, 22, 33]);
}
#[test]
fn raw_base_iterator_traverses_the_actual_typed_nodes() {
    let sequence: Sequence<i32> = [1, 2, 3].into_iter().collect();
    let base: SequenceBaseAccess<'_> = sequence.base_access();
    let mut cursor = BaseSequenceBasicIterator::<SequenceNode<i32>>::new(&base, true);
    assert_eq!(cursor.current_node(), base.first_node().cast_const());
    let mut values = Vec::new();
    while cursor.more() {
        values.push(unsafe { *cursor.value() });
        unsafe { cursor.next() };
    }
    assert_eq!(values, [1, 2, 3]);
}

#[test]
fn base_guard_allows_non_cache_structural_queries() {
    let sequence: Sequence<i32> = [1, 2, 3].into_iter().collect();
    let base = sequence.base_access();
    assert_eq!(base.size(), sequence.len());
    assert_eq!(sequence.first(), Some(&1));
    assert_eq!(sequence.last(), Some(&3));
    assert_eq!(sequence.iter().copied().collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(sequence.cursor().value(&sequence), Some(&1));
}
#[test]
fn shared_item_references_survive_native_const_cache_updates() {
    let sequence: Sequence<i32> = [1, 2, 3].into_iter().collect();
    let first = sequence.value(1).unwrap();
    let second = sequence.value(2).unwrap();
    let third = sequence.value(3).unwrap();
    assert_eq!((*first, *second, *third), (1, 2, 3));
    assert_eq!(sequence.base_access().current_index(), 3);
}

fn require_unpin<T: Unpin>() {}
fn require_ref_unwind_safe<T: std::panic::RefUnwindSafe>() {}
#[test]
fn boxed_item_stability_and_readonly_unwind_traits_are_preserved() {
    require_unpin::<Sequence<std::marker::PhantomPinned>>();
    require_ref_unwind_safe::<Sequence<i32>>();
    let sequence: Sequence<i32> = [1, 2].into_iter().collect();
    let failed = std::panic::catch_unwind(|| {
        let _guard = sequence.base_access();
        panic!("injected reader panic");
    });
    assert!(failed.is_err());
    assert_eq!(sequence.value(2).unwrap(), &2);
    assert_eq!(sequence.base_access().current_index(), 2);
}
