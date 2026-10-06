//! Direct native subclass oracles for the four protected PackedMap contracts.
mod packed_map_oracle;
use packed_map_oracle::{NativeRows, decode_native_rows};
use rustycad_tkernel::ncollection::{PackedInteger, PackedMap, PackedWord};
use std::sync::OnceLock;

fn native_rows() -> &'static NativeRows {
    static ROWS: OnceLock<NativeRows> = OnceLock::new();
    ROWS.get_or_init(|| {
        decode_native_rows(include_str!(
            "../../../tools/oracle/tkernel_packed_map_helpers.expected"
        ))
    })
}

fn fixture_keys<T: PackedInteger>() -> [T; 10] {
    let bits = [
        T::MIN.bits(),
        T::MAX.bits(),
        u64::MAX,
        0,
        1,
        31,
        32,
        63,
        64,
        65,
    ];
    bits.map(T::from_bits)
}

fn fixture_masks<T: PackedInteger>() -> [T::Word; 8] {
    let highest = 1_u64 << (PackedMap::<T>::BITS_PER_BLOCK - 1);
    [
        0,
        1,
        31,
        63,
        0x12345678,
        0xabcdef0123456789,
        u64::MAX,
        highest,
    ]
    .map(T::Word::from_bits)
}

fn fixture_population_bits<T: PackedInteger>(index: usize) -> T::Word {
    let fixed = [
        0,
        u64::MAX,
        0x5555555555555555,
        0xaaaaaaaaaaaaaaaa,
        0x80000001,
        0xffffffff,
    ];
    let bits = fixed
        .get(index)
        .copied()
        .unwrap_or_else(|| 1 << (index - 6));
    T::Word::from_bits(bits)
}

fn check_native_row(label: &str, actual: &[u64]) {
    assert_eq!(actual, native_rows()[label].as_slice(), "{label}");
}

fn key_index_cases<T: PackedInteger>(name: &str) {
    for (index, key) in fixture_keys::<T>().into_iter().enumerate() {
        let actual = PackedMap::<T>::packed_key_index(key);
        check_native_row(&format!("{name}:key:{index}"), &[actual.bits()]);
    }
}

fn hash_cases<T: PackedInteger>(name: &str) {
    let indices = fixture_keys::<T>().map(PackedMap::<T>::packed_key_index);
    for (index, key_index) in indices.into_iter().enumerate() {
        check_hash_moduli::<T>(name, "hash", index, key_index);
    }
    for (index, key_index) in fixture_masks::<T>().into_iter().enumerate() {
        check_hash_moduli::<T>(name, "raw_hash", index, key_index);
    }
}

fn check_hash_moduli<T: PackedInteger>(name: &str, kind: &str, index: usize, key_index: T::Word) {
    for (modulus, buckets) in [1, 2, 3, 101, 1009, usize::MAX].into_iter().enumerate() {
        let actual = PackedMap::<T>::hash_code(key_index, buckets);
        check_native_row(
            &format!("{name}:{kind}:{index}:{modulus}"),
            &[actual as u64],
        );
    }
}

fn population_cases<T: PackedInteger>(name: &str) {
    for (mask_index, original_mask) in fixture_masks::<T>().into_iter().enumerate() {
        for bits_index in 0..PackedMap::<T>::BITS_PER_BLOCK as usize + 6 {
            check_population_case::<T>(name, mask_index, original_mask, bits_index);
        }
    }
}

fn check_population_case<T: PackedInteger>(
    name: &str,
    mask_index: usize,
    mut mask: T::Word,
    bits_index: usize,
) {
    let count = PackedMap::<T>::population(&mut mask, fixture_population_bits::<T>(bits_index));
    let label = format!("{name}:population:{mask_index}:{bits_index}");
    check_native_row(&label, &[count as u64, mask.bits()]);
}

fn growth_cases<T: PackedInteger>(name: &str) {
    let mut map = PackedMap::<T>::default();
    check_growth_row(name, "growth", 0, &map);
    for index in 0..104 {
        map.add(T::from_bits(index * PackedMap::<T>::BITS_PER_BLOCK as u64));
        check_growth_row(name, "growth", index + 1, &map);
    }
    for index in 0..104 {
        map.remove(T::from_bits(index * PackedMap::<T>::BITS_PER_BLOCK as u64));
        check_growth_row(name, "growth", index + 105, &map);
    }
    map.clear();
    check_growth_row(name, "growth", 209, &map);
}

fn dense_cases<T: PackedInteger>(name: &str) {
    let mut map = PackedMap::<T>::default();
    check_growth_row(name, "dense", 0, &map);
    for index in 0..104 {
        map.add(T::from_bits(index));
        check_growth_row(name, "dense", index + 1, &map);
    }
}

fn check_growth_row<T: PackedInteger>(name: &str, kind: &str, step: u64, map: &PackedMap<T>) {
    let actual = [
        u64::from(map.resizable()),
        map.bucket_count() as u64,
        map.size() as u64,
    ];
    check_native_row(&format!("{name}:{kind}:{step}"), &actual);
}

macro_rules! helper_width_tests {
    ($module:ident, $integer:ty) => {
        mod $module {
            use super::*;
            #[test]
            fn key_index() {
                key_index_cases::<$integer>(stringify!($integer));
            }
            #[test]
            fn hash_code() {
                hash_cases::<$integer>(stringify!($integer));
            }
            #[test]
            fn population() {
                population_cases::<$integer>(stringify!($integer));
            }
            #[test]
            fn resizable() {
                growth_cases::<$integer>(stringify!($integer));
                dense_cases::<$integer>(stringify!($integer));
            }
        }
    };
}
helper_width_tests!(signed_32, i32);
helper_width_tests!(unsigned_32, u32);
helper_width_tests!(signed_64, i64);
helper_width_tests!(unsigned_64, u64);

#[test]
#[should_panic(expected = "bucket count 0; expected a value in 1..=usize::MAX")]
fn zero_bucket_count_rejects_native_undefined_domain() {
    PackedMap::<i32>::hash_code(7, 0);
}
