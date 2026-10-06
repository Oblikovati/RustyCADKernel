//! Native PackedMapNode fields and bit contracts; links never own their targets.
use super::{PackedInteger, PackedMap, PackedWord};
mod ownership;

/// One native packed block with a non-owning chain link; `PackedMapNode::<i32>::default()`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PackedMapNode<T: PackedInteger> {
    next: *mut Self,
    mask: T::Word,
    bits: T::Word,
}

impl<T: PackedInteger> Default for PackedMapNode<T> {
    fn default() -> Self {
        Self::new(std::ptr::null_mut())
    }
}

impl<T: PackedInteger> PackedMapNode<T> {
    /// Preserves the native empty node's zero mask; `PackedMapNode::<i32>::new(next)`.
    pub fn new(next: *mut Self) -> Self {
        Self::from_parts(T::Word::from_bits(0), T::Word::from_bits(0), next)
    }
    /// Sets one key without changing the borrowed chain head; `PackedMapNode::from_value(42, &mut head)`.
    pub fn from_value(value: T, next: &mut *mut Self) -> Self {
        let mask = T::Word::from_bits(value.bits() & !Self::low_mask());
        let bits = T::Word::from_bits(Self::value_bit(value));
        Self::from_parts(mask, bits, *next)
    }
    /// Retains raw native mask/bits, including inconsistent counts; `PackedMapNode::<i32>::from_parts(31, 0, next)`.
    pub fn from_parts(mask: T::Word, bits: T::Word, next: *mut Self) -> Self {
        Self { next, mask, bits }
    }
    /// Copies the encoded block base/count word; `node.mask()`.
    pub fn mask(&self) -> T::Word {
        self.mask
    }
    /// Copies the occupied-bit word; `node.bits()`.
    pub fn bits(&self) -> T::Word {
        self.bits
    }
    /// Mutates the actual encoded mask field; `*node.change_mask() = 32`.
    pub fn change_mask(&mut self) -> &mut T::Word {
        &mut self.mask
    }
    /// Mutates the actual bit field without repairing the count; `*node.change_bits() = 3`.
    pub fn change_bits(&mut self) -> &mut T::Word {
        &mut self.bits
    }
    /// Returns the native block base converted to the key type; `node.key()`.
    pub fn key(&self) -> T {
        T::from_bits(self.mask.bits() & !Self::low_mask())
    }
    /// Decodes count-minus-one even for empty nodes; `assert_eq!(PackedMapNode::<i32>::default().nb_values(), 1)`.
    pub fn nb_values(&self) -> usize {
        (self.mask.bits() & Self::low_mask()) as usize + 1
    }
    /// Checks the actual bits independently of the encoded count; `node.has_values()`.
    pub fn has_values(&self) -> bool {
        self.bits.bits() != 0
    }
    /// Tests the value's low bits without checking its block base; `node.has_value(42)`.
    pub fn has_value(&self, value: T) -> bool {
        self.bits.bits() & Self::value_bit(value) != 0
    }
    /// Inserts a low-bit position and increments the entire native mask; `node.add_value(42)`.
    pub fn add_value(&mut self, value: T) -> bool {
        if self.has_value(value) {
            return false;
        }
        self.bits = T::Word::from_bits(self.bits.bits() ^ Self::value_bit(value));
        self.mask = T::Word::from_bits(self.mask.bits().wrapping_add(1));
        true
    }
    /// Erases a low-bit position and preserves native unsigned mask underflow; `node.del_value(42)`.
    pub fn del_value(&mut self, value: T) -> bool {
        if !self.has_value(value) {
            return false;
        }
        self.bits = T::Word::from_bits(self.bits.bits() ^ Self::value_bit(value));
        self.mask = T::Word::from_bits(self.mask.bits().wrapping_sub(1));
        true
    }
    /// Returns the unowned chain address without dereferencing it; `node.next()`.
    pub fn next(&self) -> *mut Self {
        self.next
    }
    /// Changes only the native next field; `node.set_next(std::ptr::null_mut())`.
    pub fn set_next(&mut self, next: *mut Self) {
        self.next = next;
    }
    /// Computes the node's one-based native bucket; `node.hash_code(101)`.
    pub fn hash_code(&self, upper: usize) -> usize {
        PackedMap::<T>::hash_code(
            T::Word::from_bits(self.mask.bits() >> PackedMap::<T>::SHIFT),
            upper,
        )
    }
    /// Compares the unsigned block index, excluding count bits; `node.is_equal(1)`.
    pub fn is_equal(&self, other: T::Word) -> bool {
        self.mask.bits() >> PackedMap::<T>::SHIFT == other.bits()
    }
    fn low_mask() -> u64 {
        PackedMap::<T>::BITS_PER_BLOCK as u64 - 1
    }
    fn value_bit(value: T) -> u64 {
        1_u64 << (value.bits() & Self::low_mask())
    }
}

#[cfg(test)]
mod layout_tests {
    use super::PackedMapNode;
    #[test]
    fn native_next_member_is_first_for_every_supported_word_width() {
        assert_eq!(std::mem::offset_of!(PackedMapNode<i32>, next), 0);
        assert_eq!(std::mem::offset_of!(PackedMapNode<u32>, next), 0);
        assert_eq!(std::mem::offset_of!(PackedMapNode<i64>, next), 0);
        assert_eq!(std::mem::offset_of!(PackedMapNode<u64>, next), 0);
        assert_eq!(std::mem::offset_of!(PackedMapNode<i8>, next), 0);
        assert_eq!(std::mem::offset_of!(PackedMapNode<bool>, next), 0);
    }
}
