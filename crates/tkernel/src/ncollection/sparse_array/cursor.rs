//! OCCT 3d097a's ConstIterator observes ascending occupied keys without copying values.
use super::{
    SparseArray,
    cursor_position::{SparseCursorPosition, exhausted_sparse_cursor},
};
use crate::standard::Failure;

/// An OCCT-style sparse read cursor; `array.cursor().more()`.
pub struct SparseArrayCursor<'a, T> {
    array: Option<&'a SparseArray<T>>,
    position: SparseCursorPosition,
}

impl<T> Default for SparseArrayCursor<'_, T> {
    fn default() -> Self {
        Self {
            array: None,
            position: SparseCursorPosition::default(),
        }
    }
}

impl<T> SparseArray<T> {
    /// Starts at the first occupied key at or after an index (native indexed iterator constructor).
    /// Example: `array.cursor_from(5).key()`; a key beyond capacity remains observable.
    pub fn cursor_from(&self, index: usize) -> SparseArrayCursor<'_, T> {
        SparseArrayCursor::from_position(Some(self), SparseCursorPosition::new(self, index))
    }

    /// Starts at the first occupied slot (`begin`, `cbegin`, ConstIterator constructor).
    /// Example: `array.cursor().key()`.
    pub fn cursor(&self) -> SparseArrayCursor<'_, T> {
        self.cursor_from(0)
    }

    /// Constructs the exhausted cursor (`end`, `cend`); `!array.end_cursor().more()`.
    pub fn end_cursor(&self) -> SparseArrayCursor<'_, T> {
        self.cursor_from(self.capacity())
    }
}

impl<'a, T> SparseArrayCursor<'a, T> {
    /// Copies a cursor's borrowed array and position, returning this destination (`operator=`).
    /// Example: `destination.assign_from(&source).key() == source.key()`.
    pub fn assign_from(&mut self, source: &Self) -> &mut Self {
        *self = source.clone();
        self
    }

    /// Preserves this cursor on self-assignment without conflicting borrows; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }

    /// Copies a mutable cursor's position into a borrowed const cursor (converting `operator=`).
    /// Example: `destination.assign_from_mutable(&source).key() == source.key()`.
    pub fn assign_from_mutable(
        &mut self,
        source: &'a super::SparseArrayCursorMut<'_, T>,
    ) -> &mut Self {
        *self = source.as_const_cursor();
        self
    }

    pub(super) fn from_position(
        array: Option<&'a SparseArray<T>>,
        position: SparseCursorPosition,
    ) -> Self {
        Self { array, position }
    }

    /// Rebinds to another array's first slot (`Init`); `cursor.initialize(&array)`.
    pub fn initialize(&mut self, array: &'a SparseArray<T>) {
        *self = array.cursor();
    }

    /// Returns whether a value is available (`More`); `cursor.more()`.
    pub fn more(&self) -> bool {
        self.position.more()
    }

    /// Returns the current key, capacity at exhaustion, or zero when detached; `cursor.key()`.
    pub fn key(&self) -> usize {
        self.position.index
    }

    /// Advances to the next occupied slot (`Next`); exhaustion is stable; `cursor.next()`.
    pub fn next(&mut self) {
        self.position.advance(self.array);
    }

    /// Restarts on the same array (`Restart`); `cursor.restart()`.
    pub fn restart(&mut self) {
        self.position.restart(self.array);
    }

    /// Borrows the current value (`Value`, `operator()`); `cursor.value()?`.
    /// Reading an exhausted native cursor is undefined; Rust reports Standard_NoSuchObject.
    pub fn value(&self) -> Result<&'a T, Failure> {
        self.array
            .and_then(|array| array.value(self.key()))
            .ok_or_else(|| {
                exhausted_sparse_cursor("NCollection_SparseArray::ConstIterator::Value", self.key())
            })
    }

    /// Returns the current value's address (`operator->`); `cursor.value_ptr()?`.
    /// Dereferencing the pointer requires the array to remain alive and unmodified.
    pub fn value_ptr(&self) -> Result<*const T, Failure> {
        self.value().map(std::ptr::from_ref)
    }

    /// Advances and returns this cursor (`operator++`); `cursor.advance_returning().key()`.
    pub fn advance_returning(&mut self) -> &mut Self {
        self.next();
        self
    }

    /// Returns a shallow copy before advancing (`operator++(int)`); `cursor.advance_post()`.
    pub fn advance_post(&mut self) -> Self {
        let previous = self.clone();
        self.next();
        previous
    }
}

impl<T> Clone for SparseArrayCursor<'_, T> {
    fn clone(&self) -> Self {
        Self::from_position(self.array, self.position)
    }

    fn clone_from(&mut self, source: &Self) {
        self.assign_from(source);
    }
}

impl<T> PartialEq for SparseArrayCursor<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        if !self.more() && !other.more() {
            return true;
        }
        let same_array = match (self.array, other.array) {
            (Some(first), Some(second)) => std::ptr::eq(first, second),
            (None, None) => true,
            _ => false,
        };
        same_array && self.key() == other.key()
    }
}

impl<T> Eq for SparseArrayCursor<'_, T> {}
