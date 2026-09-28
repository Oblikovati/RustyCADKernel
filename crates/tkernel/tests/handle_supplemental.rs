//! Handle ownership, type projection and actual resource behavior.
use rustycad_tkernel::ncollection::{
    AllocatorObject, HeapAllocatorObject, IncrementalAllocatorObject,
};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient, Transient};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
#[repr(C)]
struct DropTracked {
    parent: StandardTransient,
    drops: Arc<AtomicUsize>,
}
rustycad_tkernel::impl_transient!(DropTracked, "DropTracked", StandardTransient, parent);
impl Drop for DropTracked {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::Relaxed);
    }
}
#[test]
fn casts_borrowed_views_and_nullification_preserve_lifetime() {
    let drops: Arc<AtomicUsize> = Arc::default();
    let handle = Handle::new(DropTracked {
        parent: StandardTransient::default(),
        drops: drops.clone(),
    });
    assert_eq!(handle.ref_count(), 1);
    assert!(handle.transient_base().this().is_ok());
    let view = handle.view::<StandardTransient>();
    assert!(!view.is_null());
    assert_eq!(handle.ref_count(), 1);
    let mut base = handle.cast::<StandardTransient>();
    assert_eq!(handle.ref_count(), 2);
    assert_eq!(view.as_ptr(), base.as_ptr());
    assert_eq!(base.clone().ref_count(), 3);
    base.nullify();
    assert!(base.is_null());
    assert_eq!(handle.ref_count(), 1);
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    drop(handle);
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}
#[test]
fn descriptors_exact_names_and_failed_views() {
    let handle = Handle::new(IncrementalAllocatorObject::default());
    assert!(handle.is_instance_name("NCollection_IncAllocator"));
    assert!(!handle.is_instance_name("NCollection_BaseAllocator"));
    assert!(handle.is_kind(AllocatorObject::static_type()));
    assert!(!handle.is_kind(HeapAllocatorObject::static_type()));
    assert!(handle.view::<HeapAllocatorObject>().is_null());
    assert!(handle.view::<HeapAllocatorObject>().as_ptr().is_null());
    assert_eq!(
        handle.dynamic_type().unwrap().parent().unwrap().name(),
        "NCollection_BaseAllocator"
    );
    assert!(
        Handle::<StandardTransient>::default()
            .dynamic_type()
            .is_none()
    );
    assert!(!Handle::<StandardTransient>::default().is_kind_name("Standard_Transient"));
}
#[test]
fn copied_transient_resets_counter_and_handle_counter_is_atomic() {
    let object = StandardTransient::default();
    assert!(object.this().is_err());
    let handle = Handle::new(object);
    assert_eq!(handle.get().unwrap().clone().ref_count(), 0);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let borrowed = &handle;
            scope.spawn(move || {
                for _ in 0..1000 {
                    let copy = borrowed.clone();
                    assert!(copy.ref_count() >= 2);
                }
            });
        }
    });
    assert_eq!(handle.ref_count(), 1);
    let mut copy = StandardTransient::default();
    copy.clone_from(handle.get().unwrap());
    assert_eq!(copy.ref_count(), 0);
}
#[test]
fn polymorphic_allocator_objects_own_real_allocations() {
    let incremental = Handle::new(IncrementalAllocatorObject::default());
    let base = incremental.cast::<AllocatorObject>();
    assert_eq!(incremental.resource(), base.resource());
    let allocation = base.resource().allocate_value(42_i32);
    assert_eq!(*allocation, 42);
    assert!(incremental.resource().reset(false).is_err());
    drop(allocation);
    incremental.resource().reset(false).unwrap();
    let heap = Handle::new(HeapAllocatorObject::default());
    assert_eq!(heap.resource().allocate(16).as_slice().len(), 16);
}
