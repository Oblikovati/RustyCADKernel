//! Faithful translations of NCollection_Array1_Test.cxx, grouped for review.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;

#[test]
fn NCollection_Array1Test__DefaultConstructor() {
    let array = Array1::<i32>::default();
    assert_eq!(0, array.len());
    assert!(array.is_empty());
    assert_eq!(1, array.lower());
    assert_eq!(0, array.upper());
}

include!("ncollection_array1/constructors.rs");
include!("ncollection_array1/assignment.rs");
include!("ncollection_array1/access.rs");
include!("ncollection_array1/size.rs");
include!("ncollection_array1/algorithms.rs");
