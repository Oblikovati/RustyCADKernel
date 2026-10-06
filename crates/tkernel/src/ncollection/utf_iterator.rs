//! NCollection_UtfIterator: reads UTF-8/16/32 text symbol by symbol as UTF-32 code points.
//!
//! The native iterator walks a NUL-terminated pointer without overrun protection. Here the
//! buffer is a slice whose end acts as the terminator and whose storage past the end reads as
//! zeros, so truncated sequences decode exactly as they do over zero-padded native memory.
mod encode;
mod mutable;
pub use mutable::UtfIteratorMut;
mod positions;
mod signed_wide;
mod unit;
pub use unit::{UtfUnit, WideChar};

use std::fmt;

/// A destination buffer too short for the symbol being written; the native getters overrun.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UtfBufferError {
    /// Code units the current symbol needs.
    pub needed: usize,
    /// Code units the caller supplied.
    pub available: usize,
}

impl fmt::Display for UtfBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "buffer of {} code units: expected at least {}",
            self.available, self.needed
        )
    }
}

impl std::error::Error for UtfBufferError {}

/// Symbol cursor over Unicode text of code unit `T`; `*iter` is `current()`.
///
/// ```
/// use rustycad_tkernel::ncollection::UtfIterator;
/// let text = "h\u{e9}".as_bytes();
/// let mut iter = UtfIterator::new(text);
/// iter.advance();
/// assert_eq!((iter.current(), iter.index(), iter.buffer_here()), (0xE9, 1, 1));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct UtfIterator<'a, T: UtfUnit> {
    buffer: &'a [T],
    position: usize,
    position_next: usize,
    char_index: i32,
    char_utf32: u32,
    is_null: bool,
}

impl<'a, T: UtfUnit> UtfIterator<'a, T> {
    /// Starts at the first symbol of `text` (index 0); `UtfIterator::new(b"abc")`.
    pub fn new(text: &'a [T]) -> Self {
        Self::from_optional(Some(text))
    }

    /// Restarts on `text`, reading its first symbol; `iter.init(b"xyz")`.
    pub fn init(&mut self, text: &'a [T]) {
        self.init_optional(Some(text));
    }

    /// Pre-increment: reads the next symbol without overrun checks; `iter.advance()`.
    pub fn advance(&mut self) -> &mut Self {
        assert!(
            !self.is_null,
            "UtfIterator buffer is null; expected a non-null code-unit buffer"
        );
        self.position = self.position_next;
        self.char_index += 1;
        let (symbol, next) = T::read_symbol(self.buffer, self.position_next);
        self.char_utf32 = symbol;
        self.position_next = next;
        self
    }

    /// Post-increment: returns the state before advancing; `let old = iter.post_increment()`.
    pub fn post_increment(&mut self) -> Self {
        let copy = *self;
        self.advance();
        copy
    }

    /// True when the current symbol is at most U+10FFFF; `iter.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.char_utf32 <= encode::UTF32_MAX_LEGAL
    }

    /// The current UTF-32 code point (native `operator*`); `iter.current()`.
    pub fn current(&self) -> u32 {
        self.char_utf32
    }

    /// Code-unit offset of the current symbol (native `BufferHere` pointer); `iter.buffer_here()`.
    pub fn buffer_here(&self) -> usize {
        self.position
    }

    /// Code-unit offset of the next symbol (native `BufferNext`); `iter.buffer_next()`.
    pub fn buffer_next(&self) -> usize {
        self.position_next
    }

    /// Symbols advanced since initialization, first symbol 0; `iter.index()`.
    pub fn index(&self) -> i32 {
        self.char_index
    }
}

impl<T: UtfUnit> UtfIterator<'_, T> {
    /// Bytes to store the symbol in UTF-8; 0 for surrogates and illegal values.
    pub fn advance_bytes_utf8(&self) -> i32 {
        encode::utf8_length(self.char_utf32) as i32
    }

    /// Bytes to store the symbol in UTF-16: 2, 4 for a surrogate pair, 0 when illegal.
    pub fn advance_bytes_utf16(&self) -> i32 {
        self.advance_code_units_utf16() * 2
    }

    /// UTF-16 code units for the symbol: 1, 2 for a surrogate pair, 0 when illegal.
    pub fn advance_code_units_utf16(&self) -> i32 {
        encode::utf16_length(self.char_utf32) as i32
    }

    /// Always 4 bytes, even for illegal values (method kept for consistency).
    pub fn advance_bytes_utf32(&self) -> i32 {
        4
    }

    /// Bytes to store the symbol with code unit `W`; `iter.advance_bytes_utf::<u16>()`.
    pub fn advance_bytes_utf<W: UtfUnit>(&self) -> i32 {
        W::advance_bytes(self.char_utf32) as i32
    }

    /// Writes the symbol as UTF-8 at the start of `out`; returns the units written.
    pub fn get_utf8(&self, out: &mut [u8]) -> Result<usize, UtfBufferError> {
        self.get_utf(out)
    }

    /// Writes native signed-char UTF-8 storage; `iter.get_utf8_signed(&mut bytes)?`.
    pub fn get_utf8_signed(&self, out: &mut [i8]) -> Result<usize, UtfBufferError> {
        self.get_utf(out)
    }

    /// Writes the symbol as UTF-16 at the start of `out`; returns the units written.
    pub fn get_utf16(&self, out: &mut [u16]) -> Result<usize, UtfBufferError> {
        self.get_utf(out)
    }

    /// Writes the raw symbol as UTF-32 (no validation); returns the units written (1).
    pub fn get_utf32(&self, out: &mut [u32]) -> Result<usize, UtfBufferError> {
        self.get_utf(out)
    }

    /// Writes the symbol with code unit `W` (native `GetUtf<W>`); illegal symbols write nothing.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::UtfIterator;
    /// let text: Vec<u32> = vec![0x1F600];
    /// let mut out = [0u16; 2];
    /// assert_eq!(UtfIterator::new(&text).get_utf(&mut out), Ok(2));
    /// assert_eq!(out, [0xD83D, 0xDE00]);
    /// ```
    pub fn get_utf<W: UtfUnit>(&self, out: &mut [W]) -> Result<usize, UtfBufferError> {
        let needed = W::advance_bytes(self.char_utf32) / std::mem::size_of::<W>();
        if out.len() < needed {
            return Err(UtfBufferError {
                needed,
                available: out.len(),
            });
        }
        W::write_symbol(self.char_utf32, &mut out[..needed]);
        Ok(needed)
    }
}

/// Native `operator==`: iterators are equal when they point at the same buffer position.
impl<T: UtfUnit> PartialEq for UtfIterator<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.buffer_here_ptr() == other.buffer_here_ptr()
    }
}
