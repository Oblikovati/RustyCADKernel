// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Parallel numerical regressions, preserving sizes and arithmetic order.
// Preserve the upstream multiplication-plus-addition operand order.
#![allow(non_snake_case, clippy::assign_op_pattern)]
use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    osd::{Parallel, ThreadPool},
};
fn saxpy(index: usize, value: &mut f64, x: &Array1<'_, f64>) {
    *value = 1e-6 * x.as_slice()[index] + *value;
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
    let mut reference = initial_vector();
    for (index, value) in reference.iter_mut().enumerate() {
        saxpy(index, value, &x);
    }
    let mut result = x.clone();
    Parallel::for_each_mut(result.as_mut_slice(), |index, value| {
        saxpy(index, value, &x)
    });
    assert_eq!(result.as_slice(), reference.as_slice());
    let pool = ThreadPool::default_pool();
    let launcher = pool.launcher();
    result = initial_vector();
    launcher.perform_mut(result.as_mut_slice(), |_worker, index, value| {
        saxpy(index, value, &x)
    });
    assert_eq!(result.as_slice(), reference.as_slice());
    result = initial_vector();
    Parallel::for_chunks_mut(result.as_mut_slice(), 100000, |batch, chunk| {
        for (offset, value) in chunk.iter_mut().enumerate() {
            saxpy(batch * 100000 + offset, value, &x);
        }
    });
    assert_eq!(result.as_slice(), reference.as_slice());
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
fn multiply_row(index: usize, row: &mut [f64], first: &Array2<'_, f64>, second: &Array2<'_, f64>) {
    for (col, value) in row.iter_mut().enumerate() {
        let mut sum = 0.0;
        for inner in 0..50 {
            sum += first[(index as i32, inner)] * second[(inner, col as i32)];
        }
        *value = sum;
    }
}
#[test]
fn OSD_ParallelTest__OCC29935_MatrixMultiplyParallelMatchesSequential() {
    let (first, second) = matrix_inputs();
    let mut reference = Array2::with_size(50, 50);
    let mut result = Array2::with_size(50, 50);
    for (index, row) in reference.as_mut_slice().chunks_mut(50).enumerate() {
        multiply_row(index, row, &first, &second);
    }
    result.init(0.0);
    Parallel::for_chunks_mut(result.as_mut_slice(), 50, |index, row| {
        multiply_row(index, row, &first, &second)
    });
    assert_eq!(result.as_slice(), reference.as_slice());
    result.init(0.0);
    ThreadPool::default_pool().launcher().perform_chunks_mut(
        result.as_mut_slice(),
        50,
        |_worker, index, row| multiply_row(index, row, &first, &second),
    );
    assert_eq!(result.as_slice(), reference.as_slice());
}
