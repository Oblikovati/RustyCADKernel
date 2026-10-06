//! Native storage observations, with private access limited to this contract suite.
use super::{PackedInteger, PackedMap, PackedMapNode, PackedWord};
use std::collections::BTreeMap;
#[path = "../../../tests/packed_map_oracle/mod.rs"]
mod decoder;
mod unwind;
fn fixture() -> BTreeMap<&'static str, Vec<u64>> {
    decoder::decode_native_rows(include_str!(
        "../../../../../tools/oracle/tkernel_packed_map_engine.expected"
    ))
}
fn snapshot<T: PackedInteger>(map: &PackedMap<T>) -> Vec<u64> {
    let mut result = vec![
        map.bucket_count as u64,
        map.count as u64,
        map.block_count as u64,
        u64::from(map.buckets.len() != 0),
    ];
    for bucket in 0..map.buckets.len() {
        for node in map.buckets.chain(bucket) {
            result.extend([bucket as u64, node.mask().bits(), node.bits().bits()]);
        }
    }
    result
}
fn pointers<T: PackedInteger>(map: &PackedMap<T>) -> Vec<*const PackedMapNode<T>> {
    map.buckets.nodes().map(std::ptr::from_ref).collect()
}
fn keys<T: PackedInteger>() -> [T; 8] {
    [
        0,
        1,
        PackedMap::<T>::BITS_PER_BLOCK as u64 - 1,
        101 * PackedMap::<T>::BITS_PER_BLOCK as u64,
        202 * PackedMap::<T>::BITS_PER_BLOCK as u64,
        u64::MAX,
        T::MIN.bits(),
        T::MAX.bits(),
    ]
    .map(T::from_bits)
}
fn native_engine<T: PackedInteger>(name: &str) {
    let rows = fixture();
    let check = |step: &str, map: &PackedMap<T>| {
        assert_eq!(
            snapshot(map),
            rows[format!("{name}:{step}").as_str()],
            "{name}:{step}"
        )
    };
    let mut map = PackedMap::<T>::default();
    check("default", &map);
    for key in keys::<T>() {
        map.add(key);
    }
    check("added", &map);
    let original = pointers(&map);
    map.resize(1000);
    let resized = pointers(&map);
    assert_eq!(
        [
            u64::from(resized.iter().all(|p| original.contains(p))),
            u64::from(original.len() == resized.len())
        ]
        .as_slice(),
        rows[format!("{name}:rehash_identity").as_str()]
    );
    check("resized", &map);
    native_copy_move(name, &rows, map, &original);
}
fn native_copy_move<T: PackedInteger>(
    name: &str,
    rows: &BTreeMap<&str, Vec<u64>>,
    mut map: PackedMap<T>,
    original: &[*const PackedMapNode<T>],
) {
    let check = |step: &str, map: &PackedMap<T>| {
        assert_eq!(
            snapshot(map),
            rows[format!("{name}:{step}").as_str()],
            "{name}:{step}"
        )
    };
    let copied = map.clone();
    check("copied", &copied);
    let separate = pointers(&copied).iter().all(|p| !original.contains(p));
    let table = map.buckets.heads;
    let mut moved = PackedMap::move_from(&mut map);
    assert_eq!(
        [u64::from(separate), u64::from(moved.buckets.heads == table)].as_slice(),
        rows[format!("{name}:ownership").as_str()]
    );
    check("moved", &moved);
    check("source", &map);
    for key in keys::<T>() {
        moved.remove(key);
    }
    check("removed", &moved);
    moved.clear();
    check("cleared", &moved);
    moved.resize(0);
    check("empty_resize", &moved);
}
macro_rules! widths {
    ($($name:ident: $ty:ty),+) => { $(#[test] fn $name() { native_engine::<$ty>(stringify!($ty)); })+ };
}
widths!(engine_i32: i32, engine_u32: u32, engine_i64: i64, engine_u64: u64, engine_i8: i8, engine_bool: bool);
#[test]
fn engine_map_and_iterator_preserve_thread_traits() {
    fn traits<T: Send + Sync>() {}
    traits::<PackedMap<i32>>();
    traits::<super::PackedMapIter<'static, i32>>();
    traits::<PackedMap<u64>>();
    traits::<super::PackedMapIter<'static, u64>>();
}

#[test]
fn engine_borrowed_iterator_lifetime_is_covariant() {
    fn shorten<'long: 'short, 'short>(
        iter: super::PackedMapIter<'long, u64>,
    ) -> super::PackedMapIter<'short, u64> {
        iter
    }
    let mut map = PackedMap::<u64>::default();
    map.add(42);
    assert_eq!(shorten(map.iter()).next(), Some(42));
}
