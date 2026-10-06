//! Borrowed item overloads allocate before invoking the native copy-constructor analogue.
use super::{Sequence, SequenceCursor};
use crate::standard::Failure;

impl<T: Clone> Sequence<T> {
    /// Prepends a copied item after allocating its node; `sequence.prepend_ref(&item)`.
    pub fn prepend_ref(&mut self, value: &T) -> &mut T {
        self.emplace_prepend(|| value.clone())
    }

    /// Copies an item after a checked one-based position; `sequence.insert_after_ref(0, &item)?`.
    pub fn insert_after_ref(&mut self, index: usize, value: &T) -> Result<&mut T, Failure> {
        self.emplace_after(index, || value.clone())
    }

    /// Copies an item before a checked one-based position; `sequence.insert_before_ref(1, &item)?`.
    pub fn insert_before_ref(&mut self, index: usize, value: &T) -> Result<&mut T, Failure> {
        self.emplace_before(index, || value.clone())
    }

    /// Copies an item after a detached node position; `sequence.insert_after_cursor_ref(&cursor, &item)?`.
    pub fn insert_after_cursor_ref(
        &mut self,
        cursor: &SequenceCursor,
        value: &T,
    ) -> Result<&mut T, Failure> {
        self.emplace_after_cursor(cursor, || value.clone())
    }
}
