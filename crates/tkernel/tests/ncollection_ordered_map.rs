//! Insertion-order and stable-key assertions from the complete OrderedMap suite.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{HashPolicy, OrderedMap, OrderedMapIter};
use rustycad_tkernel::tcollection::AsciiString;
fn integers(keys: impl IntoIterator<Item = i32>) -> OrderedMap<i32> {
    let mut map = OrderedMap::default();
    for key in keys {
        map.add(key);
    }
    map
}
fn collect_keys<H: HashPolicy<i32>>(map: &OrderedMap<i32, H>) -> Vec<i32> {
    map.iter().copied().collect()
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
include!("ncollection_ordered_map/basic.rs");
include!("ncollection_ordered_map/iteration.rs");
