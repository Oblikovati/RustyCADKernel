//! Const callback arguments retain storage while preserving the native reference counter.
use rustycad_tkernel::standard::{Handle, StandardTransient};
#[test]
fn observer_and_projection_do_not_add_an_owning_reference() {
    let handle = Handle::new(StandardTransient::default());
    let observed = handle.observe();
    assert_eq!(observed.object_address(), handle.object_address());
    assert_eq!(handle.ref_count(), 1);
    let owned = (*observed).clone();
    assert_eq!(handle.ref_count(), 2);
    drop(owned);
    drop(observed);
    assert_eq!(handle.ref_count(), 1);
}
#[test]
fn null_observer_is_a_null_const_argument() {
    let null = Handle::<StandardTransient>::default();
    let observed = null.observe();
    assert!(observed.is_null());
    assert!(observed.get().is_none());
    assert!(observed.object_address().is_null());
}
#[test]
fn last_native_owner_can_disappear_during_callback_without_dangling_rust_storage() {
    let handle = Handle::new(StandardTransient::default());
    let weak = handle.downgrade();
    let observed = handle.observe();
    drop(handle);
    assert_eq!(observed.ref_count(), 0);
    let owned = (*observed).clone();
    assert_eq!(observed.ref_count(), 1);
    drop(observed);
    assert_eq!(owned.ref_count(), 1);
    drop(owned);
    assert!(weak.upgrade().is_null());
}
#[test]
fn observers_survive_container_growth_without_affecting_its_handle_counts() {
    let mut container = vec![Handle::new(StandardTransient::default())];
    let observed = container[0].observe();
    for _ in 0..128 {
        container.push(Handle::new(StandardTransient::default()));
    }
    assert_eq!(observed.ref_count(), 1);
    assert_eq!(observed.object_address(), container[0].object_address());
    container.clear();
    assert_eq!(observed.ref_count(), 0);
}

#[test]
fn projected_observers_preserve_identity_counts_and_reject_unrelated_types() {
    use rustycad_tkernel::standard::RuntimeType;
    let handle = Handle::new(StandardTransient::default());
    let projected = handle.observe_as::<StandardTransient>();
    let unrelated = handle.observe_as::<RuntimeType>();
    assert_eq!(projected.object_address(), handle.object_address());
    assert_eq!(handle.ref_count(), 1);
    assert!(unrelated.is_null());
    drop(projected);
    assert_eq!(handle.ref_count(), 1);
    assert!(
        Handle::<StandardTransient>::default()
            .observe_as::<RuntimeType>()
            .is_null()
    );
}
