//! Compiler-mapped RTTI descriptors for the two concrete TKernel base classes.
use rustycad_tkernel::standard::{
    Handle, RuntimeType, RuntimeTyped, StandardPersistent, Transient,
};

#[test]
fn standard_type_rtti_accessors_select_the_runtime_type_descriptor() {
    let descriptor = <RuntimeType as RuntimeTyped>::static_type();
    assert_eq!("Standard_Type", <RuntimeType as RuntimeTyped>::TYPE_NAME);
    assert_eq!("Standard_Type", descriptor.name());
    assert_eq!("Standard_Transient", descriptor.parent().unwrap().name());
    assert_eq!("Standard_Type", descriptor.dynamic_type().name());
}

#[test]
fn standard_persistent_rtti_accessors_select_the_persistent_descriptor() {
    let descriptor = <StandardPersistent as RuntimeTyped>::static_type();
    assert_eq!(
        "Standard_Persistent",
        <StandardPersistent as RuntimeTyped>::TYPE_NAME
    );
    assert_eq!("Standard_Persistent", descriptor.name());
    assert_eq!("Standard_Transient", descriptor.parent().unwrap().name());
}

#[test]
fn static_descriptor_owner_is_an_explicit_handle_copy() {
    assert_static_handle_copy::<RuntimeType>();
    assert_static_handle_copy::<StandardPersistent>();
}

fn assert_static_handle_copy<T: RuntimeTyped>() {
    let descriptor = T::static_type();
    let baseline = descriptor.transient_base().ref_count();
    let owner = T::static_type_handle();
    assert_eq!(baseline + 1, owner.ref_count());
    drop(owner);
    assert_eq!(baseline, descriptor.transient_base().ref_count());
}

#[test]
fn persistent_dynamic_descriptor_is_borrowed_and_can_be_owned_explicitly() {
    let object = Handle::new(StandardPersistent::default());
    let borrowed = object.get().unwrap().dynamic_type();
    assert!(std::ptr::eq(
        borrowed,
        <StandardPersistent as RuntimeTyped>::static_type()
    ));
    assert_eq!("Standard_Persistent", borrowed.name());
}

#[test]
fn persistent_dynamic_descriptor_handle_adds_and_releases_one_owner() {
    let object = Handle::new(StandardPersistent::default());
    let descriptor = <StandardPersistent as RuntimeTyped>::static_type();
    let baseline = descriptor.transient_base().ref_count();
    let owner = object.dynamic_type_handle().unwrap();
    assert_eq!(baseline + 1, owner.ref_count());
    drop(owner);
    assert_eq!(baseline, descriptor.transient_base().ref_count());
}
