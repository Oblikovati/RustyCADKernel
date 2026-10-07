const CHANGE_COUNTS: [usize; 7] = [
    1,
    2_147_483_647,
    2_147_483_648,
    4_294_967_295,
    4_294_967_296,
    4_294_967_297,
    usize::MAX - 16,
];
use super::{PackedInteger, PackedMap, binary, fixture, profile, snapshot, unary};
const EXTENTS: [usize; 9] = [
    0,
    1,
    2,
    2_147_483_647,
    2_147_483_648,
    4_294_967_295,
    4_294_967_296,
    4_294_967_297,
    usize::MAX,
];
fn reconstructed<T: PackedInteger>(name: &str, a: usize, shape: usize, mode: usize) {
    let mut left: PackedMap<T> = PackedMap::default();
    left.add(T::from_bits(0));
    let right: PackedMap<T> = profile([0, 2, 4][shape]);
    left.count = EXTENTS[a];
    let result: i32 = unary(2, mode != 0, &mut left, &right);
    let key: String = format!("{name}:synthetic_intersect:{a}:{shape}:{mode}");
    assert_eq!(snapshot(result, &left), fixture()[key.as_str()], "{key}");
}
fn selected<T: PackedInteger>(name: &str, a: usize, b: usize, mode: usize) {
    let mut left: PackedMap<T> = profile(2);
    let mut right: PackedMap<T> = profile(3);
    let mut destination: PackedMap<T> = profile(4);
    let saved: (usize, usize) = (left.count, right.count);
    (left.count, right.count) = (EXTENTS[a], EXTENTS[b]);
    let result: i32 = binary(3, mode != 0, &mut destination, &left, &right);
    (left.count, right.count) = saved;
    let key: String = format!("{name}:synthetic_intersection:{a}:{b}:{mode}");
    assert_eq!(
        snapshot(result, &destination),
        fixture()[key.as_str()],
        "{key}"
    );
}
fn changed_extent<T: PackedInteger>(name: &str, op: usize, a: usize, mode: usize) {
    let mut map: PackedMap<T> = PackedMap::default();
    let mut other: PackedMap<T> = PackedMap::default();
    map.add(T::from_bits(0));
    other.add(T::from_bits(if op == 0 { 33 } else { 0 }));
    if op == 6 {
        other.add(T::from_bits(33));
    }
    map.count = CHANGE_COUNTS[a];
    let result: i32 = unary(op, mode != 0, &mut map, &other);
    let observed: Vec<i128> = snapshot(result, &map);
    map.count = map.iter().count();
    let key: String = format!("{name}:synthetic_changes:{op}:{a}:{mode}");
    assert_eq!(observed, fixture()[key.as_str()], "{key}");
}
fn changed_rows<T: PackedInteger>(name: &str) {
    for op in [0, 4, 6] {
        for case in 0..14 {
            changed_extent::<T>(name, op, case / 2, case % 2);
        }
    }
}
fn synthetic<T: PackedInteger>(name: &str) {
    for a in 0..EXTENTS.len() {
        for case in 0..6 {
            reconstructed::<T>(name, a, case / 2, case % 2);
        }
        for case in 0..EXTENTS.len() * 2 {
            selected::<T>(name, a, case / 2, case % 2);
        }
    }
}
#[test]
fn synthetic_i64() {
    synthetic::<i64>("i64");
    changed_rows::<i64>("i64");
}
#[test]
fn synthetic_u64() {
    synthetic::<u64>("u64");
    changed_rows::<u64>("u64");
}
