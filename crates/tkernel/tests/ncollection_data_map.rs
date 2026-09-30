//! DataMap assertions retain replacement, nonreplacement and hasher-state contracts.
#![allow(non_snake_case)]
use rustycad_tkernel::{
    ncollection::{DataMap, HashPolicy},
    tcollection::AsciiString,
};
use std::collections::BTreeSet;
type StringMap = DataMap<i32, AsciiString>;
fn string_map(entries: &[(i32, &str)]) -> StringMap {
    let mut map = StringMap::default();
    for &(key, value) in entries {
        map.bind(key, AsciiString::new(value));
    }
    map
}
fn check(value: &AsciiString, expected: &str) {
    assert_eq!(value.as_bytes(), expected.as_bytes());
}
#[derive(Clone)]
struct StatefulHasher {
    salt: i32,
}
impl HashPolicy<i32> for StatefulHasher {
    fn hash(&self, key: &i32) -> usize {
        (*key + self.salt) as usize
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        first == second
    }
}
include!("ncollection_data_map/basic.rs");
include!("ncollection_data_map/emplacement.rs");
include!("ncollection_data_map/iteration.rs");
