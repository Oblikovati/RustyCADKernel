//! Decode once outside allocation observation; every scenario still checks its original row.
use std::{collections::BTreeMap, sync::OnceLock};
pub(super) type NativeBucketRows = BTreeMap<&'static str, &'static str>;
const NATIVE_ROWS: &str =
    include_str!("../../../../../../../tools/oracle/tkernel_packed_map_bucket_policy.expected");

pub(super) fn native_bucket_rows() -> &'static NativeBucketRows {
    static ROWS: OnceLock<NativeBucketRows> = OnceLock::new();
    ROWS.get_or_init(|| index_native_rows(NATIVE_ROWS))
}
fn index_native_rows(fixture: &'static str) -> NativeBucketRows {
    let mut rows = BTreeMap::new();
    for line in fixture.lines() {
        let (key, expected) = line
            .split_once(' ')
            .unwrap_or_else(|| panic!("native row {line:?}: expected key, space, and payload"));
        assert!(
            rows.insert(key, expected).is_none(),
            "native key {key:?}: expected one row"
        );
    }
    rows
}
#[test]
fn indexed_rows_preserve_all_240_native_payloads_and_reuse_one_table() {
    let rows = native_bucket_rows();
    assert_eq!(rows.len(), 240);
    assert!(std::ptr::eq(rows, native_bucket_rows()));
    for line in NATIVE_ROWS.lines() {
        let (key, expected) = line.split_once(' ').unwrap();
        assert_eq!(rows.get(key).copied(), Some(expected), "{key}");
    }
}
#[test]
#[should_panic(expected = "native key \"duplicate\": expected one row")]
fn duplicate_native_fixture_keys_are_rejected() {
    index_native_rows("duplicate first\nduplicate second");
}
#[test]
#[should_panic(expected = "native row \"missing-payload\": expected key, space, and payload")]
fn malformed_native_fixture_rows_are_rejected() {
    index_native_rows("missing-payload");
}
