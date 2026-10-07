//! Compare the native UniversalIterator defined domain while rejecting native null UB safely.
#[path = "support/osd_universal/mod.rs"]
mod fixture;
use fixture::{BorrowedPayload, Observations, cursor, position};
use rustycad_tkernel::osd::ParallelUniversalIterator;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn default_and_pointer_constructors_preserve_empty_state() {
    let mut empty = ParallelUniversalIterator::new();
    assert!(empty.value().is_none());
    assert!(empty.value_mut().is_none());
    assert!(ParallelUniversalIterator::default().value().is_none());
    assert!(
        ParallelUniversalIterator::from_interface(None)
            .value()
            .is_none()
    );
}

#[test]
fn independently_owned_clone_retains_non_clone_borrowed_payload() {
    let values = [11, 23, 31];
    let payload = BorrowedPayload { values: &values };
    let observations = Observations::default();
    let source = cursor(&payload, 0, &observations);
    let mut copy = source.clone();
    drop(source);
    copy.increment();
    assert_eq!(position(&copy), 1);
    assert_eq!(*observations.trace.borrow(), "cdi");
    drop(copy);
    assert_eq!(*observations.trace.borrow(), "cdid");
}

#[test]
fn native_equality_pre_and_post_increment_dispatch_order() {
    let payload = BorrowedPayload { values: &[1; 9] };
    let observations = Observations::default();
    let mut source = cursor(&payload, 2, &observations);
    let copy = source.clone();
    assert!(source == copy);
    assert!(!(source != copy));
    let original = std::ptr::from_ref(&source);
    assert_eq!(
        std::ptr::from_mut(source.increment()).cast_const(),
        original
    );
    let prior = source.post_increment();
    assert_eq!((position(&prior), position(&source)), (3, 4));
    assert!(source != copy);
    assert!(!(source == copy));
    assert_eq!(*observations.trace.borrow(), "ceeiciee");
}

#[test]
fn assignment_and_self_assignment_clone_before_old_drop() {
    let payload = BorrowedPayload { values: &[1; 9] };
    let observations = Observations::default();
    let source = cursor(&payload, 2, &observations);
    let mut target = cursor(&payload, 5, &observations);
    let original = std::ptr::from_ref(&target);
    assert_eq!(
        std::ptr::from_mut(target.assign(&source)).cast_const(),
        original
    );
    assert_eq!(
        (position(&target), observations.trace.borrow().as_str()),
        (2, "cd")
    );
    observations.trace.borrow_mut().clear();
    target.assign_self();
    assert_eq!(
        (position(&target), observations.trace.borrow().as_str()),
        (2, "cd")
    );
    observations.trace.borrow_mut().clear();
    target.clone_from(&source);
    assert_eq!(*observations.trace.borrow(), "cd");
}

#[test]
fn clone_panic_preserves_assignment_target_and_post_increment_source() {
    let payload = BorrowedPayload { values: &[1; 9] };
    let observations = Observations::default();
    let mut source = cursor(&payload, 2, &observations);
    let mut target = cursor(&payload, 5, &observations);
    observations.fail_clone.set(true);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            target.assign(&source);
        }))
        .is_err()
    );
    assert_eq!(position(&target), 5);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            source.post_increment();
        }))
        .is_err()
    );
    assert_eq!(position(&source), 2);
    assert_eq!(*observations.trace.borrow(), "cc");
}

#[test]
fn mutable_dereference_dispatches_without_extra_clone() {
    let payload = BorrowedPayload { values: &[7, 8] };
    let observations = Observations::default();
    let mut iterator = cursor(&payload, 0, &observations);
    iterator.value_mut().unwrap().increment();
    assert_eq!(position(&iterator), 1);
    assert_eq!(*observations.trace.borrow(), "i");
}

#[test]
fn empty_operations_are_rejected_without_undefined_behavior() {
    let empty = ParallelUniversalIterator::new();
    assert!(catch_unwind(AssertUnwindSafe(|| empty.clone())).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| empty == empty)).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| empty != empty)).is_err());
    let mut target = ParallelUniversalIterator::new();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            target.assign(&empty);
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            target.assign_self();
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            target.increment();
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            target.post_increment();
        }))
        .is_err()
    );
    assert!(target.value().is_none());
}

#[test]
fn legal_null_clone_preserves_native_copy_and_assignment_states() {
    let payload = BorrowedPayload { values: &[1; 9] };
    let observations = Observations::default();
    let source = cursor(&payload, 2, &observations);
    let mut target = cursor(&payload, 5, &observations);
    observations.null_clone.set(true);
    let copy = source.clone();
    assert!(copy.value().is_none());
    target.assign(&source);
    assert!(target.value().is_none());
    assert_eq!(*observations.trace.borrow(), "ccd");
}

#[test]
fn legal_null_clone_preserves_self_assignment_and_post_increment_states() {
    let payload = BorrowedPayload { values: &[1; 9] };
    let observations = Observations::default();
    let mut source = cursor(&payload, 2, &observations);
    observations.null_clone.set(true);
    let previous = source.post_increment();
    assert!(previous.value().is_none());
    assert_eq!(position(&source), 3);
    source.assign_self();
    assert!(source.value().is_none());
    assert_eq!(*observations.trace.borrow(), "cicd");
}
