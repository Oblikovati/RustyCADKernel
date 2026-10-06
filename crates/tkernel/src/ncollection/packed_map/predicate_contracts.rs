//! Extent injection below is synthetic branch evidence, never a claim of giant-set parity.
#![allow(deprecated)]
use super::{PackedInteger, PackedMap};
use crate::ncollection::packed_map_algo as algo;
use std::collections::BTreeMap;
use std::sync::OnceLock;
mod callbacks;
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
fn fixture() -> &'static BTreeMap<&'static str, Vec<i128>> {
    static ROWS: OnceLock<BTreeMap<&'static str, Vec<i128>>> = OnceLock::new();
    ROWS.get_or_init(|| {
        include_str!("../../../../../tools/oracle/tkernel_packed_map_predicates.expected")
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
    if shape == 1 {
        map.resize(0);
    }
    if shape == 2 || shape == 4 {
        map.add(T::from_bits(0));
    }
    if shape == 3 || shape == 4 {
        for key in [1, 31, 32, 3232, u64::MAX] {
            map.add(T::from_bits(key));
        }
    }
    if shape == 5 {
        for key in [2, 63, 64, 6464] {
            map.add(T::from_bits(key));
        }
    }
    map
}
fn predicate_results<T: PackedInteger>(left: &PackedMap<T>, right: &PackedMap<T>) -> [bool; 8] {
    [
        algo::contains(left, right),
        algo::is_subset(left, right),
        algo::is_equal(left, right),
        algo::has_intersection(left, right),
        left.contains_map(right),
        left.is_subset(right),
        left.is_equal(right),
        left.has_intersection(right),
    ]
}
fn observations<T: PackedInteger>(left: &PackedMap<T>, right: &PackedMap<T>) -> Vec<i128> {
    let mut results: Vec<i128> = vec![
        left.len() as i128,
        i128::from(left.extent()),
        i128::from(left.is_empty()),
        right.len() as i128,
        i128::from(right.extent()),
        i128::from(right.is_empty()),
        i128::from(left.extent() > right.extent()),
    ];
    results.extend(predicate_results(left, right).map(i128::from));
    results
}
fn ordinary<T: PackedInteger>(name: &str) {
    for a in 0..6 {
        let left: PackedMap<T> = profile(a);
        assert_eq!(
            observations(&left, &left),
            fixture()[format!("{name}:self:{a}").as_str()]
        );
        for b in 0..6 {
            let right: PackedMap<T> = profile(b);
            assert_eq!(
                observations(&left, &right),
                fixture()[format!("{name}:ordinary:{a}:{b}").as_str()]
            );
        }
    }
}
fn synthetic_pair<T: PackedInteger>(
    name: &str,
    shape: usize,
    left: &mut PackedMap<T>,
    right: &mut PackedMap<T>,
) {
    let saved: (usize, usize) = (left.count, right.count);
    for (a, left_extent) in EXTENTS.into_iter().enumerate() {
        for (b, right_extent) in EXTENTS.into_iter().enumerate() {
            left.count = left_extent;
            right.count = right_extent;
            let label: String = format!("{name}:synthetic:{shape}:{a}:{b}");
            assert_eq!(
                observations(left, right),
                fixture()[label.as_str()],
                "{label}"
            );
        }
    }
    (left.count, right.count) = saved;
}
fn synthetic<T: PackedInteger>(name: &str) {
    for (shape, (a, b)) in [(0, 2), (2, 0), (2, 2), (2, 4), (4, 2), (2, 5)]
        .into_iter()
        .enumerate()
    {
        synthetic_pair(name, shape, &mut profile::<T>(a), &mut profile::<T>(b));
    }
}
macro_rules! ordinary_test {
    ($name:ident, $type:ty) => {
        #[test]
        fn $name() {
            ordinary::<$type>(stringify!($type));
        }
    };
}
ordinary_test!(ordinary_i32, i32);
ordinary_test!(ordinary_u32, u32);
ordinary_test!(ordinary_i64, i64);
ordinary_test!(ordinary_u64, u64);
ordinary_test!(ordinary_i8, i8);
ordinary_test!(ordinary_bool, bool);
#[test]
fn synthetic_i64() {
    synthetic::<i64>("i64");
}
#[test]
fn synthetic_u64() {
    synthetic::<u64>("u64");
}
