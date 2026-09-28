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

//! Faithful IndexedDataMap declarations; exclusive references are reborrowed after insertion.
#![allow(non_snake_case, clippy::unnecessary_mut_passed)]
use rustycad_tkernel::ncollection::{HashKey, HashPolicy, IndexedDataMap};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::BTreeSet;
type NumericMap = IndexedDataMap<i32, f64>;
fn numeric(keys: impl IntoIterator<Item = i32>) -> NumericMap {
    let mut map = NumericMap::default();
    for key in keys {
        map.add(key, key as f64 / 10.0);
    }
    map
}
fn check_numeric(map: &NumericMap, keys: &[i32]) {
    assert_eq!(map.extent(), keys.len());
    for (index, key) in keys.iter().enumerate() {
        assert!(map.contains(key));
        assert_eq!(map.find_index(key), index + 1);
        assert_eq!(map.find_key(index + 1), Ok(key));
        assert_eq!(map.find_from_key(key), Ok(&(*key as f64 / 10.0)));
        assert_eq!(map.find_from_index(index + 1), Ok(&(*key as f64 / 10.0)));
        assert_eq!(map[index + 1], *key as f64 / 10.0);
    }
}
fn check_values(map: &NumericMap, values: &[f64]) {
    for (index, value) in values.iter().enumerate() {
        assert_eq!(map.find_from_index(index + 1), Ok(value));
        assert_eq!(map.find_from_key(&((index as i32 + 1) * 10)), Ok(value));
    }
}
fn strings(values: &[(i32, &str)]) -> IndexedDataMap<i32, AsciiString> {
    let mut map = IndexedDataMap::default();
    for &(key, value) in values {
        map.add(key, AsciiString::new(value));
    }
    map
}
#[derive(Clone)]
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
#[derive(Clone, Debug, PartialEq)]
struct TestKey {
    id: i32,
    name: String,
}
impl TestKey {
    fn new(id: i32, name: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
        }
    }
}
#[derive(Clone, Default)]
struct TestKeyHasher;
impl HashPolicy<TestKey> for TestKeyHasher {
    fn hash(&self, key: &TestKey) -> usize {
        // OCCT combines std::hash<int> and std::hash<string>; this fixture
        // captures libstdc++ string hashes for the four upstream keys.
        let name_hash: u64 = match key.name.as_str() {
            "One" => 4974007186610801277,
            "Two" => 18140831775427969181,
            "Three" => 7219768605936779428,
            "Four" => 6500357484398430375,
            _ => unreachable!(),
        };
        let words = [key.id as usize, name_hash as usize];
        let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_ne_bytes()).collect();
        AsciiString::from_raw(&bytes).hash_code()
    }
    fn equal(&self, left: &TestKey, right: &TestKey) -> bool {
        left == right
    }
}

impl HashKey for TestKey {
    fn occt_hash(&self) -> usize {
        TestKeyHasher.hash(self)
    }
}
#[derive(Debug, Clone)]
struct TestItem {
    value: f64,
    description: String,
}
impl TestItem {
    fn new(value: f64, description: &str) -> Self {
        Self {
            value,
            description: description.to_owned(),
        }
    }
}
impl PartialEq for TestItem {
    fn eq(&self, other: &Self) -> bool {
        (self.value - other.value).abs() < 1e-10 && self.description == other.description
    }
}
include!("ncollection_indexed_data_map/basic.rs");
include!("ncollection_indexed_data_map/binding.rs");
include!("ncollection_indexed_data_map/iteration.rs");
