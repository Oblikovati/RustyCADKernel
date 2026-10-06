//! Native exact bucket/traversal fixtures for every destination/source alias partition.
use rustycad_tkernel::ncollection::{PackedInteger, PackedMap, PackedMapOperand};
use std::collections::BTreeMap;
use std::sync::OnceLock;

type NativeRows = BTreeMap<&'static str, Vec<u64>>;

fn native_rows() -> &'static NativeRows {
    static ROWS: OnceLock<NativeRows> = OnceLock::new();
    ROWS.get_or_init(|| {
        include_str!("../../../tools/oracle/tkernel_packed_map_aliases.expected")
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

fn populated<T: PackedInteger>(modulus: usize, which: usize, mask: usize) -> PackedMap<T> {
    let mut map = PackedMap::new(modulus);
    if mask & (1 << which) != 0 {
        for value in fixture_values::<T>(which) {
            map.add(value);
        }
    }
    map
}

fn fixture_values<T: PackedInteger>(which: usize) -> [T; 7] {
    let minimum = T::MIN.bits();
    let maximum = T::MAX.bits();
    let bits = [
        [minimum, maximum, 0, 31, 32, 96, 3232],
        [maximum, minimum, 1, 31, 63, 96, 6464],
        [3232, 64, 31, 0, maximum, 95, 97],
    ];
    bits[which].map(T::from_bits)
}

fn operand<'a, T: PackedInteger>(
    index: usize,
    first: &'a PackedMap<T>,
    second: &'a PackedMap<T>,
) -> PackedMapOperand<'a, T> {
    match index {
        0 => PackedMapOperand::Destination,
        1 => PackedMapOperand::Borrowed(first),
        2 => PackedMapOperand::Borrowed(second),
        _ => panic!("operand index {index}; expected 0..=2"),
    }
}

fn apply<T: PackedInteger>(
    operation: usize,
    map: &mut PackedMap<T>,
    left: PackedMapOperand<'_, T>,
    right: PackedMapOperand<'_, T>,
) {
    match operation {
        0 => map.union_operands(left, right),
        1 => map.intersection_operands(left, right),
        2 => map.subtraction_operands(left, right),
        3 => map.difference_operands(left, right),
        _ => panic!("operation index {operation}; expected 0..=3"),
    }
}

fn check_case<T: PackedInteger>(
    name: &str,
    operation: usize,
    mask: usize,
    left: usize,
    right: usize,
) {
    let mut destination = populated::<T>(700, 0, mask);
    let first = populated::<T>(200, 1, mask);
    let second = populated::<T>(300, 2, mask);
    apply(
        operation,
        &mut destination,
        operand(left, &first, &second),
        operand(right, &first, &second),
    );
    let mut actual = vec![destination.bucket_count() as u64, destination.size() as u64];
    actual.extend(destination.iter().map(PackedInteger::bits));
    let label = format!("{name}:{operation}:{mask}:{left}:{right}");
    assert_eq!(&actual, &native_rows()[label.as_str()], "{label}");
}

fn check_operation<T: PackedInteger>(name: &str, operation: usize) {
    for mask in 0..8 {
        for pair in 0..9 {
            check_case::<T>(name, operation, mask, pair / 3, pair % 3);
        }
    }
}

macro_rules! alias_width_tests {
    ($module:ident, $integer:ty) => {
        mod $module {
            use super::*;
            #[test]
            fn union() {
                check_operation::<$integer>(stringify!($integer), 0);
            }
            #[test]
            fn intersection() {
                check_operation::<$integer>(stringify!($integer), 1);
            }
            #[test]
            fn subtraction() {
                check_operation::<$integer>(stringify!($integer), 2);
            }
            #[test]
            fn difference() {
                check_operation::<$integer>(stringify!($integer), 3);
            }
        }
    };
}
alias_width_tests!(signed_32, i32);
alias_width_tests!(unsigned_32, u32);
alias_width_tests!(signed_64, i64);
alias_width_tests!(unsigned_64, u64);
