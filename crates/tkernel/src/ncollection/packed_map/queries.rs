use super::{PackedInteger, PackedMap, PackedWord};
impl<T: PackedInteger> PackedMap<T> {
    /// Return the native minimum or type maximum when empty; `map.minimal()`.
    pub fn minimal(&self) -> T {
        let mut result: T = T::MAX;
        let mut selected = None;
        for block in self.buckets.iter().flatten() {
            let base: T = T::from_bits(block.index.bits() << Self::SHIFT);
            if result > base {
                result = base;
                selected = Some(block);
            }
        }
        selected.map_or(result, |block| {
            T::from_bits(
                (block.index.bits() << Self::SHIFT) | u64::from(block.bits.bits().trailing_zeros()),
            )
        })
    }
    /// Preserve native block-base selection, including the lowest-block edge case; `map.maximal()`.
    pub fn maximal(&self) -> T {
        let mut result: T = T::MIN;
        let mut selected = None;
        for block in self.buckets.iter().flatten() {
            let base: T = T::from_bits(block.index.bits() << Self::SHIFT);
            if result < base {
                result = base;
                selected = Some(block);
            }
        }
        selected.map_or(result, |block| {
            T::from_bits(
                (block.index.bits() << Self::SHIFT)
                    | u64::from(63 - block.bits.bits().leading_zeros()),
            )
        })
    }
}
