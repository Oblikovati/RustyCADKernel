use super::{NonZeroUsize, U32_READER, U64_READER};
#[test]
fn primitive_reader_covers_widen_shift_truncate_modulus_and_one_based_index() {
    for mask in [0, 1, 31, 32, 63, 64, u32::MAX as u64, u64::MAX] {
        for upper in [1, 2, 101, 1009, usize::MAX] {
            check_mask(mask, NonZeroUsize::new(upper).unwrap());
        }
    }
}
fn check_mask(mask: u64, upper: NonZeroUsize) {
    for is_64_bit in [false, true] {
        let shift = if is_64_bit { 6 } else { 5 };
        let expected32 = ((mask as u32) >> shift) as usize % upper.get() + 1;
        let expected64 = (mask >> shift) as usize % upper.get() + 1;
        assert_eq!(U32_READER.bucket(mask as u32, is_64_bit, upper), expected32);
        assert_eq!(U64_READER.bucket(mask, is_64_bit, upper), expected64);
    }
}
use crate::ncollection::{PackedInteger, PackedWord};
use std::collections::BTreeMap;
use std::sync::OnceLock;
fn native_rows() -> &'static BTreeMap<&'static str, u64> {
    static ROWS: OnceLock<BTreeMap<&str, u64>> = OnceLock::new();
    ROWS.get_or_init(|| {
        include_str!("../../../../../../../tools/oracle/tkernel_packed_map_native_rehash.expected")
            .lines()
            .filter(|line| line.contains(":hash:"))
            .map(|line| {
                let (key, value) = line.split_once(' ').unwrap();
                (key, value.parse().unwrap())
            })
            .collect()
    })
}
const NATIVE_MASKS: [u64; 11] = [
    0,
    1,
    31,
    32,
    63,
    64,
    u32::MAX as u64,
    u64::MAX,
    1 << 31,
    1 << 63,
    0x1234abcdffff1234,
];

fn primitive_hashes<T: PackedInteger>(name: &str) {
    let reader = T::Word::NATIVE_REHASH_READER.unwrap();
    for (index, mask) in NATIVE_MASKS.into_iter().enumerate() {
        for (slot, upper) in [1, 2, 3, 101, 1009, usize::MAX].into_iter().enumerate() {
            let actual = reader.bucket(
                T::Word::from_bits(mask),
                T::IS_64_BIT,
                NonZeroUsize::new(upper).unwrap(),
            );
            assert_eq!(
                actual as u64,
                native_rows()[format!("{name}:hash:{index}:{slot}").as_str()]
            );
        }
    }
}
macro_rules! widths {
    ($($name:ident: $ty:ty),+) => { $(#[test] fn $name() { primitive_hashes::<$ty>(stringify!($ty)); })+ };
}
widths!(reader_i8: i8, reader_u8: u8, reader_i16: i16, reader_u16: u16, reader_i32: i32, reader_u32: u32, reader_i64: i64, reader_u64: u64, reader_usize: usize, reader_isize: isize, reader_bool: bool);
