//! Behaviour of `WeakHandle`, the non-owning raw pointer of the MAT graph: the cases
//! the doc example does not reach (a destroyed object, a null handle, clones).
use rustycad_tkernel::standard::{Handle, StandardTransient, WeakHandle};

fn owned_transient() -> Handle<StandardTransient> {
    Handle::new(StandardTransient::default())
}

#[test]
fn upgrade_of_a_destroyed_object_is_a_null_handle_but_the_pointer_is_not_null() {
    let owner: Handle<StandardTransient> = owned_transient();
    let pointer: WeakHandle<StandardTransient> = owner.downgrade();
    drop(owner);
    assert!(!pointer.is_null(), "C++ keeps a non-null dangling pointer");
    assert!(
        pointer.upgrade().is_null(),
        "the upgrade must not revive the object"
    );
}

#[test]
fn downgrade_of_a_null_handle_is_a_null_pointer() {
    let null_handle: Handle<StandardTransient> = Handle::default();
    let pointer: WeakHandle<StandardTransient> = null_handle.downgrade();
    assert!(pointer.is_null());
    assert!(pointer.upgrade().is_null());
}

#[test]
fn pointers_never_change_the_reference_count() {
    let owner: Handle<StandardTransient> = owned_transient();
    let pointer: WeakHandle<StandardTransient> = owner.downgrade();
    let copy: WeakHandle<StandardTransient> = pointer.clone();
    assert_eq!(owner.ref_count(), 1);
    let upgraded: Handle<StandardTransient> = copy.upgrade();
    assert!(upgraded == owner);
    assert_eq!(
        owner.ref_count(),
        2,
        "only the upgraded handle owns a reference"
    );
}
