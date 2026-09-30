//! Legacy segmented-vector scenarios with stable detached cursor positions.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray};
fn values(input: &[i32]) -> DynamicArray<i32> {
    let mut array = DynamicArray::default();
    for &value in input {
        array.append(value);
    }
    array
}
fn equal(array: &DynamicArray<i32>, expected: &[i32]) {
    assert_eq!(array.len(), expected.len());
    assert_eq!(array.iter().copied().collect::<Vec<_>>(), expected);
}
#[derive(Default)]
struct MultiArg {
    integer: i32,
    real: f64,
}
#[derive(Default)]
struct MoveOnly(i32);
#[derive(Clone, Default)]
struct ConstructOnly(i32);
fn check_pair(value: &MultiArg, integer: i32, real: f64) {
    assert_eq!(value.integer, integer);
    assert!((value.real - real).abs() <= 1e-10);
}
include!("ncollection_dynamic_array/basic.rs");
include!("ncollection_dynamic_array/iteration.rs");
include!("ncollection_dynamic_array/emplacement.rs");
include!("ncollection_dynamic_array/insertion.rs");
include!("ncollection_dynamic_array/indices.rs");
