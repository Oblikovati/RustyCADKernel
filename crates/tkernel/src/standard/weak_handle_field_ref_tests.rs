//! Captured shared fields retain no owner or target and reject changed locator identity.
use super::*;
use crate::standard::{StandardTransient, catch_native_failure};

struct WeakFieldAliasFixture {
    target: Handle<StandardTransient>,
    alternate: Handle<StandardTransient>,
    choose_alternate: bool,
}
fn weak_alias_target(fields: &WeakFieldAliasFixture) -> &Handle<StandardTransient> {
    // The locator identifies the original field independently of later virtual mode changes.
    &fields.target
}
fn weak_alias_alternate(fields: &WeakFieldAliasFixture) -> &Handle<StandardTransient> {
    // Virtual selection chooses this locator once, rather than on each retained alias read.
    &fields.alternate
}
fn weak_alias_invalid_mode_locator(fields: &WeakFieldAliasFixture) -> &Handle<StandardTransient> {
    if fields.choose_alternate {
        return &fields.alternate;
    }
    &fields.target
}
fn weak_alias_fields(
    target: &Handle<StandardTransient>,
    alternate: &Handle<StandardTransient>,
) -> Arc<SharedState<WeakFieldAliasFixture>> {
    Arc::new(SharedState::new(WeakFieldAliasFixture {
        target: target.clone(),
        alternate: alternate.clone(),
        choose_alternate: false,
    }))
}
#[test]
fn weak_field_alias_keeps_actual_slot_identity_across_replacement_without_ownership() {
    let first = Handle::new(StandardTransient::default());
    let second = Handle::new(StandardTransient::default());
    let fields = weak_alias_fields(&first, &second);
    let alias = WeakHandleFieldRef::new(&fields, weak_alias_target);
    assert_eq!(Arc::strong_count(&fields), 1);
    assert_eq!(first.ref_count(), 2);
    assert_eq!(alias.reference_address(), alias.clone().reference_address());
    fields.write().target = second.clone();
    assert_eq!(first.ref_count(), 1);
    assert_eq!(alias.observe().object_address(), second.object_address());
    assert_eq!(second.ref_count(), 3);
    verify_weak_alias_erased_operations(&alias, &fields);
    drop(fields);
    assert_eq!(second.ref_count(), 1);
    assert!(alias.is_null());
    assert!(alias.reference_address().is_null());
}
fn verify_weak_alias_erased_operations(
    alias: &WeakHandleFieldRef<WeakFieldAliasFixture, StandardTransient>,
    fields: &Arc<SharedState<WeakFieldAliasFixture>>,
) {
    let erased: &dyn HandleReference<StandardTransient> = alias;
    assert_eq!(erased.reference_address(), alias.reference_address());
    assert_eq!(erased.observe().ref_count(), 3);
    let copied = alias.clone_handle();
    assert_eq!(copied.ref_count(), 4);
    drop(copied);
    fields.write().target = Handle::default();
    assert!(erased.is_null());
    assert!(erased.object_address().is_null());
    assert!(erased.clone_handle().is_null());
}
#[test]
fn weak_alias_expiration_does_not_keep_native_field_ownership_alive() {
    let target = Handle::new(StandardTransient::default());
    let fields = weak_alias_fields(&target, &Handle::default());
    let alias = WeakHandleFieldRef::new(&fields, weak_alias_target);
    drop(fields);
    assert_eq!(target.ref_count(), 1);
    assert!(alias.observe().is_null());
    assert!(alias.object_address().is_null());
    assert!(alias.clone_handle().is_null());
}
#[test]
fn captured_fields_do_not_reselect_when_virtual_mode_changes() {
    let first = Handle::new(StandardTransient::default());
    let second = Handle::new(StandardTransient::default());
    let fields = weak_alias_fields(&first, &second);
    let original = WeakHandleFieldRef::new(&fields, weak_alias_target);
    fields.write().choose_alternate = true;
    let alternate = WeakHandleFieldRef::new(&fields, weak_alias_alternate);
    assert_eq!(original.object_address(), first.object_address());
    assert_eq!(alternate.object_address(), second.object_address());
    assert_ne!(original.reference_address(), alternate.reference_address());
    assert_eq!(first.ref_count(), 2);
    assert_eq!(second.ref_count(), 2);
}
#[test]
fn changing_a_locator_field_is_rejected_instead_of_rebinding_a_retained_alias() {
    let fields = weak_alias_fields(&Handle::default(), &Handle::default());
    let alias = WeakHandleFieldRef::new(&fields, weak_alias_invalid_mode_locator);
    fields.write().choose_alternate = true;
    let failure = catch_native_failure(|| Ok(alias.observe())).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert!(failure.message().contains("expected captured slot="));
}
