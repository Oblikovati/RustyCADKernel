//! Protected native contracts available to Rust callers without subclassing.
use super::{PackedInteger, PackedMap, PackedWord};

impl<T: PackedInteger> PackedMap<T> {
    /// Tests native packed-block growth pressure; `assert!(PackedMap::<i32>::default().resizable())`.
    pub fn resizable(&self) -> bool {
        self.is_empty() || self.block_count > self.bucket_count
    }

    /// Computes the native unsigned block index; `assert_eq!(PackedMap::<i32>::packed_key_index(32), 1)`.
    pub fn packed_key_index(key: T) -> T::Word {
        T::Word::from_bits(key.bits() >> Self::SHIFT)
    }

    /// Computes the native one-based bucket; `assert_eq!(PackedMap::<i32>::hash_code(5, 3), 3)`.
    /// A zero bucket count is outside the native defined domain and panics here.
    pub fn hash_code(key_index: T::Word, bucket_count: usize) -> usize {
        assert!(
            bucket_count != 0,
            "bucket count {bucket_count}; expected a value in 1..=usize::MAX"
        );
        key_index.bits() as usize % bucket_count + 1
    }

    /// Updates the mask's native count-minus-one bits; `PackedMap::<i32>::population(&mut 0, 0b101)` returns 2.
    /// Zero population stores all low bits set, preserving OCCT's unsigned underflow behavior.
    pub fn population(mask: &mut T::Word, block_bits: T::Word) -> usize {
        let count: u64 = u64::from(block_bits.bits().count_ones());
        let low_mask: u64 = Self::BITS_PER_BLOCK as u64 - 1;
        let preserved_base: u64 = mask.bits() & !low_mask;
        *mask = T::Word::from_bits(preserved_base | (count.wrapping_sub(1) & low_mask));
        count as usize
    }
}
