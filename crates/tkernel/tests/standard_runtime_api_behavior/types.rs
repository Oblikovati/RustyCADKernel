//! Standard_Type registry, parent chain, SubType and Print.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{AllocatorObject, IncrementalAllocatorObject};
use rustycad_tkernel::standard::{
    RuntimeType, RuntimeTyped, StandardPersistent, StandardTransient, Transient, occ,
};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;

#[derive(Default)]
struct RegistryProbe {
    parent: StandardTransient,
}
impl_transient!(RegistryProbe, "RegistryProbe", StandardTransient, parent);

#[test]
fn register_returns_the_first_descriptor_of_a_class() {
    let first = RegistryProbe::static_type();
    let again = RuntimeType::register::<RegistryProbe>("Other", 999, None);
    assert!(std::ptr::eq(first, again));
    assert_eq!("RegistryProbe", again.name());
    assert_eq!(std::mem::size_of::<RegistryProbe>(), again.size());
    assert!(std::ptr::eq(
        StandardTransient::static_type(),
        again.parent().unwrap()
    ));
}
#[test]
fn register_copies_the_name_up_to_its_first_nul() {
    struct NulNamed;
    let descriptor = RuntimeType::register::<NulNamed>("Head\0Tail", 3, None);
    assert_eq!("Head", descriptor.name());
    assert_eq!(3, descriptor.size());
    assert!(descriptor.parent().is_none());
    assert!(descriptor.system_name().ends_with("NulNamed"));
}
#[test]
fn instance_type_of_and_type_name_agree() {
    assert!(std::ptr::eq(
        RuntimeType::instance::<StandardPersistent>(),
        occ::type_of::<StandardPersistent>()
    ));
    assert_eq!("Standard_Persistent", StandardPersistent::TYPE_NAME);
    assert_eq!("Standard_Transient", StandardTransient::TYPE_NAME);
    assert_eq!(
        "TColStd_HPackedMapOfInteger",
        HPackedMapOfInteger::static_type().name()
    );
}
#[test]
fn descriptors_are_transients_of_type_standard_type() {
    let descriptor = StandardTransient::static_type();
    assert_eq!("Standard_Type", descriptor.dynamic_type().name());
    assert_eq!(
        "Standard_Transient",
        RuntimeType::static_type().parent().unwrap().name()
    );
    assert!(descriptor.is_kind_name("Standard_Transient"));
}
#[test]
fn sub_type_walks_the_parent_chain() {
    let incremental = IncrementalAllocatorObject::static_type();
    assert!(incremental.is_subtype(incremental));
    assert!(incremental.is_subtype(AllocatorObject::static_type()));
    assert!(incremental.is_subtype(StandardTransient::static_type()));
    assert!(!AllocatorObject::static_type().is_subtype(incremental));
    assert!(incremental.is_subtype_name("NCollection_BaseAllocator"));
    assert!(incremental.is_subtype_name("Standard_Transient\0junk"));
    assert!(!incremental.is_subtype_name(""));
}
#[test]
fn sub_type_stops_at_an_ancestor_smaller_than_the_base() {
    // A (fake) base registered larger than its derived class is never reached, as C++.
    struct HugeBase;
    struct SmallDerived;
    let huge = RuntimeType::register::<HugeBase>("HugeBase", 1 << 20, None);
    let small = RuntimeType::register::<SmallDerived>("SmallDerived", 8, Some(huge));
    assert!(!small.is_subtype(huge));
    assert!(small.is_subtype_name("HugeBase"));
}
#[test]
fn print_writes_address_and_name() {
    let descriptor = StandardTransient::static_type();
    let mut printed: Vec<u8> = Vec::new();
    descriptor.print(&mut printed).unwrap();
    let text = String::from_utf8(printed).unwrap();
    let expected = format!("{:p} : Standard_Transient", descriptor);
    assert_eq!(expected, text);
    assert_eq!(expected, descriptor.to_string());
    assert!(text.starts_with("0x"));
}
