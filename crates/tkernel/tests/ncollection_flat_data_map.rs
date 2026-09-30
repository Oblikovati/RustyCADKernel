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
