//! Parallel numerical regressions, preserving sizes and arithmetic order.
//!
//! The native functors write disjoint elements of a shared array through a const
//! reference; the Rust translation keeps the index-based `For`/`Launcher::Perform` calls
//! and gives each element an atomic cell so that the shared writes stay safe.
// Preserve the upstream multiplication-plus-addition operand order.
#![allow(non_snake_case, clippy::assign_op_pattern)]
use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    osd::{Launcher, Parallel, ThreadPool},
};
use std::sync::atomic::{AtomicU64, Ordering};

/// Shared f64 slots written by disjoint indices (the native `NCollection_Array&` member).
struct SharedReals(Vec<AtomicU64>);
impl SharedReals {
    fn from_slice(values: &[f64]) -> Self {
        Self(values.iter().map(|v| AtomicU64::new(v.to_bits())).collect())
    }
    fn get(&self, index: usize) -> f64 {
        f64::from_bits(self.0[index].load(Ordering::Relaxed))
    }
    fn set(&self, index: usize, value: f64) {
        self.0[index].store(value.to_bits(), Ordering::Relaxed);
    }
    fn to_vec(&self) -> Vec<f64> {
        (0..self.0.len()).map(|index| self.get(index)).collect()
    }
}
const THE_BATCH_SIZE: usize = 100000;
fn saxpy(index: usize, y: &SharedReals, x: &Array1<'_, f64>) {
    y.set(index, 1e-6 * x.as_slice()[index] + y.get(index));
}
fn saxpy_batch(batch: usize, y: &SharedReals, x: &Array1<'_, f64>) {
    let lower = batch * THE_BATCH_SIZE;
    let upper = (lower + THE_BATCH_SIZE - 1).min(x.len() - 1);
    for index in lower..=upper {
        saxpy(index, y, x);
    }
}
fn initial_vector() -> Array1<'static, f64> {
    let mut vector = Array1::with_size(500000);
    for (index, value) in vector.iter_mut().enumerate() {
        *value = index as f64;
    }
    vector
}
#[test]
fn OSD_ParallelTest__OCC24826_SaxpyParallelMatchesSequential() {
    let x = initial_vector();
    let reference = SharedReals::from_slice(initial_vector().as_slice());
    for index in 0..x.len() {
        saxpy(index, &reference, &x);
    }
    let length = x.len() as i32;
    let y = SharedReals::from_slice(initial_vector().as_slice());
    Parallel::for_range(0, length, |index| saxpy(index as usize, &y, &x), false).unwrap();
    assert_eq!(y.to_vec(), reference.to_vec());
    let y = SharedReals::from_slice(initial_vector().as_slice());
    let launcher = Launcher::new(ThreadPool::default_pool(-1), -1);
    launcher
        .perform(0, length, |_thread, index| saxpy(index as usize, &y, &x))
        .unwrap();
    assert_eq!(y.to_vec(), reference.to_vec());
    let y = SharedReals::from_slice(initial_vector().as_slice());
    let batches = (x.len() as f64 / THE_BATCH_SIZE as f64).ceil() as i32;
    Parallel::for_range(
        0,
        batches,
        |batch| saxpy_batch(batch as usize, &y, &x),
        false,
    )
    .unwrap();
    assert_eq!(y.to_vec(), reference.to_vec());
}
fn matrix_inputs() -> (Array2<'static, f64>, Array2<'static, f64>) {
    let sequence: Vec<f64> = include_str!("fixtures/parallel_mt19937_seed42.txt")
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect();
    assert_eq!(sequence.len(), 5000);
    let mut first = Array2::with_size(50, 50);
    let mut second = Array2::with_size(50, 50);
    for index in 0..2500 {
        first.as_mut_slice()[index] = sequence[2 * index];
        second.as_mut_slice()[index] = sequence[2 * index + 1];
    }
    (first, second)
}
fn multiply_row(
    index: usize,
    result: &SharedReals,
    first: &Array2<'_, f64>,
    second: &Array2<'_, f64>,
) {
    for col in 0..50 {
        let mut sum = 0.0;
        for inner in 0..50 {
            sum += first[(index as i32, inner)] * second[(inner, col as i32)];
        }
        result.set(index * 50 + col, sum);
    }
}
#[test]
fn OSD_ParallelTest__OCC29935_MatrixMultiplyParallelMatchesSequential() {
    let (first, second) = matrix_inputs();
    let reference = SharedReals::from_slice(&[0.0; 2500]);
    for index in 0..50 {
        multiply_row(index, &reference, &first, &second);
    }
    let result = SharedReals::from_slice(&[0.0; 2500]);
    let row = |index: i32| multiply_row(index as usize, &result, &first, &second);
    Parallel::for_range(0, 50, row, false).unwrap();
    assert_eq!(result.to_vec(), reference.to_vec());
    let result = SharedReals::from_slice(&[0.0; 2500]);
    let launcher = Launcher::new(ThreadPool::default_pool(-1), -1);
    launcher
        .perform(0, 50, |_thread, index| {
            multiply_row(index as usize, &result, &first, &second)
        })
        .unwrap();
    assert_eq!(result.to_vec(), reference.to_vec());
}
