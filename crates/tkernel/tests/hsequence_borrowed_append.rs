//! Explicit copy construction supports native item types without a Rust Clone implementation.
#[path = "hsequence_borrowed_append/fixture.rs"]
mod fixture;
#[path = "hsequence_borrowed_append/native_rows.rs"]
mod native_rows;
use fixture::{BorrowedCase, BorrowedItem, copy_item};
use rustycad_tkernel::ncollection::HSequence;

#[test]
fn native_borrowed_append_rows_match_allocation_publication_and_cleanup() {
    assert_eq!(
        native_rows::all_rows(),
        include_str!("../../../tools/oracle/tkernel_hsequence_borrowed_append.expected")
    );
}
#[test]
fn explicit_copy_policy_consumes_capture_once_and_retains_donor() {
    let case = BorrowedCase::new(0);
    let donor = BorrowedItem::new(9, false, &case.journal);
    let token = String::from("construct once");
    case.owner.append_ref_with_constructor(&donor, |source| {
        assert_eq!(token, "construct once");
        drop(token);
        copy_item(source, &case.arena, &case.observer)
    });
    assert_eq!(case.values(), "9");
    assert_eq!((donor.number, donor.reads()), (9, 1));
}
#[test]
fn clone_convenience_retains_the_borrowed_item() {
    let donor = String::from("OCCT");
    let owner = HSequence::default();
    owner.append_ref(&donor);
    assert_eq!(donor, "OCCT");
    assert_eq!(owner.sequence(|s| s.first().cloned()), Some(donor));
}
#[test]
fn explicit_constructor_supports_values_borrowed_from_the_callers_scope() {
    let text = String::from("scope-owned");
    let donor = text.as_str();
    let owner = HSequence::default();
    owner.append_ref_with_constructor(&donor, |source| *source);
    assert_eq!(owner.sequence(|s| s.first().copied()), Some("scope-owned"));
}
