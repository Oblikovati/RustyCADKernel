//! Native iterator expressions and the distinct checked legacy / cached STL key domains.
use super::{PackedInteger, PackedMap, PackedMapIter};
use crate::standard::Failure;
impl<'a, T: PackedInteger> PackedMapIter<'a, T> {
    /// Constructs an iterator at the first packed bit; `let cursor = PackedMapIter::new(&map)`.
    pub fn new(map: &'a PackedMap<T>) -> Self {
        map.iter()
    }
    /// Returns the legacy key or its native exception at end; `cursor.key_checked()?`.
    pub fn key_checked(&self) -> Result<&T, Failure> {
        self.key().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "NCollection_PackedMap::Iterator::Key at end; expected a current packed key",
                "",
            )
        })
    }
    /// Returns legacy Value with the same checked domain as Key; `cursor.value_checked()?`.
    pub fn value_checked(&self) -> Result<&T, Failure> {
        self.key_checked()
    }
    /// Borrows the STL cached key, defined even after exhaustion; `cursor.cached_value()`.
    pub fn cached_value(&self) -> &T {
        &self.cached_key
    }
    /// Returns the STL arrow pointer to iterator-owned cached storage; `cursor.value_ptr()`.
    pub fn value_ptr(&self) -> *const T {
        std::ptr::from_ref(&self.cached_key)
    }
    /// Copies the iterator state and returns its destination; `cursor.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        self.clone_from(other);
        self
    }
    /// Returns the destination of native prefix increment; `cursor.advance_returning()`.
    pub fn advance_returning(&mut self) -> &mut Self {
        self.advance();
        self
    }
    /// Copies the old state before incrementing; `let previous = cursor.post_advance()`.
    pub fn post_advance(&mut self) -> Self {
        let previous = self.clone();
        self.advance();
        previous
    }
    /// Compares native iterator state, treating all end iterators as equal; `cursor.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }
}
