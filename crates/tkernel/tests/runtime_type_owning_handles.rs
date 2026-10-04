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
    let handles: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert!(
        handles
            .iter()
            .all(|handle| handle.name() == "ConcurrentChild")
    );
    assert_eq!(before + 1, parent.transient_base().ref_count());
    assert_eq!(
        13,
        ConcurrentChild::static_type().transient_base().ref_count()
    );
    drop(handles);
    assert_eq!(
        1,
        ConcurrentChild::static_type().transient_base().ref_count()
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
