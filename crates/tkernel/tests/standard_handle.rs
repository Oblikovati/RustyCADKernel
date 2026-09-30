//! OCC24533 pointer observation for null and live handles.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::{Handle, StandardTransient};
#[test]
fn Standard_Handle_Test__OCC24533_IsNullAndPointerChecks() {
    let mut handle = Handle::<StandardTransient>::default();
    assert!(handle.is_null());
    let pointer = handle.as_ptr();
    assert!(pointer.is_null());
    assert_eq!(std::ptr::null(), pointer);
    handle = Handle::new(StandardTransient::default());
    assert!(!handle.is_null());
    let pointer = handle.as_ptr();
    assert!(!pointer.is_null());
    assert_ne!(std::ptr::null(), pointer);
}
