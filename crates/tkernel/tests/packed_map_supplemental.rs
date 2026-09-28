//! Native boundary quirks, allocation transitions and cursor equality.
use rustycad_tkernel::ncollection::{PackedMap, packed_map_algo};
#[test]
fn native_lowest_block_maximum_and_cardinality_quirks() {
    let mut lowest: PackedMap<i32> = PackedMap::default();
    lowest.add(i32::MIN + 1);
    assert_eq!(lowest.minimal(), i32::MIN + 1);
    assert_eq!(lowest.maximal(), i32::MIN);
    let mut zero: PackedMap<u32> = PackedMap::default();
    zero.add(7);
    assert_eq!(zero.minimal(), 7);
    assert_eq!(zero.maximal(), 0);
    let mut left: PackedMap<i32> = PackedMap::default();
    let mut right: PackedMap<i32> = PackedMap::default();
    for value in [1, 2] {
        left.add(value);
    }
    for value in [2, 3] {
        right.add(value);
    }
    assert!(!packed_map_algo::differ(&mut left, &right));
    assert_eq!(left.iter().collect::<Vec<i32>>(), [1, 3]);
}
#[test]
fn empty_limits_bucket_retention_and_rehash_order() {
    let mut map: PackedMap<i64> = PackedMap::new(0);
    assert_eq!(map.bucket_count(), 1);
    assert_eq!(map.minimal(), i64::MAX);
    assert_eq!(map.maximal(), i64::MIN);
    assert!(!map.remove(1));
    assert!(!map.contains(1));
    for value in [0, 64, 6464, i64::MAX, i64::MIN] {
        map.add(value);
    }
    assert_eq!(map.bucket_count(), 101);
    map.resize(1000);
    assert_eq!(map.bucket_count(), 1009);
    for value in [0, 64, 6464, i64::MAX, i64::MIN] {
        assert!(map.contains(value));
    }
    map.clear();
    assert_eq!(map.bucket_count(), 1009);
    map.add(42);
    let copy: PackedMap<i64> = map.clone();
    assert_eq!(copy.bucket_count(), 101);
    assert_eq!(copy.iter().collect::<Vec<i64>>(), [42]);
}
#[test]
fn cursors_compare_position_and_all_exhausted_ends() {
    let mut map: PackedMap<u32> = PackedMap::default();
    for value in [31, 32, 33] {
        map.add(value);
    }
    let mut first = map.iter();
    let second = first.clone();
    assert!(first == second);
    assert_eq!(first.key(), Some(&31));
    first.advance();
    assert!(first != second);
    assert_eq!(first.key(), Some(&32));
    first.reset();
    assert!(first == second);
    while first.more() {
        first.advance();
    }
    first.advance();
    assert!(first == PackedMap::<u32>::default().iter());
}
#[test]
fn in_place_empty_operands_preserve_native_results() {
    let mut left: PackedMap<i32> = PackedMap::default();
    let mut right: PackedMap<i32> = PackedMap::default();
    assert!(!packed_map_algo::unite(&mut left, &right));
    assert!(!packed_map_algo::intersect(&mut left, &right));
    assert!(!packed_map_algo::differ(&mut left, &right));
    right.add(5);
    assert!(packed_map_algo::unite(&mut left, &right));
    left.clear();
    assert!(packed_map_algo::differ(&mut left, &right));
    right.clear();
    assert!(packed_map_algo::intersect(&mut left, &right));
    assert!(left.is_empty());
}
