//! Translated NCollection_LocalArray tests; source IDs remain unchanged.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::LocalArray;

#[test]
fn NCollection_LocalArrayTest__DefaultConstructor() {
    let array = LocalArray::<i32>::default();
    assert_eq!(0, array.len());
    let range = array.as_slice().as_ptr_range();
    assert_eq!(range.start, range.end);
    assert_eq!(array.iter().as_slice().as_ptr_range().start, range.end);
}
include!("ncollection_local_array/allocation.rs");
include!("ncollection_local_array/moves_views.rs");
include!("ncollection_local_array/lifetimes.rs");

// Compile-time equivalents of the source's HasToArray1/return-type checks.
// A const source maps to a Rust shared slice; mutable aliases require &mut.
const _: fn(&mut LocalArray<i32>) -> rustycad_tkernel::ncollection::Array1<'_, i32> =
    LocalArray::to_array1;
const _: fn(&LocalArray<i32>) -> &[i32] = LocalArray::as_slice;
const _: () = assert!(std::mem::needs_drop::<LocalTracked>());
