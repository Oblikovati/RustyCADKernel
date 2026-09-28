//! opencascade::handle construction, casts, assignment, comparison and hashing.
use rustycad_tkernel::ncollection::{AllocatorObject, HeapAllocatorObject};
use rustycad_tkernel::standard::{Handle, StandardPersistent, StandardTransient, occ};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn down_cast_shares_or_returns_null() {
    let base = Handle::new(StandardPersistent::default()).cast::<StandardTransient>();
    let derived = Handle::<StandardPersistent>::down_cast(&base);
    assert!(!derived.is_null());
    assert_eq!(2, base.ref_count());
    let unrelated = occ::down_cast::<HeapAllocatorObject, _>(&base);
    assert!(unrelated.is_null());
    assert_eq!(2, base.ref_count());
    let same = Handle::<StandardTransient>::down_cast(&base);
    assert!(same == base);
    assert_eq!(3, base.ref_count());
}
#[test]
fn from_view_adds_one_reference() {
    let owner = Handle::new(StandardPersistent::default());
    let view = owner.view::<StandardTransient>();
    let shared = Handle::<StandardTransient>::from_view(&view);
    assert_eq!(2, owner.ref_count());
    assert!(shared == owner);
    let derived = occ::down_cast_view::<StandardPersistent, _>(&view);
    assert_eq!(3, owner.ref_count());
    assert!(derived == owner);
    let failed = Handle::<AllocatorObject>::from_view(&view);
    assert!(failed.is_null());
    assert_eq!(3, owner.ref_count());
}
#[test]
fn into_cast_moves_without_changing_the_count() {
    let owner = Handle::new(StandardPersistent::default());
    let copy = owner.clone();
    assert_eq!(2, owner.ref_count());
    let base: Handle<StandardTransient> = copy.into_cast();
    assert_eq!(2, owner.ref_count());
    let rejected: Handle<AllocatorObject> = base.into_cast();
    assert!(rejected.is_null());
    assert_eq!(1, owner.ref_count());
    assert!(
        Handle::<StandardTransient>::default()
            .into_cast::<StandardPersistent>()
            .is_null()
    );
}
#[test]
fn reset_replaces_and_clone_from_keeps_shared_objects() {
    let mut handle = Handle::new(StandardTransient::default());
    let first = handle.clone();
    handle.reset(StandardTransient::default());
    assert!(handle != first);
    assert_eq!(1, first.ref_count());
    let mut same = first.clone();
    same.clone_from(&first);
    assert_eq!(2, first.ref_count());
    same.clone_from(&handle);
    assert_eq!(1, first.ref_count());
    assert_eq!(2, handle.ref_count());
}
#[test]
fn ordering_and_hash_use_the_object_address() {
    let null = Handle::<StandardTransient>::default();
    let live = Handle::new(StandardTransient::default());
    assert_eq!(0, null.hash_code());
    assert_eq!(live.as_ptr() as usize, live.hash_code());
    assert!(null < live);
    assert_eq!(live.hash_code().cmp(&null.hash_code()), live.cmp(&null));
    let copy = live.clone();
    assert_eq!(std::cmp::Ordering::Equal, live.cmp(&copy));
    assert_eq!(digest(&live), digest(&copy));
}
fn digest(handle: &Handle<StandardTransient>) -> u64 {
    let mut hasher = DefaultHasher::new();
    handle.hash(&mut hasher);
    hasher.finish()
}
