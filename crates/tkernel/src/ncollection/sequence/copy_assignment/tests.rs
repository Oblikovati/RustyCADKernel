use super::super::copy_construction::tests::fixture;
use crate::ncollection::{BaseAllocator, Sequence};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
mod native_rows;
use fixture::{CopyItem, CopyTrace, events};

#[test]
fn assign_and_actual_clone_from_trait_match_native_assignment_rows() {
    assert_eq!(
        native_rows::all_rows(),
        include_str!("../../../../../../tools/oracle/tkernel_sequence_assignment_policy.expected")
    );
}
#[test]
fn non_clone_callback_keeps_borrowed_values_and_receiver_allocator() {
    struct Borrowed<'a>(&'a str);
    let text = String::from("OCCT");
    let source: Sequence<_> = [Borrowed(&text)].into_iter().collect();
    let arena = BaseAllocator::incremental(1024);
    let mut target = Sequence::with_allocator(arena.clone());
    let address = std::ptr::from_ref(&target);
    let returned = target.assign_with_constructor(&source, |item| Borrowed(item.0));
    assert_eq!(std::ptr::from_ref(returned), address);
    assert_eq!(returned.first().unwrap().0, "OCCT");
    assert_eq!(returned.allocator(), &arena);
}
#[test]
fn callback_failure_keeps_committed_prefix_and_releases_only_failed_slot() {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let trace = make_trace(&arena, 2);
    let source = fixture::make_source(&trace, 3);
    let mut target = Sequence::with_allocator(arena.clone());
    target.emplace_append(|| CopyItem {
        number: 9,
        reads: Cell::new(0),
        trace: &trace,
    });
    assert!(failed_policy_assignment(&mut target, &source));
    assert_eq!(target.len(), 1);
    assert_eq!(events(&trace), "d9;c1/4;c2/5;");
    assert_eq!(fixture::live_allocations(&arena), 4);
    trace.events.borrow_mut().clear();
    target.clear(None);
    assert_eq!(events(&trace), "d1;");
}

fn make_trace(arena: &BaseAllocator, failure: i32) -> CopyTrace<'_> {
    CopyTrace {
        arena,
        events: Default::default(),
        failing: Cell::new(failure),
    }
}
fn failed_policy_assignment<'a>(
    target: &mut Sequence<CopyItem<'a>>,
    source: &Sequence<CopyItem<'a>>,
) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        target.assign_with_constructor(source, Clone::clone);
    }))
    .is_err()
}
