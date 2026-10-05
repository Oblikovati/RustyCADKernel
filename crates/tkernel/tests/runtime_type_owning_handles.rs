//! Focused ownership checks for native-style class descriptors.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped, StandardTransient, Transient};
use std::sync::{Arc, Barrier};

#[derive(Default)]
struct OwnerParent {
    base: StandardTransient,
}
impl_transient!(OwnerParent, "OwnerParent", StandardTransient, base);

#[derive(Default)]
struct OwnerChild {
    parent: OwnerParent,
}
impl_transient!(OwnerChild, "OwnerChild", OwnerParent, parent);

#[derive(Default)]
struct ConcurrentParent {
    base: StandardTransient,
}
impl_transient!(
    ConcurrentParent,
    "ConcurrentParent",
    StandardTransient,
    base
);

#[derive(Default)]
struct ConcurrentChild {
    parent: ConcurrentParent,
}
impl_transient!(ConcurrentChild, "ConcurrentChild", ConcurrentParent, parent);

#[derive(Default)]
struct DynamicOwner {
    base: StandardTransient,
}
impl_transient!(DynamicOwner, "DynamicOwner", StandardTransient, base);

#[derive(Default)]
struct WeakOwnerDescriptor {
    base: StandardTransient,
}
impl_transient!(
    WeakOwnerDescriptor,
    "WeakOwnerDescriptor",
    StandardTransient,
    base
);

#[derive(Default)]
struct RawParent {
    base: StandardTransient,
}
impl_transient!(RawParent, "RawParent", StandardTransient, base);

#[test]
fn static_descriptors_own_one_reference_and_parent_links_own_one_each() {
    let parent = OwnerParent::static_type();
    assert_eq!(1, parent.transient_base().ref_count());
    let before = parent.transient_base().ref_count();
    let child = OwnerChild::static_type();
    assert_eq!(before + 1, parent.transient_base().ref_count());
    assert_eq!(1, child.transient_base().ref_count());
    assert_eq!(2, OwnerChild::static_type_handle().ref_count());
    assert_eq!(1, child.transient_base().ref_count());
}

#[test]
fn concurrent_descriptor_requests_register_parent_ownership_once() {
    let parent = ConcurrentParent::static_type();
    let before = parent.transient_base().ref_count();
    let handles = concurrent_static_child_handles();
    assert_static_registration_counts(before, &handles);
    drop(handles);
    assert_eq!(
        1,
        ConcurrentChild::static_type().transient_base().ref_count()
    );
}

fn concurrent_static_child_handles() -> Vec<rustycad_tkernel::standard::Handle<RuntimeType>> {
    let barrier = Arc::new(Barrier::new(12));
    let workers: Vec<_> = (0..12)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                ConcurrentChild::static_type_handle()
            })
        })
        .collect();
    workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect()
}

fn assert_static_registration_counts(
    before: i32,
    handles: &[rustycad_tkernel::standard::Handle<RuntimeType>],
) {
    assert!(
        handles
            .iter()
            .all(|handle| handle.get().unwrap().name() == "ConcurrentChild")
    );
    assert_eq!(
        13,
        ConcurrentChild::static_type().transient_base().ref_count()
    );
    assert_eq!(
        before + 1,
        ConcurrentParent::static_type().transient_base().ref_count()
    );
}

#[test]
fn raw_registration_does_not_create_a_descriptor_owner() {
    struct RawRegistration;
    let parent = RawParent::static_type();
    let before = parent.transient_base().ref_count();
    let descriptor = RuntimeType::register::<RawRegistration>("RawRegistration", 0, Some(parent));
    assert_eq!(0, descriptor.transient_base().ref_count());
    assert_eq!(before + 1, parent.transient_base().ref_count());
}

#[test]
fn generic_class_owner_cache_keeps_one_descriptor_reference() {
    struct GenericClass;
    let descriptor = RuntimeType::register::<GenericClass>("GenericClass", 0, None);
    assert_eq!(0, descriptor.transient_base().ref_count());
    RuntimeType::retain_class_descriptor::<GenericClass>(descriptor);
    assert_eq!(1, descriptor.transient_base().ref_count());
    RuntimeType::retain_class_descriptor::<GenericClass>(descriptor);
    assert_eq!(1, descriptor.transient_base().ref_count());
}

#[test]
fn transient_and_handle_descriptor_accessors_return_owning_handles() {
    let object = rustycad_tkernel::standard::Handle::new(DynamicOwner::default());
    let dynamic = object.dynamic_type_handle().unwrap();
    assert_eq!("DynamicOwner", dynamic.name());
    assert_eq!(2, dynamic.ref_count());
    drop(dynamic);
    assert_eq!(1, DynamicOwner::static_type().transient_base().ref_count());
    let from_object = object.get().unwrap().dynamic_type_handle();
    assert_eq!("DynamicOwner", from_object.name());
    assert_eq!(2, from_object.ref_count());
}

