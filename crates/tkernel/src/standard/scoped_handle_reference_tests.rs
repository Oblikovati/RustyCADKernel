//! Rust-only safety evidence supplements valid native callback reads, never dangling native reads.
use super::*;
use crate::standard::{ProjectedHandleReference, StandardTransient};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn scoped_slot_retains_original_identity_without_intrusive_ownership() {
    let peer = Handle::new(StandardTransient::default());
    let slot = ScopedHandleReferenceSlot::from_owned(peer.clone());
    let alias = slot.retain_reference().unwrap();
    let second = alias.retain_reference().unwrap();
    assert_eq!(alias.reference_address(), slot.reference_address());
    assert_eq!(second.reference_address(), slot.reference_address());
    assert_eq!(alias.object_address(), peer.object_address());
    assert_eq!(peer.ref_count(), 2);
    drop(slot);
    assert_eq!(peer.ref_count(), 1);
    assert!(alias.is_expired() && second.is_expired());
    assert!(alias.is_null() && alias.reference_address().is_null());
}
#[test]
fn projected_retention_preserves_slot_and_checked_expiration() {
    let slot = ScopedHandleReferenceSlot::from_owned(Handle::new(StandardTransient::default()));
    let projected = ProjectedHandleReference::<StandardTransient, StandardTransient>::new(&slot);
    let alias = projected.retain_reference().unwrap();
    assert_eq!(alias.reference_address(), slot.reference_address());
    assert_eq!(alias.observe().ref_count(), 1);
    let owned = alias.clone_handle();
    assert_eq!(owned.ref_count(), 2);
    drop(slot);
    assert!(alias.is_expired() && alias.observe().is_null());
    assert_eq!(owned.ref_count(), 1);
}
#[test]
fn ordinary_stack_reference_rejects_lifetime_erasure() {
    let stack = Handle::new(StandardTransient::default());
    let failure = stack.retain_reference().err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert!(failure.message().contains("borrowed lifetime"));
    assert!(
        failure
            .message()
            .contains(&format!("{:p}", stack.reference_address()))
    );
    assert_eq!(stack.ref_count(), 1);
}
struct FakeThreadLocalReference<'a> {
    original: &'a dyn HandleReference<StandardTransient>,
    reads: &'a Cell<usize>,
}
impl HandleReference<StandardTransient> for FakeThreadLocalReference<'_> {
    fn retain_reference(&self) -> Result<RetainedHandleReference<StandardTransient>, Failure> {
        self.original.retain_reference()
    }
    fn observe(&self) -> HandleObserver<StandardTransient> {
        self.reads.set(self.reads.get() + 1);
        self.original.observe()
    }
    fn reference_address(&self) -> *const () {
        self.original.reference_address()
    }
}
#[test]
fn custom_borrowed_thread_local_adapter_delegates_original_capability() {
    let slot = ScopedHandleReferenceSlot::from_owned(Handle::new(StandardTransient::default()));
    let reads = Cell::new(0);
    let source = FakeThreadLocalReference {
        original: &slot,
        reads: &reads,
    };
    assert_eq!(source.observe().ref_count(), 1);
    let alias = source.retain_reference().unwrap();
    assert_eq!(alias.reference_address(), slot.reference_address());
    assert_eq!(reads.get(), 1);
    drop(slot);
    assert!(alias.is_expired());
}
#[test]
fn success_typed_foreign_failure_success_releases_each_original_scope() {
    let peer = Handle::new(StandardTransient::default());
    for policy in [0, 1, 2, 0] {
        assert_scoped_failure_reuse(&peer, policy);
    }
}
fn assert_scoped_failure_reuse(peer: &Handle<StandardTransient>, policy: usize) {
    let mut retained = None;
    let terminal = catch_unwind(AssertUnwindSafe(|| {
        let slot = ScopedHandleReferenceSlot::from_owned(peer.clone());
        retained = Some(slot.retain_reference().unwrap());
        assert_eq!(retained.as_ref().unwrap().observe().ref_count(), 2);
        scoped_callback_failure(policy);
    }));
    assert_eq!(terminal.is_err(), policy != 0);
    if let Err(cause) = terminal {
        assert_scoped_failure_cause(cause.as_ref(), policy);
    }
    assert_eq!(peer.ref_count(), 1);
    assert!(retained.unwrap().is_expired());
}
fn scoped_callback_failure(policy: usize) {
    if policy == 1 {
        std::panic::panic_any(Failure::new("callback policy=1; expected success"));
    }
    if policy == 2 {
        std::panic::panic_any("callback policy=2; expected success");
    }
}
fn assert_scoped_failure_cause(cause: &(dyn std::any::Any + Send), policy: usize) {
    if policy == 1 {
        assert_eq!(
            cause.downcast_ref::<Failure>().unwrap().exception_type(),
            "Standard_Failure"
        );
        return;
    }
    assert_eq!(
        cause.downcast_ref::<&str>(),
        Some(&"callback policy=2; expected success")
    );
}
#[test]
fn live_null_slot_is_distinct_from_expiration() {
    let slot = ScopedHandleReferenceSlot::<StandardTransient>::from_owned(Handle::default());
    let alias = slot.retain_reference().unwrap();
    assert!(alias.is_null() && !alias.is_expired());
    assert!(!alias.reference_address().is_null());
    drop(slot);
    assert!(alias.is_expired() && alias.object_address().is_null());
}

#[test]
fn native_slot_identity_and_count_rows_replay_without_occt() {
    let callbacks = include_str!("../../tests/fixtures/picked_view_retention/native-callbacks.txt");
    let reset = include_str!("../../tests/fixtures/picked_view_retention/native-reset-failure.txt");
    for line in callbacks.lines().chain(reset.lines()) {
        let columns: Vec<i32> = line
            .split_whitespace()
            .map(|item| item.parse().unwrap())
            .collect();
        assert_eq!(columns.len(), 11);
        assert_native_slot_row(&columns);
    }
}
fn assert_native_slot_row(columns: &[i32]) {
    let peer = Handle::new(StandardTransient::default());
    let slot = ScopedHandleReferenceSlot::from_owned(peer.clone());
    let alias = slot.retain_reference().unwrap();
    let second = alias.retain_reference().unwrap();
    if columns[2] == 2 {
        drop(slot);
        assert_eq!(peer.ref_count() - 1, columns[5]);
        assert!(alias.is_expired());
        return;
    }
    assert_native_live_identity(&alias, &second, &peer, columns);

    assert_eq!(alias.observe().ref_count() - 1, columns[5]);
    assert!(!alias.is_expired());
}

fn assert_native_live_identity(
    alias: &RetainedHandleReference<StandardTransient>,
    second: &RetainedHandleReference<StandardTransient>,
    peer: &Handle<StandardTransient>,
    columns: &[i32],
) {
    let identities = (
        i32::from(alias.reference_address() == second.reference_address()),
        i32::from(alias.object_address() == peer.object_address()),
    );
    assert_eq!(identities, (columns[3], columns[4]));
}
