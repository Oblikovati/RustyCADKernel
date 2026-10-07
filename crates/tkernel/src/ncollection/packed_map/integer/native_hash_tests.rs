//! Native ReSize bucket index of a block mask, compared with the pinned OCCT rows.
use crate::ncollection::{PackedInteger, PackedMap, PackedWord};
use std::collections::BTreeMap;
use std::sync::OnceLock;
fn native_rows() -> &'static BTreeMap<&'static str, u64> {
    static ROWS: OnceLock<BTreeMap<&str, u64>> = OnceLock::new();
    ROWS.get_or_init(|| {
        include_str!("../../../../../../tools/oracle/tkernel_packed_map_native_rehash.expected")
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
/// Native rehash reads the stored mask as its unsigned word, drops the count bits and
/// takes `% upper + 1`; the node hash code applies the same steps.
fn rehash_bucket<T: PackedInteger>(mask: u64, upper: usize) -> usize {
    let word = T::Word::from_bits(mask);
    let block = T::Word::from_bits(word.bits() >> PackedMap::<T>::SHIFT);
    PackedMap::<T>::hash_code(block, upper)
}
fn primitive_hashes<T: PackedInteger>(name: &str) {
    for (index, mask) in NATIVE_MASKS.into_iter().enumerate() {
        for (slot, upper) in [1, 2, 3, 101, 1009, usize::MAX].into_iter().enumerate() {
            assert_eq!(
                rehash_bucket::<T>(mask, upper) as u64,
                native_rows()[format!("{name}:hash:{index}:{slot}").as_str()]
            );
        }
    }
}
macro_rules! widths {
    ($($name:ident: $ty:ty),+) => { $(#[test] fn $name() { primitive_hashes::<$ty>(stringify!($ty)); })+ };
}
widths!(hash_i8: i8, hash_u8: u8, hash_i16: i16, hash_u16: u16, hash_i32: i32, hash_u32: u32, hash_i64: i64, hash_u64: u64, hash_usize: usize, hash_isize: isize, hash_bool: bool);