#[test]
fn weak_static_descriptor_reupgrades_without_changing_identity() {
    let descriptor = WeakOwnerDescriptor::static_type_handle();
    let address = descriptor.object_address();
    let weak = descriptor.downgrade();
    drop(descriptor);
    let upgraded = weak.upgrade();
    assert_eq!(address, upgraded.object_address());
    assert_eq!(2, upgraded.ref_count());
}

struct OwnedLifecycleParent;
struct OwnedLifecycleChild;

#[test]
fn owned_registration_unbinds_then_reuses_type_identity() {
    let parent = RuntimeType::register_owned::<OwnedLifecycleParent>("OwnedParent", 24, None);
    let first = register_owned_child(&parent);
    assert_owned_first_registration_wins(&first, &parent);
    let weak = first.downgrade();
    assert_eq!(1, first.ref_count());
    assert_eq!(2, parent.ref_count());
    drop(first);
    assert!(weak.upgrade().is_null());
    assert_eq!(1, parent.ref_count());
    assert_re_registered_child(&parent);
}

fn assert_owned_first_registration_wins(
    first: &rustycad_tkernel::standard::Handle<RuntimeType>,
    parent: &rustycad_tkernel::standard::Handle<RuntimeType>,
) {
    let repeated = RuntimeType::register_owned::<OwnedLifecycleChild>("Ignored", 99, None);
    assert_eq!(first.object_address(), repeated.object_address());
    assert_eq!("FirstChild", repeated.name());
    assert_eq!(48, repeated.size());
    assert_eq!(
        parent.object_address(),
        repeated.parent_handle().unwrap().object_address()
    );
}

fn register_owned_child(
    parent: &rustycad_tkernel::standard::Handle<RuntimeType>,
) -> rustycad_tkernel::standard::Handle<RuntimeType> {
    RuntimeType::register_owned::<OwnedLifecycleChild>("FirstChild", 48, Some(parent.clone()))
}

fn assert_re_registered_child(parent: &rustycad_tkernel::standard::Handle<RuntimeType>) {
    let replacement =
        RuntimeType::register_owned::<OwnedLifecycleChild>("ReplacementChild", 64, None);
    assert_eq!("ReplacementChild", replacement.name());
    assert_eq!(64, replacement.size());
    assert!(replacement.parent().is_none());
    assert_eq!(1, replacement.ref_count());
    assert_eq!(1, parent.ref_count());
}

struct ConcurrentOwnedDescriptor;

#[test]
fn concurrent_owned_registration_shares_one_live_descriptor() {
    let anchor = RuntimeType::register_owned::<ConcurrentOwnedDescriptor>("Concurrent", 8, None);
    assert_concurrent_registrations_share(&anchor);
}

fn assert_concurrent_registrations_share(anchor: &rustycad_tkernel::standard::Handle<RuntimeType>) {
    let workers = owned_registration_workers();
    let handles = join_owned_registration_workers(workers);
    assert!(
        handles
            .iter()
            .all(|handle| handle.object_address() == anchor.object_address())
    );
    assert!(handles.iter().all(|handle| handle.name() == "Concurrent"));
    assert_eq!(13, anchor.ref_count());
}

fn owned_registration_workers()
-> Vec<std::thread::JoinHandle<rustycad_tkernel::standard::Handle<RuntimeType>>> {
    let gate = Arc::new(Barrier::new(12));
    (0..12)
        .map(|_| spawn_owned_registration_worker(Arc::clone(&gate)))
        .collect()
}

fn spawn_owned_registration_worker(
    gate: Arc<Barrier>,
) -> std::thread::JoinHandle<rustycad_tkernel::standard::Handle<RuntimeType>> {
    std::thread::spawn(move || {
        gate.wait();
        RuntimeType::register_owned::<ConcurrentOwnedDescriptor>("Concurrent", 8, None)
    })
}

fn join_owned_registration_workers(
    workers: Vec<std::thread::JoinHandle<rustycad_tkernel::standard::Handle<RuntimeType>>>,
) -> Vec<rustycad_tkernel::standard::Handle<RuntimeType>> {
    workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect()
}

struct PinnedThenClassDescriptor;

#[test]
fn class_owner_retains_baseline_after_legacy_storage_pin() {
    let owner = RuntimeType::register_owned::<PinnedThenClassDescriptor>("Pinned", 16, None);
    let descriptor = RuntimeType::register::<PinnedThenClassDescriptor>("Ignored", 99, None);
    assert_eq!(1, owner.ref_count());
    RuntimeType::retain_class_descriptor::<PinnedThenClassDescriptor>(descriptor);
    assert_eq!(2, owner.ref_count());
    drop(owner);
    assert_eq!(1, descriptor.transient_base().ref_count());
}
