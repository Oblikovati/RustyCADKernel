//! Faithful List scenarios, with detached cursors for mutation during iteration.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::ncollection::{BaseAllocator, List};
fn list(values: &[i32]) -> List<i32> {
    values.iter().copied().collect()
}
fn equal(list: &List<i32>, expected: &[i32]) {
    assert_eq!(list.len(), expected.len());
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), expected);
}
fn random_values() -> Vec<i32> {
    include_str!("fixtures/array1_mt19937_seed1.txt")
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect()
}
include!("ncollection_list/basic.rs");
include!("ncollection_list/ownership.rs");
include!("ncollection_list/emplacement.rs");
