//! Complete native Handle_Operations suite with safe projected base views.
#![allow(non_snake_case)]
include!("handles/support.rs");
use rustycad_tkernel::ncollection::{
    AllocatorObject, HeapAllocatorObject, IncrementalAllocatorObject,
};
#[test]
fn HandleOperationsTest__BasicHandleOperations() {
    let derived = Handle::new(TestDerived::default());
    assert!(!derived.is_null());
    let const_derived = &derived;
    let _const_base = derived.view::<TestBase>();
    let derived_ptr = derived.as_ptr();
    let const_ptr = const_derived.as_ptr();
    assert!(!derived_ptr.is_null());
    assert!(!const_ptr.is_null());
    assert_eq!(derived_ptr, const_ptr);
    assert_eq!(&*derived as *const TestDerived, derived_ptr);
    assert_eq!(&**const_derived as *const TestDerived, const_ptr);
    let base = derived.cast::<TestBase>();
    assert!(!base.is_null());
    assert_eq!(base.as_ptr().cast::<()>(), derived.as_ptr().cast::<()>());
}
#[test]
fn HandleOperationsTest__HandleDownCast() {
    let derived = Handle::new(TestDerived::default());
    let base = derived.cast::<TestBase>();
    let down_cast = base.cast::<TestDerived>();
    assert!(!down_cast.is_null());
    assert_eq!(down_cast.as_ptr(), derived.as_ptr());
    let other = Handle::new(TestOther::default());
    let transient = other.cast::<StandardTransient>();
    assert!(transient.cast::<TestDerived>().is_null());
}
#[test]
fn HandleOperationsTest__HandleComparisons() {
    let derived = Handle::new(TestDerived::default());
    let derived2 = derived.clone();
    let derived3 = Handle::new(TestDerived::default());
    let base = derived.cast::<TestBase>();
    assert!(derived == derived.clone());
    assert!(derived == derived2);
    assert!(derived == base);
    assert!(derived != derived3);
    let pointer = derived.as_ptr();
    let const_pointer = derived.as_ptr();
    assert!(derived == pointer);
    assert!(pointer == derived);
    assert!(derived == const_pointer);
    assert!(const_pointer == derived);
    assert!(!(derived != derived2));
    assert!(derived != derived3);
    assert!(derived.is_some());
    assert!(!Handle::<TestDerived>::default().is_some());
}
#[test]
fn HandleOperationsTest__AllocatorHandleCasting() {
    let base_ptr = Handle::new(IncrementalAllocatorObject::default()).cast::<AllocatorObject>();
    assert!(!base_ptr.is_null());
    let incremental = base_ptr.cast::<IncrementalAllocatorObject>();
    assert!(!incremental.is_null());
    let base = incremental.cast::<AllocatorObject>();
    assert!(!base.is_null());
    assert_eq!(base.as_ptr(), base_ptr.as_ptr());
    assert!(base_ptr.cast::<HeapAllocatorObject>().is_null());
}
#[test]
fn HandleOperationsTest__HandleCopyAndAssignment() {
    let derived1 = Handle::new(TestDerived::default());
    let derived2 = derived1.clone();
    assert!(!derived2.is_null());
    assert_eq!(derived1.as_ptr(), derived2.as_ptr());
    let mut derived3 = Handle::<TestDerived>::default();
    assert!(derived3.is_null());
    derived3.clone_from(&derived1);
    assert!(!derived3.is_null());
    assert_eq!(derived1.as_ptr(), derived3.as_ptr());
    // Rust self-move preserves ownership without a second mutable alias.
    let derived1 = derived1;
    assert!(!derived1.is_null());
    derived3 = Handle::default();
    assert!(derived3.is_null());
}
#[test]
fn HandleOperationsTest__HandleWithDifferentTypes() {
    let derived = Handle::new(TestDerived::default());
    let other = Handle::new(TestOther::default());
    let derived_transient = derived.cast::<StandardTransient>();
    let other_transient = other.cast::<StandardTransient>();
    assert!(!(derived_transient == other_transient));
    assert!(derived_transient != other_transient);
    assert!(derived_transient == derived);
    assert!(other_transient == other);
    assert_eq!(
        derived_transient.as_ptr().cast::<()>(),
        derived.as_ptr().cast::<()>()
    );
    assert_eq!(
        other_transient.as_ptr().cast::<()>(),
        other.as_ptr().cast::<()>()
    );
}
#[test]
fn HandleOperationsTest__HandleNullOperations() {
    let null = Handle::<TestDerived>::default();
    let another = Handle::<TestDerived>::default();
    let valid = Handle::new(TestDerived::default());
    assert!(null.is_null());
    assert!(null.as_ptr().is_null());
    assert!(null == another);
    assert!(!(null == valid));
    assert!(!(valid == null));
    assert!(!(null != another));
    assert!(null != valid);
    assert!(valid != null);
    assert!(!null.is_some());
    assert!(valid.is_some());
}
