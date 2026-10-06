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

impl<T: Clone> Sequence<T> {
    /// Matches native iterator InsertAfter(T&&), which copies its named rvalue argument.
    /// Example: `sequence.insert_after_cursor_rvalue(&cursor, &mut item)?` leaves item unchanged.
    /// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 omits std::forward in this overload.
    pub fn insert_after_cursor_rvalue(
        &mut self,
        cursor: &SequenceCursor,
        value: &mut T,
    ) -> Result<&mut T, Failure> {
        self.insert_after_cursor_ref(cursor, value)
    }
}
