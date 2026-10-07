//! Native mutation results; synthetic extent probes do not claim giant-set execution.
#![allow(deprecated)]
use super::storage::allocation::observation::capture;
use super::{PackedInteger, PackedMap};
use crate::ncollection::packed_map_algo as algo;
use std::collections::BTreeMap;
use std::sync::OnceLock;
mod callbacks;
mod synthetic;
fn fixture() -> &'static BTreeMap<&'static str, Vec<i128>> {
    static ROWS: OnceLock<BTreeMap<&'static str, Vec<i128>>> = OnceLock::new();
    ROWS.get_or_init(|| {
        include_str!("../../../../../tools/oracle/tkernel_packed_map_mutations.expected")
            .lines()
            .map(|line| {
                let (label, values) = line.split_once(' ').unwrap();
                (
                    label,
                    values
                        .split_whitespace()
                        .map(|value| value.parse().unwrap())
                        .collect(),
                )
            })
            .collect()
    })
}
fn profile<T: PackedInteger>(shape: usize) -> PackedMap<T> {
    let mut map: PackedMap<T> = PackedMap::default();
    let collision: u64 = PackedMap::<T>::BITS_PER_BLOCK as u64 * 101;
    if shape == 1 {
        map.resize(0);
    }
    let keys: &[u64] = match shape {
        2 => &[0, collision, 32, 96, u64::MAX],
        3 => &[collision, 0, 32, 64],
        4 => &[2, 63, collision * 2],
        _ => &[],
    };
    for key in keys {
        map.add(T::from_bits(*key));
    }
    map
}
fn unary<T: PackedInteger>(
    op: usize,
    member: bool,
    map: &mut PackedMap<T>,
    other: &PackedMap<T>,
) -> i32 {
    let changed: bool = match (op, member) {
        (0, false) => algo::unite(map, other),
        (0, true) => map.unite(other),
        (2, false) => algo::intersect(map, other),
        (2, true) => map.intersect(other),
        (4, false) => algo::subtract(map, other),
        (4, true) => map.subtract(other),
        (_, false) => algo::differ(map, other),
        (_, true) => map.differ(other),
    };
    i32::from(changed)
}
fn binary<T: PackedInteger>(
    op: usize,
    member: bool,
    map: &mut PackedMap<T>,
    left: &PackedMap<T>,
    right: &PackedMap<T>,
) -> i32 {
    match (op, member) {
        (1, false) => algo::union(map, left, right),
        (1, true) => map.union(left, right),
        (3, false) => algo::intersection(map, left, right),
        (3, true) => map.intersection(left, right),
        (5, false) => algo::subtraction(map, left, right),
        (5, true) => map.subtraction(left, right),
        (_, false) => algo::difference(map, left, right),
        (_, true) => map.difference(left, right),
    }
    -2
}
fn snapshot<T: PackedInteger>(result: i32, map: &PackedMap<T>) -> Vec<i128> {
    let keys: Vec<i128> = map.iter().map(|key| i128::from(key.bits())).collect();
    let mut row: Vec<i128> = vec![
        i128::from(result),
        map.bucket_count as i128,
        map.count as i128,
        i128::from(map.extent()),
        i128::from(map.is_empty()),
        map.block_count as i128,
        i128::from(map.buckets.len() != 0),
        keys.len() as i128,
    ];
    row.extend(keys);
    row
}
fn caught(fail: bool, operation: impl FnOnce() -> i32) -> i32 {
    capture(fail, || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
    })
    .0
    .unwrap_or(-1)
}
fn ordinary_row<T: PackedInteger>(name: &str, op: usize, shape: usize, mode: usize) {
    let (a, b): (usize, usize) = [(0, 2), (2, 0), (2, 3), (3, 4)][shape];
    let mut left: PackedMap<T> = profile(a);
    let right: PackedMap<T> = profile(b);
    let mut destination: PackedMap<T> = profile(4);
    let result: i32 = caught(mode / 2 != 0, || {
        if op.is_multiple_of(2) {
            unary(op, !mode.is_multiple_of(2), &mut left, &right)
        } else {
            binary(op, !mode.is_multiple_of(2), &mut destination, &left, &right)
        }
    });
    let key: String = format!("{name}:ordinary:{op}:{shape}:{mode}");
    let map: &PackedMap<T> = if op.is_multiple_of(2) {
        &left
    } else {
        &destination
    };
    assert_eq!(snapshot(result, map), fixture()[key.as_str()], "{key}");
}
fn ordinary<T: PackedInteger>(name: &str) {
    for op in 0..8 {
        for case in 0..16 {
            ordinary_row::<T>(name, op, case / 4, case % 4);
        }
    }
}
fn self_rows<T: PackedInteger>(name: &str) {
    for op in [0, 2, 4, 6] {
        for case in 0..6 {
            self_row::<T>(name, op, case / 2, case % 2);
        }
    }
}
fn self_row<T: PackedInteger>(name: &str, op: usize, shape: usize, mode: usize) {
    let mut map: PackedMap<T> = profile(shape);
    let result: bool = match op {
        0 => map.unite_self(),
        2 => map.intersect_self(),
        4 => map.subtract_self(),
        _ => map.differ_self(),
    };
    let key: String = format!("{name}:self:{op}:{shape}:{mode}");
    assert_eq!(
        snapshot(i32::from(result), &map),
        fixture()[key.as_str()],
        "{key}"
    );
}
macro_rules! width_test {
    ($name:ident, $type:ty) => {
        #[test]
        fn $name() {
            ordinary::<$type>(stringify!($type));
            self_rows::<$type>(stringify!($type));
        }
    };
}
width_test!(mutations_i32, i32);
width_test!(mutations_u32, u32);
width_test!(mutations_i64, i64);
width_test!(mutations_u64, u64);
width_test!(mutations_i8, i8);
width_test!(mutations_bool, bool);
