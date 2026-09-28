//! Standard_Transient RTTI queries, This() and Standard_Persistent.
use rustycad_tkernel::ncollection::{AllocatorObject, HeapAllocatorObject};
use rustycad_tkernel::standard::{
    Handle, RuntimeTyped, StandardPersistent, StandardTransient, Transient, occ,
};
use std::sync::atomic::Ordering;

#[test]
fn is_instance_and_is_kind_by_descriptor_and_name() {
    let object = HeapAllocatorObject::default();
    assert!(object.is_instance(HeapAllocatorObject::static_type()));
    assert!(!object.is_instance(AllocatorObject::static_type()));
    assert!(object.is_kind(AllocatorObject::static_type()));
    assert!(object.is_instance_name("NCollection_HeapAllocator"));
    assert!(object.is_instance_name("NCollection_HeapAllocator\0tail"));
    assert!(!object.is_instance_name("NCollection_BaseAllocator"));
    assert!(object.is_kind_name("NCollection_BaseAllocator"));
    assert!(!object.is_kind_name("Standard_Persistent"));
}
#[test]
fn occ_helpers_reject_null_objects() {
    let object = StandardPersistent::default();
    assert!(occ::is_kind::<StandardTransient>(Some(&object)));
    assert!(occ::is_instance::<StandardPersistent>(Some(&object)));
    assert!(!occ::is_instance::<StandardTransient>(Some(&object)));
    assert!(!occ::is_kind::<StandardTransient>(None));
    let null = Handle::<StandardTransient>::default();
    assert!(!occ::is_kind_handle::<StandardTransient, _>(&null));
    assert!(!occ::is_instance_handle::<StandardTransient, _>(&null));
    let handle = Handle::new(StandardPersistent::default()).cast::<StandardTransient>();
    assert!(occ::is_kind_handle::<StandardPersistent, _>(&handle));
    assert!(occ::is_instance_handle::<StandardPersistent, _>(&handle));
    assert!(occ::is_kind::<StandardTransient>(
        handle.view::<StandardTransient>().transient()
    ));
}
#[test]
fn this_rejects_an_object_no_handle_owns() {
    let object = StandardTransient::default();
    let Err(failure) = object.this() else {
        panic!("This() accepted an object with reference count 0");
    };
    assert_eq!("Standard_ProgramError", failure.exception_type());
    assert_eq!(
        "Attempt to create handle to object created in stack, not yet constructed, or destroyed",
        failure.message()
    );
    let handle = Handle::new(StandardTransient::default());
    assert!(handle.this().is_ok());
}
#[test]
fn reference_counter_sequence() {
    let object = StandardTransient::default();
    assert_eq!(0, object.ref_count());
    object.increment_ref_counter();
    object.increment_ref_counter();
    assert_eq!(1, object.decrement_ref_counter());
    assert_eq!(0, object.decrement_ref_counter());
    assert_eq!(-1, object.decrement_ref_counter());
}
#[test]
fn persistent_numbers_start_at_zero_and_are_shared() {
    let handle = Handle::new(StandardPersistent::default());
    assert_eq!(0, handle.type_num().load(Ordering::Relaxed));
    assert_eq!(0, handle.ref_num().load(Ordering::Relaxed));
    let copy = handle.clone();
    copy.type_num().store(7, Ordering::Relaxed);
    assert_eq!(7, handle.type_num().load(Ordering::Relaxed));
    assert_eq!(
        "Standard_Transient",
        handle.dynamic_type().unwrap().parent().unwrap().name()
    );
}
