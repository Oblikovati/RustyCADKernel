//! Protected bit searches operate on the actual node's fields and update the caller's mask.
use super::{PackedInteger, PackedMap, PackedMapNode, PackedWord};

impl<T: PackedInteger> PackedMap<T> {
    /// Finds the next low set bit; empty search returns the block base and all-ones mask; `PackedMap::find_next(&node, &mut mask)`.
    pub fn find_next(node: &PackedMapNode<T>, mask: &mut T::Word) -> T {
        let selected = node.bits().bits() & mask.bits();
        if selected == 0 {
            *mask = T::Word::from_bits(u64::MAX);
            return node.key();
        }
        let offset = selected.trailing_zeros();
        *mask = T::Word::from_bits(u64::MAX.checked_shl(offset + 1).unwrap_or(0));
        T::from_bits(node.key().bits().wrapping_add(u64::from(offset)))
    }
    /// Finds the highest set bit; empty search returns the block's last key and all-ones mask; `PackedMap::find_prev(&node, &mut mask)`.
    pub fn find_prev(node: &PackedMapNode<T>, mask: &mut T::Word) -> T {
        let selected = node.bits().bits() & mask.bits();
        if selected == 0 {
            *mask = T::Word::from_bits(u64::MAX);
            return T::from_bits(
                node.key()
                    .bits()
                    .wrapping_add(Self::BITS_PER_BLOCK as u64 - 1),
            );
        }
        let offset = 63 - selected.leading_zeros();
        *mask = T::Word::from_bits((1_u64 << offset) - 1);
        T::from_bits(node.key().bits().wrapping_add(u64::from(offset)))
    }
}
