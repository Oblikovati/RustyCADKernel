//! Shared exact-row decoder for PackedMap native fixtures.
use std::collections::BTreeMap;

pub type NativeRows = BTreeMap<&'static str, Vec<u64>>;

pub fn decode_native_rows(fixture: &'static str) -> NativeRows {
    fixture.lines().map(decode_native_row).collect()
}

fn decode_native_row(line: &'static str) -> (&'static str, Vec<u64>) {
    let (label, values) = line.split_once(' ').unwrap();
    let columns = values
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect();
    (label, columns)
}
