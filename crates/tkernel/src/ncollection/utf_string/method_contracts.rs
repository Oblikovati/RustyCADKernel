//! Pointer and ownership expressions of the immutable native UTF buffer.
use super::{UtfString, UtfUnit};
impl<T: UtfUnit> UtfString<T> {
    pub(super) fn null_storage() -> Self {
        Self {
            units: Vec::new(),
            size: 0,
            length: 0,
        }
    }
    /// Borrows native ToCString pointer identity, null only after move construction; `text.to_c_ptr()`.
    pub fn to_c_ptr(&self) -> *const T {
        if self.units.is_empty() {
            return std::ptr::null();
        }
        self.units.as_ptr()
    }
    /// Returns a symbol's native buffer pointer or null when absent; `text.get_char_buffer_ptr(1)`.
    pub fn get_char_buffer_ptr(&self, index: i32) -> *const T {
        self.get_char_buffer(index)
            .map_or(std::ptr::null(), <[T]>::as_ptr)
    }
    /// Transfers the buffer, leaving native null storage in the source; `UtfString::move_from(&mut text)`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::replace(source, Self::null_storage())
    }
    /// Swaps buffers for native move assignment and returns the destination; `text.move_assign(&mut other)`.
    pub fn move_assign(&mut self, other: &mut Self) -> &mut Self {
        self.swap(other);
        self
    }
    /// Preserves native self-copy-assignment storage and returns the const destination; `text.assign_self()`.
    pub fn assign_self(&mut self) -> &Self {
        self
    }
    /// Preserves native self-move-assignment storage and returns the mutable destination; `text.move_assign_self()`.
    pub fn move_assign_self(&mut self) -> &mut Self {
        self
    }
    /// Implements returning UTF-pointer assignment; `text.assign_unicode(b"hello")`.
    pub fn assign_unicode<F: UtfUnit>(&mut self, source: &[F]) -> &Self {
        self.from_unicode(source, -1);
        self
    }
    /// Copies a nullable Unicode source, treating null as an empty string; `text.from_optional_unicode::<u8>(None, -1)`.
    pub fn from_optional_unicode<F: UtfUnit>(&mut self, source: Option<&[F]>, length: i32) {
        self.from_unicode(source.unwrap_or(&[]), length);
    }
    /// Copies an overlapping own-buffer suffix without conflicting borrows; `text.from_self_unicode(1, 2)`.
    pub fn from_self_unicode(&mut self, start: i32, length: i32) {
        let Some((_, position)) = self.symbol_start(start) else {
            self.clear();
            return;
        };
        *self = Self::from_unicode_units(&self.units[position..], length);
    }
}
