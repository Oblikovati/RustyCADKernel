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

//! The complete FlatDataMap fixture, with empty upstream setup and teardown.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{FlatDataMap, HashPolicy};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::BTreeSet;
type StringMap = FlatDataMap<i32, AsciiString>;
fn strings(entries: &[(i32, &str)]) -> StringMap {
    let mut map = StringMap::default();
    for &(key, value) in entries {
        map.bind(key, AsciiString::new(value));
    }
    map
}
#[derive(Clone, Default)]
struct StatefulHasher {
    salt: i32,
}
impl HashPolicy<i32> for StatefulHasher {
    fn hash(&self, key: &i32) -> usize {
        (key + self.salt) as usize
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
#[derive(Clone, Default)]
struct ConstantHasher;
impl HashPolicy<i32> for ConstantHasher {
    fn hash(&self, _key: &i32) -> usize {
        0
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
include!("ncollection_flat_data_map/basic.rs");
include!("ncollection_flat_data_map/binding.rs");
include!("ncollection_flat_data_map/iteration.rs");
