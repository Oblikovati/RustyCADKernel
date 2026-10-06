use super::{UtfBufferError, UtfIterator, UtfUnit};

impl<'a, T: UtfUnit> UtfIterator<'a, T> {
    /// Accepts OCCT's null constructor argument; `UtfIterator::<u8>::from_optional(None)`.
    pub fn from_optional(text: Option<&'a [T]>) -> Self {
        let mut iterator = Self {
            buffer: &[],
            position: 0,
            position_next: 0,
            char_index: 0,
            char_utf32: 0,
            is_null: true,
        };
        iterator.init_optional(text);
        iterator
    }

    /// Reinitializes to a buffer or OCCT's null state; `iterator.init_optional(None)`.
    pub fn init_optional(&mut self, text: Option<&'a [T]>) {
        self.buffer = text.unwrap_or(&[]);
        self.is_null = text.is_none();
        self.position = 0;
        self.position_next = 0;
        self.char_utf32 = 0;
        self.char_index = 0;
        if !self.is_null {
            self.advance();
            self.char_index = 0;
        }
    }

    /// Returns the current code-unit address, preserving null; `iterator.buffer_here_ptr()`.
    /// Synthetic zero padding beyond the slice is not dereferenceable storage.
    pub fn buffer_here_ptr(&self) -> *const T {
        self.buffer_position_ptr(self.position)
    }

    /// Returns the next code-unit address, preserving null; `iterator.buffer_next_ptr()`.
    /// Synthetic zero padding beyond the slice is not dereferenceable storage.
    pub fn buffer_next_ptr(&self) -> *const T {
        self.buffer_position_ptr(self.position_next)
    }

    fn buffer_position_ptr(&self, offset: usize) -> *const T {
        if self.is_null {
            return std::ptr::null();
        }
        self.buffer.as_ptr().wrapping_add(offset)
    }

    /// Encodes one symbol and returns the remaining output, like GetUtf's next pointer.
    /// Example: `let remaining = iterator.get_utf_remaining(&mut output)?;`.
    pub fn get_utf_remaining<'out, W: UtfUnit>(
        &self,
        output: &'out mut [W],
    ) -> Result<&'out mut [W], UtfBufferError> {
        let written = self.get_utf(output)?;
        Ok(&mut output[written..])
    }
}

impl<T: UtfUnit> std::ops::Deref for UtfIterator<'_, T> {
    type Target = u32;
    fn deref(&self) -> &u32 {
        &self.char_utf32
    }
}
