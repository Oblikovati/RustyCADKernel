use super::super::{BaseAllocator, Sequence, SequenceNode, SequenceNodeOwner};
use super::IncompleteSequenceCopy;
use crate::ncollection::HSequence;
use std::cell::Cell;
use std::marker::PhantomPinned;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;
pub(in crate::ncollection::sequence) mod fixture;
mod native_rows;
#[allow(unsafe_code)]
mod safety;

#[test]
fn actual_clone_and_wrapper_copy_match_native_constructor_failure_rows() {
    assert_eq!(
        native_rows::all_rows(),
        include_str!("../../../../../../tools/oracle/tkernel_hsequence_copy_construction.expected")
    );
}
#[test]
fn callback_constructor_and_wrapper_accept_non_clone_borrowed_payloads() {
    struct Borrowed<'a>(&'a str);
    let text = String::from("OCCT");
    let source: Sequence<_> = [Borrowed(&text)].into_iter().collect();
    let copied = Sequence::from_copied_with(&source, |item| Borrowed(item.0));
    let shared = HSequence::from_sequence_with_constructor(&source, |item| Borrowed(item.0));
    assert_eq!(copied.first().unwrap().0, "OCCT");
    shared.sequence(|sequence| assert_eq!(sequence.first().unwrap().0, "OCCT"));
}
#[test]
fn previous_clone_algorithm_reproduces_the_observed_native_failure_mismatch() {
    use fixture::{CopyTrace, events, live_allocations, make_source};
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let trace = CopyTrace {
        arena: &arena,
        events: Default::default(),
        failing: Cell::new(2),
    };
    let source = make_source(&trace, 3);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        // Exact pre-fix Sequence::clone body: live assignment owns and destroys its prefix.
        let mut copy = Sequence::with_allocator(source.allocator().clone());
        copy.assign(&source);
        copy
    }));
    assert!(failure.is_err());
    assert_eq!(events(&trace), "c1/4;c2/5;d1;");
    assert_eq!(live_allocations(&arena), 3);
}
