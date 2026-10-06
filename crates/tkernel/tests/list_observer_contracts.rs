//! Live canonical owner/cursor observations at native destructor boundaries.
use rustycad_tkernel::ncollection::{
    List, ListCursor, ListCursorObserver, ListNativeCursor, ListObserver,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
#[path = "list_observer_contracts/fixture.rs"]
mod fixture;
fn compare_rows(rows: &[String], expected: &str) {
    assert_eq!(rows.len(), 23);
    assert_eq!(expected.lines().count(), 23);
    for (index, (actual, expected)) in rows.iter().zip(expected.lines()).enumerate() {
        assert_eq!(
            actual, expected,
            "native destructor observation row {index}"
        );
    }
}
#[test]
fn all_23_native_legacy_and_endpoint_observation_rows_match() {
    let (legacy, endpoints) = fixture::rows();
    compare_rows(
        &legacy,
        include_str!("../../../tools/oracle/tkernel_list_destructor_observer.expected"),
    );
    compare_rows(
        &endpoints,
        include_str!("../../../tools/oracle/tkernel_list_observer_phases.expected"),
    );
}
fn require_send_sync<T: Send + Sync>() {}
fn require_copy<T: Copy>() {}
fn require_unwind<T: std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
#[test]
fn observers_preserve_owner_traits_and_the_existing_copy_cursor() {
    require_send_sync::<List<i32>>();
    require_send_sync::<ListObserver>();
    require_send_sync::<ListNativeCursor>();
    require_send_sync::<ListCursorObserver>();
    require_copy::<ListCursor>();
    require_unwind::<List<i32>>();
    require_unwind::<ListObserver>();
    require_unwind::<ListNativeCursor>();
}
#[test]
fn observer_identity_stays_with_owner_across_exchange_and_native_moves() {
    let mut first: List<i32> = [1].into_iter().collect();
    let mut second: List<i32> = [2, 3].into_iter().collect();
    let original = first.observer();
    let donor = second.observer();
    let donor_head = donor.first_node();
    first.exchange(&mut second);
    assert_eq!((original.size(), donor.size()), (2, 1));
    assert_eq!(original.first_node(), donor_head);
    let mut moved = List::move_from(&mut first);
    assert!(original.is_empty());
    second.move_assign(&mut moved);
    assert_eq!((donor.size(), moved.observer().size()), (2, 0));
    assert_eq!(donor.first_node(), donor_head);
}
#[test]
fn observer_remains_safe_and_empty_after_owner_is_destroyed() {
    let observer = {
        let list: List<i32> = [1, 2].into_iter().collect();
        list.observer()
    };
    assert_eq!(observer.size(), 0);
    assert!(observer.is_empty());
    assert_eq!((observer.first_node(), observer.last_node()), (None, None));
}
#[test]
fn native_cursor_clone_has_independent_canonical_state() {
    let list: List<i32> = [1, 2].into_iter().collect();
    let cursor = ListNativeCursor::new(&list);
    let observer = cursor.observer();
    let mut copied = cursor.clone();
    copied.next(&list).unwrap();
    assert_eq!(cursor.position().value(&list), Some(&1));
    assert_eq!(copied.position().value(&list), Some(&2));
    assert_eq!(observer.current_node(), list.cursor().current_node());
    copied.next(&list).unwrap();
    let previous = copied.observer().previous_node();
    copied.next(&list).unwrap();
    assert_eq!(copied.observer().previous_node(), previous);
    assert!(!copied.observer().more());
    assert!(!ListNativeCursor::default().observer().more());
}
#[test]
fn native_cursor_rejects_stale_predecessor_without_mutating_owner() {
    let mut list: List<i32> = [1, 2].into_iter().collect();
    let mut cursor = ListNativeCursor::new(&list);
    cursor.next(&list).unwrap();
    list.insert_before(9, &cursor.position()).unwrap();
    assert!(list.remove_native(&mut cursor).is_err());
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [1, 9, 2]);
    let foreign: List<i32> = [4].into_iter().collect();
    assert!(cursor.next(&foreign).is_err());
}
#[test]
fn native_cursor_panic_still_finishes_owner_and_cursor_phases() {
    let (mut list, trace) = fixture::fixture(1, 1);
    let mut cursor = ListNativeCursor::new(&list);
    *trace.cursor.borrow_mut() = Some(cursor.observer());
    trace.recording.set(true);
    assert!(catch_unwind(AssertUnwindSafe(|| list.remove_native(&mut cursor))).is_err());
    assert_eq!(&*trace.legacy.borrow(), &["drop 1 1 1 1"]);
    assert!(list.observer().is_empty());
    assert_eq!(list.observer().size(), 0);
    assert!(!cursor.observer().more());
}
#[test]
fn clear_panic_resets_canonical_metadata_before_remaining_destructors() {
    let (mut list, trace) = fixture::fixture(3, 2);
    trace.recording.set(true);
    assert!(catch_unwind(AssertUnwindSafe(|| list.clear())).is_err());
    assert_eq!(
        &*trace.legacy.borrow(),
        &["drop 1 3 0 0", "drop 2 3 0 0", "drop 3 0 1 0"]
    );
    assert_eq!(list.observer().size(), 0);
    assert!(list.observer().is_empty());
}

#[path = "list_observer_contracts/parallel.rs"]
mod parallel;

fn reused_cursor_fixture() -> (List<i32>, ListNativeCursor) {
    let arena = rustycad_tkernel::ncollection::BaseAllocator::incremental(4096);
    let mut list = List::from_iter_in([1, 2], arena.clone());
    let cursor = ListNativeCursor::new(&list);
    let original = cursor.observer().current_node();
    list.clear();
    arena.reset(false).unwrap();
    list.append(3);
    list.append(4);
    assert_eq!(list.observer().first_node(), original);
    (list, cursor)
}
#[test]
fn native_cursor_resolves_fresh_provenance_after_arena_address_reuse() {
    let (mut list, mut cursor) = reused_cursor_fixture();
    cursor.next(&list).unwrap();
    assert_eq!(cursor.position().value(&list), Some(&4));
    list.remove_native(&mut cursor).unwrap();
    assert_eq!(list.first(), Some(&3));
    let (mut list, mut cursor) = reused_cursor_fixture();
    list.remove_native(&mut cursor).unwrap();
    assert_eq!(list.first(), Some(&4));
}
