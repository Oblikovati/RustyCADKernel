//! A custom key with primitive storage must not enter key conversions during native rehash.
use rustycad_tkernel::ncollection::{PackedInteger, PackedMap};
use std::cell::Cell;
thread_local! { static FORBID_KEY_CALLBACKS: Cell<bool> = const { Cell::new(false) }; }
fn require_key_conversion() {
    FORBID_KEY_CALLBACKS.with(|forbid| {
        assert!(
            !forbid.get(),
            "key conversion during rehash: expected only primitive word operations"
        );
    });
}
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        FORBID_KEY_CALLBACKS.with(|forbid| forbid.set(false));
    }
}
macro_rules! key {
    ($name:ident, $signed:ty, $word:ty) => {
        #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct $name($signed);
        impl PackedInteger for $name {
            type Word = $word;
            const IS_64_BIT: bool = size_of::<$word>() == 8;
            const MIN: Self = Self(<$signed>::MIN);
            const MAX: Self = Self(<$signed>::MAX);
            fn bits(self) -> u64 {
                require_key_conversion();
                (self.0 as $word) as u64
            }
            fn from_bits(bits: u64) -> Self {
                require_key_conversion();
                Self(bits as $signed)
            }
        }
    };
}
key!(Observed32, i32, u32);
key!(Observed64, i64, u64);
fn no_key_conversions<T: PackedInteger>() {
    let mut map = PackedMap::<T>::default();
    for bits in [0, 1, 64, 6464, u64::MAX] {
        map.add(T::from_bits(bits));
    }
    let result = {
        FORBID_KEY_CALLBACKS.with(|forbid| forbid.set(true));
        let _reset = Reset;
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.resize(1000)))
    };
    assert!(result.is_ok());
    assert_eq!(map.bucket_count(), 1009);
    assert_eq!(map.len(), 5);
    assert!(map.contains(T::from_bits(u64::MAX)));
}
#[test]
fn native_word32_rehash_never_calls_custom_key_conversions() {
    no_key_conversions::<Observed32>();
}
#[test]
fn native_word64_rehash_never_calls_custom_key_conversions() {
    no_key_conversions::<Observed64>();
}
