//! Bidirectional key lookup and duplicate rejection from the complete legacy suite.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{DoubleMap, HashKey};
use rustycad_tkernel::tcollection::AsciiString;
type NumericMap = DoubleMap<i32, f64>;
fn numeric(keys: impl IntoIterator<Item = i32>) -> NumericMap {
    let mut map = NumericMap::default();
    for key in keys {
        map.bind(key, key as f64 / 10.0).unwrap();
    }
    map
}
fn check_pairs(map: &NumericMap, keys: &[i32]) {
    assert_eq!(map.extent(), keys.len());
    for key in keys {
        let value = *key as f64 / 10.0;
        assert!(map.is_bound1(key));
        assert!(map.is_bound2(&value));
        assert_eq!(map.find1(key), Ok(&value));
        assert_eq!(map.find2(&value), Ok(key));
    }
}
#[derive(Debug, Clone, PartialEq)]
struct TestKey1 {
    id: i32,
    name: String,
}
impl TestKey1 {
    fn new(id: i32, name: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
        }
    }
}
impl HashKey for TestKey1 {
    fn occt_hash(&self) -> usize {
        let hash: u64 = match self.name.as_str() {
            "One" => 4974007186610801277,
            "Two" => 18140831775427969181,
            "Three" => 7219768605936779428,
            _ => unreachable!(),
        };
        (hash as usize).wrapping_add(self.id as usize)
    }
}
#[derive(Debug, Clone)]
struct TestKey2 {
    value: f64,
    code: String,
}
impl TestKey2 {
    fn new(value: f64, code: &str) -> Self {
        Self {
            value,
            code: code.to_owned(),
        }
    }
}
impl PartialEq for TestKey2 {
    fn eq(&self, other: &Self) -> bool {
        (self.value - other.value).abs() < 1e-10 && self.code == other.code
    }
}
impl HashKey for TestKey2 {
    fn occt_hash(&self) -> usize {
        let hash: u64 = match self.code.as_str() {
            "A" => 6919333181322027406,
            "B" => 4199750530058918380,
            "C" => 12432341034569643010,
            _ => unreachable!(),
        };
        // Preserve the C++ conversion to double before addition and truncation.
        (self.value * 100.0 + hash as f64) as usize
    }
}
include!("ncollection_double_map/basic.rs");
include!("ncollection_double_map/binding.rs");
