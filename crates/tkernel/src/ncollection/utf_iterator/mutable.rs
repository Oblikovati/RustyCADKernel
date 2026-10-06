//! Exclusive buffer access for the native ChangeBufferHere contract.
use super::{UtfIterator, UtfUnit};

/// A Unicode cursor that exclusively borrows writable code-unit storage.
///
/// Native ChangeBufferHere does not refresh the cached symbol or next pointer.
/// Rust views borrow this cursor, so a view cannot outlive a subsequent buffer edit.
/// ```
/// use rustycad_tkernel::ncollection::UtfIteratorMut;
/// let mut text = *b"ab\0";
/// let mut cursor = UtfIteratorMut::new(&mut text);
/// cursor.change_buffer_here().unwrap()[0] = b'z';
/// assert_eq!(cursor.view().current(), u32::from(b'a'));
/// cursor.rewind();
/// assert_eq!(cursor.view().current(), u32::from(b'z'));
/// ```
#[derive(Debug)]
pub struct UtfIteratorMut<'a, T: UtfUnit> {
    buffer: Option<&'a mut [T]>,
    position: usize,
    position_next: usize,
    char_index: i32,
    char_utf32: u32,
}

impl<'a, T: UtfUnit> UtfIteratorMut<'a, T> {
    /// Reads the first symbol with exclusive storage access; `UtfIteratorMut::new(&mut text)`.
    pub fn new(text: &'a mut [T]) -> Self {
        Self::from_optional(Some(text))
    }

    /// Preserves a native null cursor; `UtfIteratorMut::<u8>::from_optional(None)`.
    pub fn from_optional(text: Option<&'a mut [T]>) -> Self {
        let mut cursor = Self {
            buffer: text,
            position: 0,
            position_next: 0,
            char_index: 0,
            char_utf32: 0,
        };
        cursor.rewind();
        cursor
    }

    /// Borrows all immutable cursor operations; `assert_eq!(cursor.view().index(), 0)`.
    pub fn view(&self) -> UtfIterator<'_, T> {
        UtfIterator {
            buffer: self.buffer.as_deref().unwrap_or(&[]),
            position: self.position,
            position_next: self.position_next,
            char_index: self.char_index,
            char_utf32: self.char_utf32,
            is_null: self.buffer.is_none(),
        }
    }

    /// Borrows writable storage from the current pointer; `cursor.change_buffer_here()?[0] = unit`.
    /// Returns None for null or synthetic padding beyond the slice. The cached symbol and
    /// next position remain unchanged until advancing or rewinding, as in native OCCT.
    pub fn change_buffer_here(&mut self) -> Option<&mut [T]> {
        self.buffer.as_deref_mut()?.get_mut(self.position..)
    }

    /// Reads the cached next position after buffer edits; `cursor.advance()`.
    pub fn advance(&mut self) -> &mut Self {
        let mut view = self.view();
        view.advance();
        let state = (
            view.position,
            view.position_next,
            view.char_index,
            view.char_utf32,
        );
        self.assign_position(state);
        self
    }

    /// Re-reads the current buffer from its start after mutation; `cursor.rewind()`.
    pub fn rewind(&mut self) {
        let view = UtfIterator::from_optional(self.buffer.as_deref());
        let state = (
            view.position,
            view.position_next,
            view.char_index,
            view.char_utf32,
        );
        self.assign_position(state);
    }

    /// Rebinds storage and resets the index; `cursor.init_optional(Some(&mut replacement))`.
    pub fn init_optional(&mut self, text: Option<&'a mut [T]>) {
        *self = Self::from_optional(text);
    }

    fn assign_position(&mut self, state: (usize, usize, i32, u32)) {
        (
            self.position,
            self.position_next,
            self.char_index,
            self.char_utf32,
        ) = state;
    }
}
