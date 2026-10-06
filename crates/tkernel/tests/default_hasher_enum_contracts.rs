//! Native enum hashing casts discriminants directly, including negative and wide representations.
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, HashKey, HashPolicy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
enum SignedKey {
    Negative = -7,
    MinusOne = -1,
    Positive = 9,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
enum WideKey {
    Zero = 0,
    High = 0x8000_0000_0000_0021,
    Max = u64::MAX,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlainKey {
    Negative = -3,
    Positive = 5,
}
rustycad_tkernel::impl_enum_hash_key!(SignedKey, WideKey, PlainKey);

#[test]
fn signed_enum_hashing_uses_size_t_cast_and_native_equality() {
    for (key, expected) in [
        (SignedKey::Negative, (-7_i32) as usize),
        (SignedKey::MinusOne, usize::MAX),
        (SignedKey::Positive, 9),
    ] {
        assert_eq!(key.occt_hash(), expected);
        assert!(DefaultHashPolicy.equal(&key, &key));
    }
    assert!(!DefaultHashPolicy.equal(&SignedKey::Negative, &SignedKey::Positive));
}

#[test]
fn wide_unsigned_enum_hashing_preserves_or_truncates_to_pointer_width() {
    assert_eq!(WideKey::Zero.occt_hash(), 0);
    assert_eq!(
        WideKey::High.occt_hash(),
        0x8000_0000_0000_0021_u64 as usize
    );
    assert_eq!(WideKey::Max.occt_hash(), usize::MAX);
    assert!(DefaultHashPolicy.equal(&WideKey::High, &WideKey::High));
    assert!(!DefaultHashPolicy.equal(&WideKey::Zero, &WideKey::High));
}

#[test]
fn enums_without_explicit_representation_also_hash_discriminants() {
    assert_eq!(PlainKey::Negative.occt_hash(), (-3_isize) as usize);
    assert_eq!(PlainKey::Positive.occt_hash(), 5);
}

#[test]
fn actual_default_maps_find_enum_keys_across_duplicate_binding_and_resize() {
    let mut signed = DataMap::<SignedKey, i32>::default();
    assert!(signed.bind(SignedKey::Negative, 1));
    assert!(!signed.bind(SignedKey::Negative, 2));
    assert_eq!(signed.find(&SignedKey::Negative), Ok(&2));
    assert_eq!(signed.len(), 1);
    let mut wide = DataMap::<WideKey, i32>::default();
    for (key, value) in [(WideKey::Zero, 1), (WideKey::High, 2), (WideKey::Max, 3)] {
        assert!(wide.bind(key, value));
    }
    wide.resize(101);
    assert_eq!(wide.len(), 3);
    assert_eq!(wide.find(&WideKey::Zero), Ok(&1));
    assert_eq!(wide.find(&WideKey::High), Ok(&2));
    assert_eq!(wide.find(&WideKey::Max), Ok(&3));
}
