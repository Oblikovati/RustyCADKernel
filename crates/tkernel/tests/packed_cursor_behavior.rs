//! The live packed-map cursor matches the snapshot iterator and skips keys removed meanwhile.
use rustycad_tkernel::{
    ncollection::{PackedMap, next_map_prime},
    standard::hash_bytes,
};
fn collect_with_cursor(map: &PackedMap<i32>) -> Vec<i32> {
    let (mut cursor, mut key) = map.cursor_first();
    let mut keys = Vec::new();
    while let Some(value) = key {
        keys.push(value);
        key = map.cursor_next(&mut cursor);
    }
    keys
}
#[test]
fn cursor_walks_in_native_iterator_order() {
    let mut map = PackedMap::<i32>::new(1);
    for value in [5, -3, 64, 31, 32, 1000, 7, -40] {
        map.add(value);
    }
    let expected: Vec<i32> = map.iter().collect();
    assert_eq!(collect_with_cursor(&map), expected);
    assert_eq!(PackedMap::<i32>::default().cursor_first().1, None);
}
#[test]
fn cursor_skips_removed_keys_of_the_current_block() {
    let mut map = PackedMap::<i32>::new(1);
    for value in [1, 2, 3, 40] {
        map.add(value);
    }
    let (mut cursor, first) = map.cursor_first();
    let order: Vec<i32> = map.iter().collect();
    assert_eq!(first, Some(order[0]));
    let removed = order[1];
    map.remove(removed);
    let rest: Vec<i32> = std::iter::from_fn(|| map.cursor_next(&mut cursor)).collect();
    let expected: Vec<i32> = order[1..]
        .iter()
        .copied()
        .filter(|v| *v != removed)
        .collect();
    assert_eq!(rest, expected);
}
#[test]
fn exposed_hash_helpers_match_native_values() {
    assert_eq!(next_map_prime(0), 101);
    assert_eq!(next_map_prime(101), 1009);
    assert_eq!(hash_bytes(&[]), hash_bytes(&[]));
    assert_ne!(
        hash_bytes(&1_i32.to_ne_bytes()),
        hash_bytes(&2_i32.to_ne_bytes())
    );
}
