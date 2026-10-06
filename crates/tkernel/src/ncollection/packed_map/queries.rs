use super::{PackedInteger, PackedMap, PackedWord};
impl<T: PackedInteger> PackedMap<T> {
    /// Return the native minimum or type maximum when empty; `map.minimal()`.
    pub fn minimal(&self) -> T {
        let mut result: T = T::MAX;
        let mut selected = None;
        for block in self.buckets.nodes() {
            let base: T = block.key();
            if result > base {
                result = base;
                selected = Some(block);
            }
        }
        selected.map_or(result, |node| {
            Self::find_next(node, &mut T::Word::from_bits(u64::MAX))
        })
    }
    /// Preserve native block-base selection, including the lowest-block edge case; `map.maximal()`.
    pub fn maximal(&self) -> T {
        let mut result: T = T::MIN;
        let mut selected = None;
        for block in self.buckets.nodes() {
            let base: T = block.key();
            if result < base {
                result = base;
                selected = Some(block);
            }
        }
        selected.map_or(result, |node| {
            Self::find_prev(node, &mut T::Word::from_bits(u64::MAX))
        })
    }
}
