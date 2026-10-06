//! Exclusive sparse cursor access edits values in place without changing occupancy.
use super::{
    SparseArray, SparseArrayCursor,
    cursor_position::{SparseCursorPosition, exhausted_sparse_cursor},
};
use crate::standard::Failure;

/// An OCCT-style mutable sparse cursor; `*array.cursor_mut().change_value()? = value`.
pub struct SparseArrayCursorMut<'a, T> {
    array: Option<&'a mut SparseArray<T>>,
    position: SparseCursorPosition,
}

impl<T> Default for SparseArrayCursorMut<'_, T> {
    fn default() -> Self {
        Self {
            array: None,
            position: SparseCursorPosition::default(),
        }
    }
}

impl<T> SparseArray<T> {
    /// Starts mutable access at an index (native indexed iterator constructor).
    /// Example: `array.cursor_from_mut(5).key()`.
    pub fn cursor_from_mut(&mut self, index: usize) -> SparseArrayCursorMut<'_, T> {
        let position = SparseCursorPosition::new(self, index);
        SparseArrayCursorMut {
            array: Some(self),
            position,
        }
    }

    /// Starts a mutable cursor at the first occupied slot (`begin`, Iterator constructor).
    /// Example: `array.cursor_mut().change_value()?`.
    pub fn cursor_mut(&mut self) -> SparseArrayCursorMut<'_, T> {
        self.cursor_from_mut(0)
    }

    /// Constructs the mutable exhausted cursor (`end`); `!array.end_cursor_mut().more()`.
    pub fn end_cursor_mut(&mut self) -> SparseArrayCursorMut<'_, T> {
        self.cursor_from_mut(self.capacity())
    }
}

impl<'a, T> SparseArrayCursorMut<'a, T> {
    /// Borrows a read-only snapshot at the same position (native mutable-to-const conversion).
    /// Example: `cursor.as_const_cursor().key() == cursor.key()`.
    pub fn as_const_cursor(&self) -> SparseArrayCursor<'_, T> {
        SparseArrayCursor::from_position(self.array.as_deref(), self.position)
    }

    /// Rebinds to another array (`Init`); `cursor.initialize(&mut array)`.
    pub fn initialize(&mut self, array: &'a mut SparseArray<T>) {
        *self = array.cursor_mut();
    }

    /// Returns whether a value is available (`More`); `cursor.more()`.
    pub fn more(&self) -> bool {
        self.position.more()
    }

    /// Returns the current key or exhausted capacity (`Key`); `cursor.key()`.
    pub fn key(&self) -> usize {
        self.position.index
    }

    /// Advances to the next occupied slot (`Next`); `cursor.next()`.
    pub fn next(&mut self) {
        self.position.advance(self.array.as_deref());
    }

    /// Restarts on the same array (`Restart`); `cursor.restart()`.
    pub fn restart(&mut self) {
        self.position.restart(self.array.as_deref());
    }

    /// Borrows the current value (`Value`, const `operator()`); `cursor.value()?`.
    pub fn value(&self) -> Result<&T, Failure> {
        self.array
            .as_deref()
            .and_then(|array| array.value(self.key()))
            .ok_or_else(|| {
                exhausted_sparse_cursor("NCollection_SparseArray::Iterator::Value", self.key())
            })
    }

    /// Mutably borrows the current value (`ChangeValue`, mutable `operator()`).
    /// Example: `*cursor.change_value()? += 1`; native exhausted access is undefined.
    pub fn change_value(&mut self) -> Result<&mut T, Failure> {
        let key = self.key();
        self.array
            .as_deref_mut()
            .and_then(|array| array.change_value(key))
            .ok_or_else(|| {
                exhausted_sparse_cursor("NCollection_SparseArray::Iterator::ChangeValue", key)
            })
    }

    /// Returns the mutable current value address (`operator->`); `cursor.value_mut_ptr()?`.
    /// Dereferencing requires exclusive access to this live slot and compatible pointer provenance.
    pub fn value_mut_ptr(&mut self) -> Result<*mut T, Failure> {
        self.change_value().map(std::ptr::from_mut)
    }
}
