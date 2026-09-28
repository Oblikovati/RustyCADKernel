// Copyright (c) 2025 OPEN CASCADE SAS
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
