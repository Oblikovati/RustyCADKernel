//! One disjoint borrowing implementation serves swaps, descending shifts and aliased insertions.
use super::DynamicArray;
impl<T> DynamicArray<T> {
    pub(super) fn distinct_value_slots(&mut self, first: usize, second: usize) -> (&mut T, &mut T) {
        debug_assert_ne!(
            first, second,
            "slot {first}: expected distinct insertion source and target"
        );
        let (lower, higher) = self.ordered_value_slots(first.min(second), first.max(second));
        if first < second {
            return (lower, higher);
        }
        (higher, lower)
    }
    fn ordered_value_slots(&mut self, lower: usize, higher: usize) -> (&mut T, &mut T) {
        let (lower_block, higher_block) = (lower / self.increment, higher / self.increment);
        let (lower_offset, higher_offset) = (lower % self.increment, higher % self.increment);
        if lower_block == higher_block {
            let (prefix, suffix) = self.blocks[lower_block].split_at_mut(higher_offset);
            return (&mut prefix[lower_offset], &mut suffix[0]);
        }
        let (prefix, suffix) = self.blocks.as_mut_slice().split_at_mut(higher_block);
        (
            &mut prefix[lower_block][lower_offset],
            &mut suffix[0][higher_offset],
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disjoint_pairs_and_self_swap_keep_native_value_order() {
        let mut array = DynamicArray::with_increment(2);
        for value in [1, 2, 3, 4] {
            array.append(value);
        }
        for (first, second) in [(0, 1), (1, 0), (0, 3), (3, 0)] {
            let (left, right) = array.distinct_value_slots(first, second);
            *left += 10;
            *right += 100;
        }
        array.swap(2, 2);
        assert_eq!(
            array.iter().copied().collect::<Vec<_>>(),
            [221, 112, 3, 114]
        );
    }
}
