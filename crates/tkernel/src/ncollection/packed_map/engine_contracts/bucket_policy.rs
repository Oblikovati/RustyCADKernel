//! Exact native allocation-boundary traces; injected failure is not host heap exhaustion.
use super::{PackedInteger, PackedMap, PackedWord, pointers};
mod fixture_index;
use crate::ncollection::packed_map::storage::allocation::observation::capture;
use fixture_index::{NativeBucketRows, native_bucket_rows};
fn keys<T: PackedInteger>() -> [T; 9] {
    [0, 1, 31, 32, 63, 64, 3232, 6464, u64::MAX].map(T::from_bits)
}
fn initial<T: PackedInteger>(state: usize) -> PackedMap<T> {
    let mut map = PackedMap::new(if state == 4 { 700 } else { 1 });
    if state == 1 {
        map.resize(0);
    }
    if matches!(state, 2 | 3) {
        for key in keys::<T>() {
            map.add(key);
        }
    }
    if state == 3 {
        for key in keys::<T>() {
            map.remove(key);
        }
    }
    map
}
fn snapshot<T: PackedInteger>(map: &PackedMap<T>) -> String {
    let mut result = header(map);
    for bucket in 0..map.buckets.len() {
        for node in map.buckets.chain(bucket) {
            result.extend([
                bucket.to_string(),
                node.mask().bits().to_string(),
                node.bits().bits().to_string(),
            ]);
        }
    }
    result.join(",")
}
fn header<T: PackedInteger>(map: &PackedMap<T>) -> Vec<String> {
    let allocated = map.buckets.len() != 0;
    let aligned = !allocated || map.buckets.heads.as_ptr().addr().is_multiple_of(16);
    [
        map.bucket_count,
        map.count,
        map.block_count,
        usize::from(allocated),
        usize::from(aligned),
    ]
    .into_iter()
    .map(|value| value.to_string())
    .collect()
}
fn scenario<T: PackedInteger>(state: usize, request: usize, fail: bool) -> String {
    let mut map = initial::<T>(state);
    let before = snapshot(&map);
    let addresses = pointers(&map);
    let table = map.buckets.heads;
    let (result, events) = capture(fail, || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.resize(request)))
    });
    let retained = retained_nodes(&map, &addresses);
    format!(
        "{before}|{}|{}|{}|{}|{}",
        events.join(","),
        u8::from(result.is_err()),
        snapshot(&map),
        u8::from(retained),
        u8::from(table == map.buckets.heads)
    )
}
fn retained_nodes<T: PackedInteger>(
    map: &PackedMap<T>,
    before: &[*const super::PackedMapNode<T>],
) -> bool {
    let after = pointers(map);
    before.len() == after.len() && before.iter().all(|node| after.contains(node))
}
fn native_bucket_policy<T: PackedInteger>(name: &str) {
    let fixture = native_bucket_rows();
    for state in 0..5 {
        for (index, request) in [0, 1000, 1009, usize::MAX].into_iter().enumerate() {
            check_request::<T>(fixture, name, state, index, request);
        }
    }
}
fn check_request<T: PackedInteger>(
    fixture: &NativeBucketRows,
    name: &str,
    state: usize,
    index: usize,
    request: usize,
) {
    for fail in [false, true] {
        let key = format!("{name}:{state}:{index}:{}", u8::from(fail));
        let expected = fixture
            .get(key.as_str())
            .unwrap_or_else(|| panic!("native row {key}: expected a unique oracle fixture entry"));
        assert_eq!(scenario::<T>(state, request, fail), *expected, "{key}");
    }
}
macro_rules! widths {
    ($($name:ident: $ty:ty),+) => { $(#[test] fn $name() { native_bucket_policy::<$ty>(stringify!($ty)); })+ };
}
widths!(bucket_policy_i32: i32, bucket_policy_u32: u32, bucket_policy_i64: i64, bucket_policy_u64: u64, bucket_policy_i8: i8, bucket_policy_bool: bool);
