//! Canonical metadata and cursor observations match pinned OCCT destructor phases.
use rustycad_tkernel::ncollection::{BaseAllocator, HSequence, Sequence, SequenceNativeCursor};
#[path = "sequence_observer_contracts/fixture.rs"]
mod fixture;
#[test]
fn sequence_observer_matches_all_304_native_phase_rows() {
    let actual = fixture::all_rows();
    let expected = include_str!("../../../tools/oracle/tkernel_sequence_observer_phases.expected");
    assert_eq!(actual.len(), 304);
    for (index, (actual, expected)) in actual.iter().zip(expected.lines()).enumerate() {
        assert_eq!(actual, expected, "native Sequence observer row {index}");
    }
    assert_eq!(actual.join("\n") + "\n", expected);
}
fn assert_traits<T: Send + Sync + Unpin + std::panic::RefUnwindSafe>() {}
#[test]
fn sequence_and_hsequence_preserve_public_auto_traits() {
    assert_traits::<Sequence<i32>>();
    assert_traits::<HSequence<i32>>();
    assert_traits::<Sequence<std::marker::PhantomPinned>>();
}
#[test]
fn native_cursor_clone_default_and_reverse_step_keep_independent_state() {
    let sequence: Sequence<i32> = [1, 2].into_iter().collect();
    let mut cursor = SequenceNativeCursor::new(&sequence, false);
    let copy = cursor.clone();
    cursor.previous(&sequence).unwrap();
    assert_eq!(cursor.position().value(&sequence), Some(&2));
    assert!(!copy.observer().more());
    cursor.previous(&sequence).unwrap();
    assert_eq!(cursor.position().value(&sequence), Some(&1));
    cursor.previous(&sequence).unwrap();
    cursor.previous(&sequence).unwrap();
    assert!(!cursor.observer().more());
    let mut empty = SequenceNativeCursor::default();
    empty.next(&sequence).unwrap();
    assert!(!empty.observer().more());
}
#[test]
fn observer_stays_with_owner_during_move_assignment_and_owner_drop() {
    let mut source: Sequence<i32> = [1, 2, 3].into_iter().collect();
    source.value(2).unwrap();
    let source_view = source.observer();
    let mut moved = Sequence::move_from(&mut source);
    let moved_view = moved.observer();
    assert_eq!(source_view.snapshot().size, 0);
    assert_eq!(moved_view.snapshot().cached_index, 2);
    source.move_assign(&mut moved);
    assert_eq!(moved_view.snapshot().size, 0);
    assert_eq!(source_view.snapshot().cached_index, 2);
    drop(source);
    assert_eq!(source_view.snapshot().size, 0);
}
#[test]
fn native_cursor_rejects_foreign_or_removed_links() {
    let mut source: Sequence<i32> = [1, 2].into_iter().collect();
    let mut cursor = SequenceNativeCursor::new(&source, true);
    let mut other: Sequence<i32> = [3].into_iter().collect();
    assert!(other.remove_native(&mut cursor).is_err());
    cursor.next(&source).unwrap();
    source.remove(1).unwrap();
    assert!(cursor.previous(&source).is_err());
    source.remove(1).unwrap();
    assert!(cursor.next(&source).is_err());
    assert!(source.remove_native(&mut cursor).is_err());
}
#[test]
fn native_cursor_resolves_reused_arena_address_to_fresh_pointer() {
    let arena = BaseAllocator::incremental(1024);
    let mut sequence = Sequence::with_allocator(arena.clone());
    sequence.append(1);
    let mut cursor = SequenceNativeCursor::new(&sequence, true);
    sequence.clear(None);
    arena.reset(false).unwrap();
    sequence.append(2);
    if cursor.position().current_node() == sequence.cursor().current_node() {
        assert_eq!(cursor.position().value(&sequence), Some(&2));
        sequence.remove_native(&mut cursor).unwrap();
        assert_eq!(sequence.len(), 0);
    }
}
#[path = "sequence_observer_contracts/destruction.rs"]
mod destruction;
#[test]
fn observer_cache_pair_remains_consistent_during_parallel_const_lookups() {
    let sequence: Sequence<usize> = (1..=8).collect();
    let addresses = (1..=8)
        .map(|index| {
            sequence.value(index).unwrap();
            sequence.observer().snapshot().cached_node
        })
        .collect::<Vec<_>>();
    std::thread::scope(|scope| {
        for index in 1..=8 {
            let (sequence, addresses) = (&sequence, &addresses);
            scope.spawn(move || {
                check_parallel_cache_pair(sequence, addresses, index);
            });
        }
    });
}

fn check_parallel_cache_pair(
    sequence: &Sequence<usize>,
    addresses: &[Option<usize>],
    index: usize,
) {
    for _ in 0..8 {
        sequence.value(index).unwrap();
        let observed = sequence.observer().snapshot();
        assert_eq!(observed.cached_node, addresses[observed.cached_index - 1]);
    }
}
