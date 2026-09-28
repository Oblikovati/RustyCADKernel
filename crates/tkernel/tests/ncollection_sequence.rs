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
