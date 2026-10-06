use super::{BaseMapIndexedIterator, IndexedCursorSource};
use crate::standard::Failure;

impl<C: IndexedCursorSource, const M: bool> BaseMapIndexedIterator<C, M> {
    /// Matches native equality: all exhausted iterators compare equal, even across owners.
    /// Example: `left.is_equal(&left_map, &right, &right_map)?`.
    pub fn is_equal<const OTHER: bool>(
        &self,
        container: &C,
        other: &BaseMapIndexedIterator<C, OTHER>,
        other_container: &C,
    ) -> Result<bool, Failure> {
        self.is_equal_with(
            container,
            other,
            other_container,
            C::indexed_size,
            C::indexed_size,
        )
    }
    /// Negates native equality with its Size short-circuit; `left.is_not_equal(&map, &right, &map)?`.
    pub fn is_not_equal<const OTHER: bool>(
        &self,
        container: &C,
        other: &BaseMapIndexedIterator<C, OTHER>,
        other_container: &C,
    ) -> Result<bool, Failure> {
        self.is_equal(container, other, other_container)
            .map(|equal| !equal)
    }
}

impl<C, const M: bool> BaseMapIndexedIterator<C, M> {
    /// Compares proxy owners without requiring borrowed getters; `left.is_equal_with(&a, &right, &b, size, size)?`.
    pub fn is_equal_with<const OTHER: bool>(
        &self,
        container: &C,
        other: &BaseMapIndexedIterator<C, OTHER>,
        other_container: &C,
        size: impl FnOnce(&C) -> usize,
        other_size: impl FnOnce(&C) -> usize,
    ) -> Result<bool, Failure> {
        // Native && skips the second Size hook when the first cursor is live.
        if !self.more_with(container, size)? && !other.more_with(other_container, other_size)? {
            return Ok(true);
        }
        Ok(self.owner == other.owner && self.index == other.index)
    }
    /// Negates proxy-owner equality with identical Size dispatch; `left.is_not_equal_with(&a, &right, &b, size, size)?`.
    pub fn is_not_equal_with<const OTHER: bool>(
        &self,
        container: &C,
        other: &BaseMapIndexedIterator<C, OTHER>,
        other_container: &C,
        size: impl FnOnce(&C) -> usize,
        other_size: impl FnOnce(&C) -> usize,
    ) -> Result<bool, Failure> {
        self.is_equal_with(container, other, other_container, size, other_size)
            .map(|equal| !equal)
    }
    /// Compares native signed index distance without Size callbacks; `left.is_less(&right)`.
    pub fn is_less<const OTHER: bool>(&self, other: &BaseMapIndexedIterator<C, OTHER>) -> bool {
        self.distance(other) < 0
    }
    /// Reverses native less-than comparison; `left.is_greater(&right)`.
    pub fn is_greater<const OTHER: bool>(&self, other: &BaseMapIndexedIterator<C, OTHER>) -> bool {
        other.is_less(self)
    }
    /// Negates the reversed native comparison; `left.is_less_or_equal(&right)`.
    pub fn is_less_or_equal<const OTHER: bool>(
        &self,
        other: &BaseMapIndexedIterator<C, OTHER>,
    ) -> bool {
        !other.is_less(self)
    }
    /// Negates native less-than comparison; `left.is_greater_or_equal(&right)`.
    pub fn is_greater_or_equal<const OTHER: bool>(
        &self,
        other: &BaseMapIndexedIterator<C, OTHER>,
    ) -> bool {
        !self.is_less(other)
    }
}
