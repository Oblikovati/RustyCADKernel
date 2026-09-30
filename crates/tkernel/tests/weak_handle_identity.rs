//! Native pointer maps retain identity without extending object lifetime.
use rustycad_tkernel::{
    ncollection::{HashKey, Map},
    standard::{Handle, StandardTransient, WeakHandle},
};
#[test]
fn weak_identity_and_hash_survive_owner_destruction() {
    let owner = Handle::new(StandardTransient::default());
    let pointer = owner.downgrade();
    let alias = pointer.clone();
    let address = owner.object_address();
    assert_eq!(pointer.object_address(), address);
    assert_eq!(pointer, alias);
    let mut pointers: Map<WeakHandle<StandardTransient>> = Map::default();
    assert!(pointers.add(pointer.clone()));
    assert!(!pointers.add(alias.clone()));
    drop(owner);
    assert!(pointer.upgrade().is_null());
    assert_eq!(pointer.object_address(), address);
    assert_eq!(pointer.occt_hash(), address as usize);
    assert!(pointers.contains(&alias));
    assert!(format!("{pointer:?}").starts_with("WeakHandle("));
}
#[test]
fn null_and_distinct_weak_pointers_compare_by_native_identity() {
    let null = WeakHandle::<StandardTransient>::default();
    assert_eq!(null.object_address(), std::ptr::null());
    assert_eq!(null.occt_hash(), 0);
    let first = Handle::new(StandardTransient::default());
    let second = Handle::new(StandardTransient::default());
    assert_ne!(first.downgrade(), second.downgrade());
    assert_ne!(null, first.downgrade());
}
