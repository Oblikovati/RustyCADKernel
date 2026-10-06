//! Native iterator assignment and increment return values preserve slot identity.
use super::{Failure, FlatMapIter};

impl<K> FlatMapIter<'_, K> {
    /// Returns the current slot address, corresponding to operator->; `cursor.key_ptr()?`.
    pub fn key_ptr(&self) -> Result<*const K, Failure> {
        self.key().map(std::ptr::from_ref)
    }
    /// Advances and returns this cursor, as native prefix increment; `cursor.advance_returning().key()`.
    pub fn advance_returning(&mut self) -> &mut Self {
        self.advance();
        self
    }
    /// Returns the previous cursor and then advances, as native postfix increment; `cursor.advance_post().key()`.
    pub fn advance_post(&mut self) -> Self {
        let previous = self.clone();
        self.advance();
        previous
    }
    /// Copies the native slot/index state and returns this cursor; `cursor.assign_from(&source)`.
    pub fn assign_from(&mut self, source: &Self) -> &mut Self {
        self.items = source.items.clone();
        self
    }
    /// Preserves assignment to the same iterator without aliased borrows; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Compares native slot identity, treating all exhausted iterators as equal; `first.is_equal(&second)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }
}
