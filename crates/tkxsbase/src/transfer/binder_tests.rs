//! Unit tests of `Transfer_Binder` and its concrete binders.
use super::*;
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient};
use rustycad_tkernel::tcollection::HAsciiString;

fn simple(text: &str) -> Handle<dyn TransferBinder> {
    let binder = TransferSimpleBinderOfTransient::new();
    binder
        .set_result(&Handle::new(HAsciiString::new(text)).cast())
        .unwrap();
    Handle::new(binder).cast()
}

fn chain_len(first: &Handle<dyn TransferBinder>) -> usize {
    let mut count = 0;
    let mut current = first.next_result();
    while current.is_some() {
        count += 1;
        current = current.next_result();
    }
    count
}

#[test]
fn chain_append_cut_and_multiple() {
    let head: Handle<dyn TransferBinder> = Handle::new(TransferVoidBinder::new()).cast();
    let (b1, b2, b3) = (simple("a"), simple("b"), simple("c"));
    head.add_result(&head);
    head.add_result(&Handle::default());
    assert_eq!(chain_len(&head), 0);
    head.add_result(&b1);
    head.add_result(&b2);
    head.add_result(&b3);
    assert_eq!(chain_len(&head), 3);
    assert!(head.is_multiple() && b1.is_multiple() && !b3.is_multiple());
    // native quirk: adding b2 again appends it after the end b3, and b3.AddResult(b2) cuts
    // b3 from the chain of b2, so b3 leaves the chain of head (head -> b1 -> b2)
    head.add_result(&b2);
    assert!(b1.next_result() == b2 && b2.next_result().is_null() && b3.next_result() == b2);
    assert_eq!(chain_len(&head), 2);
    head.add_result(&b3);
    assert!(b2.next_result() == b3 && b3.next_result().is_null());
    assert_eq!(chain_len(&head), 3);
}

#[test]
fn statuses_checks_and_merge() {
    let binder = TransferSimpleBinderOfTransient::new();
    assert_eq!(binder.status(), TransferStatusResult::StatusVoid);
    binder.set_already_used();
    assert_eq!(binder.status(), TransferStatusResult::StatusVoid);
    binder
        .set_result(&Handle::new(StandardTransient::default()))
        .unwrap();
    assert_eq!(binder.status_exec(), TransferStatusExec::StatusDone);
    binder.set_already_used();
    let err = binder.set_result(&Handle::default()).unwrap_err();
    assert_eq!(err.exception_type(), "Transfer_TransferFailure");
    let other: Handle<dyn TransferBinder> = Handle::new(TransferVoidBinder::new()).cast();
    other.add_fail("bad", "");
    other.add_warning("warn", "");
    binder.merge(&other);
    binder.merge(&Handle::default());
    assert_eq!(binder.status_exec(), TransferStatusExec::StatusError);
    assert!(binder.check().has_failed() && binder.c_check().has_warnings());
    binder.set_status_exec(TransferStatusExec::StatusRun);
    assert_eq!(binder.status_exec(), TransferStatusExec::StatusRun);
}

#[test]
fn simple_binder_typed_result_and_types() {
    let empty = TransferSimpleBinderOfTransient::new();
    assert_eq!(empty.result_type_name(), "(void)");
    assert_eq!(empty.result_type().name(), "Standard_Transient");
    let head = simple("x");
    head.add_result(&simple("y"));
    let mut res = Handle::default();
    assert!(TransferSimpleBinderOfTransient::get_typed_result(
        &head,
        HAsciiString::static_type(),
        &mut res
    ));
    assert!(res.is_some());
    let none: Handle<dyn TransferBinder> = Handle::new(TransferVoidBinder::new()).cast();
    assert!(!TransferSimpleBinderOfTransient::get_typed_result(
        &none,
        HAsciiString::static_type(),
        &mut Handle::default()
    ));
    assert_eq!(head.result_type_name(), "TCollection_HAsciiString");
    let void = TransferVoidBinder::new();
    assert_eq!(
        (void.result_type().name(), void.result_type_name().as_str()),
        ("Transfer_VoidBinder", "(void)")
    );
}

#[test]
fn multiple_and_list_binders() {
    let multi = TransferMultipleBinder::new();
    assert!(!multi.is_multiple() && multi.nb_results() == 0 && multi.multiple_result().is_empty());
    multi.add_transient_result(&Handle::new(StandardTransient::default()));
    assert!(!multi.is_multiple());
    multi.add_transient_result(&Handle::default());
    assert!(multi.is_multiple() && multi.result_value(1).is_ok());
    assert_eq!(
        multi.result_value(3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    multi.set_multiple_result(None);
    assert_eq!(
        (multi.nb_results(), multi.result_type_name().as_str()),
        (0, "(list)")
    );
    let list = TransferTransientListBinder::with_list(&[Handle::default()]);
    assert!(!list.is_multiple());
    list.add_transient_result(&Handle::new(StandardTransient::default()));
    assert!(
        list.is_multiple()
            && list
                .set_result(1, &Handle::new(StandardTransient::default()))
                .is_ok()
    );
    assert!(list.set_result(0, &Handle::default()).is_err() && list.transient(3).is_err());
    assert!(list.transient(1).unwrap().is_some() && list.result().len() == 2);
    assert_eq!(list.result_type().name(), "Transfer_TransientListBinder");
    assert_eq!(
        TransferTransientListBinder::new().result_type_name(),
        "list(Standard_Transient)"
    );
}
