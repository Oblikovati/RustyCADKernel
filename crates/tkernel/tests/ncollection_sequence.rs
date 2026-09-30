//! Sequence preserves one-based indices and stable allocator-owned elements.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::ncollection::{BaseAllocator, Sequence};
fn seq(values: &[i32]) -> Sequence<i32> {
    values.iter().copied().collect()
}
fn equal(sequence: &Sequence<i32>, expected: &[i32]) {
    assert_eq!(sequence.len(), expected.len());
    for (index, value) in expected.iter().enumerate() {
        assert_eq!(sequence[index + 1], *value);
    }
}
fn random_values() -> Vec<i32> {
    include_str!("fixtures/array1_mt19937_seed1.txt")
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect()
}
include!("ncollection_sequence/basic.rs");
include!("ncollection_sequence/ownership.rs");
include!("ncollection_sequence/emplacement.rs");
