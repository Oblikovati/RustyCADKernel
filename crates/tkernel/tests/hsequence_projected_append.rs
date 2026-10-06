//! Explicit projection covers native derived-handle lookup; no inheritance callback is inferred.
#[path = "hsequence_projected_append/fixture.rs"]
mod fixture;
#[path = "hsequence_projected_append/native_rows.rs"]
mod native_rows;
use fixture::{AppendJournal, DerivedSequence, values};
use rustycad_tkernel::{
    ncollection::{BaseAllocator, HSequence},
    standard::Handle,
};

#[test]
fn native_derived_handle_rows_match_dispatch_transfer_and_copy_failure_prefixes() {
    assert_eq!(
        native_rows::all_rows(),
        include_str!("../../../tools/oracle/tkernel_hsequence_projected_append.expected")
    );
}
#[test]
fn null_handle_rejects_before_projection_as_a_rust_safety_extension() {
    let target = HSequence::<i32>::default();
    let error = target
        .append_handle_projected(&Handle::<DerivedSequence>::default(), |_| {
            panic!("null derived handle reached projection; expected null rejection")
        })
        .unwrap_err();
    assert!(error.to_string().contains("null handle"));
    assert!(target.sequence(|items| items.is_empty()));
}
#[test]
fn projection_accepts_a_consumed_callback_capture_before_self_noop() {
    let journal = AppendJournal::default();
    let source = Handle::new(DerivedSequence::new(&BaseAllocator::default(), &journal));
    let token = String::from("project once");
    source
        .get()
        .unwrap()
        .base
        .append_handle_projected(&source, move |owner| {
            assert_eq!(token, "project once");
            drop(token);
            owner.selected()
        })
        .unwrap();
    assert_eq!(journal.events(), "g;");
    assert_eq!(values(&source.get().unwrap().base), "-");
}
