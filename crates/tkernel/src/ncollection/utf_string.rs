//! NCollection_UtfString: a NUL-terminated Unicode string in UTF-8, UTF-16 or UTF-32.
mod conversion;
mod locale;
mod operators;
#[cfg(not(target_os = "android"))]
pub use locale::UtfStringTool;

use super::{UtfIterator, UtfUnit, WideChar};

/// Unicode string of code unit `T` with cached byte size and symbol length.
///
/// Allocated storage ends with one zero unit; move construction leaves a null source. `length()` counts
/// the symbols consumed when the string was converted, including illegal symbols that wrote
/// no units (a native quirk kept on purpose).
///
/// ```
/// use rustycad_tkernel::ncollection::UtfString;
/// let text = UtfString::<u16>::from_unicode_units("h\u{e9}!".as_bytes(), -1);
/// assert_eq!((text.size(), text.length(), text.get_char(1)), (6, 3, 0xE9));
/// ```
#[derive(Debug)]
pub struct UtfString<T: UtfUnit> {
    units: Vec<T>,
    size: i32,
    length: i32,
}

/// The UTF-8 string alias `NCollection_String`; `String8::from_unicode_units(b"abc", -1)`.
pub type String8 = UtfString<u8>;

impl<T: UtfUnit> Default for UtfString<T> {
    fn default() -> Self {
        Self::with_size(0, 0)
    }
}

impl<T: UtfUnit> Clone for UtfString<T> {
    fn clone(&self) -> Self {
        let mut copy = Self::with_size(self.size, self.length);
        let count = self.size as usize / std::mem::size_of::<T>();
        copy.units[..count].copy_from_slice(&self.units[..count]);
        copy
    }
    fn clone_from(&mut self, source: &Self) {
        // Native Assign frees the old buffer before allocating its replacement.
        self.units = Vec::new();
        *self = source.clone();
    }
}

impl<T: UtfUnit> UtfString<T> {
    /// An empty string (native default constructor); `UtfString::<u8>::new().is_empty()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Zero-filled storage of `size` bytes plus the terminator (native `strAlloc`).
    fn with_size(size: i32, length: i32) -> Self {
        let count = size as usize / std::mem::size_of::<T>();
        Self {
            units: vec![T::default(); count + 1],
            size,
            length,
        }
    }

    /// Copies Unicode text of any unit type, stopping at a zero symbol or after `length`
    /// symbols when `length > 0` (native converting constructors); see `from_unicode`.
    pub fn from_unicode_units<F: UtfUnit>(source: &[F], length: i32) -> Self {
        let mut text = Self::null_storage();
        text.from_unicode(source, length);
        text
    }

    /// Symbol cursor at the first symbol (native `Iterator()`); `text.iterator().current()`.
    pub fn iterator(&self) -> UtfIterator<'_, T> {
        UtfIterator::from_optional((!self.units.is_empty()).then_some(&self.units[..]))
    }

    /// Buffer size in bytes, excluding the terminator; `text.size()`.
    pub fn size(&self) -> i32 {
        self.size
    }

    /// Cached length in symbols; `text.length()`.
    pub fn length(&self) -> i32 {
        self.length
    }

    /// The code units with their terminating zero (native `ToCString`); `text.to_c_string()`.
    pub fn to_c_string(&self) -> &[T] {
        &self.units
    }

    /// True when the first unit is the terminator; `text.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.units.first().is_none_or(|unit| *unit == T::default())
    }

    /// Resets to the empty string, length 0; `text.clear()`.
    pub fn clear(&mut self) {
        // Native Clear frees before it allocates a new terminating unit.
        self.units = Vec::new();
        self.size = 0;
        self.length = 0;
        self.units.push(T::default());
    }

    /// Copies `other` (native `Assign` / copy `operator=`); `text.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &Self {
        self.clone_from(other);
        self
    }

    /// Exchanges contents; also the native move assignment, which swaps; `a.swap(&mut b)`.
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }

    /// Symbol at `index` by a linear scan; 0 when out of range; `text.get_char(0)`.
    pub fn get_char(&self, index: i32) -> u32 {
        self.symbol_start(index).map_or(0, |(symbol, _)| symbol)
    }

    /// Units from the symbol at `index` through the terminator; None when out of range.
    pub fn get_char_buffer(&self, index: i32) -> Option<&[T]> {
        self.symbol_start(index)
            .map(|(_, position)| &self.units[position..])
    }

    /// (symbol, unit offset) of symbol `index`, scanning until the zero symbol.
    fn symbol_start(&self, index: i32) -> Option<(u32, usize)> {
        let mut iter = self.iterator();
        while iter.current() != 0 {
            if iter.index() == index {
                return Some((iter.current(), iter.buffer_here()));
            }
            iter.advance();
        }
        None
    }

    /// Symbols `[start, end)` (native `SubString`); empty when `start >= end`.
    pub fn sub_string(&self, start: i32, end: i32) -> Self {
        if start >= end {
            return Self::new();
        }
        let mut iter = self.iterator();
        while iter.current() != 0 {
            if iter.index() >= start {
                return Self::from_unicode_units(&self.units[iter.buffer_here()..], end - start);
            }
            iter.advance();
        }
        Self::new()
    }

    /// Byte-wise equality of the buffers; lengths are not compared; `a.is_equal(&b)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        let count = self.size as usize / std::mem::size_of::<T>();
        std::ptr::eq(self, other)
            || (self.size == other.size && self.units[..count] == other.units[..count])
    }

    /// A UTF-8 copy; `text.to_utf8()`.
    pub fn to_utf8(&self) -> UtfString<u8> {
        UtfString::from_unicode_units(&self.units, -1)
    }

    /// A UTF-16 copy; `text.to_utf16()`.
    pub fn to_utf16(&self) -> UtfString<u16> {
        UtfString::from_unicode_units(&self.units, -1)
    }

    /// A UTF-32 copy; `text.to_utf32()`.
    pub fn to_utf32(&self) -> UtfString<u32> {
        UtfString::from_unicode_units(&self.units, -1)
    }

    /// A copy in the platform wide encoding (UTF-32, UTF-16 on Windows); `text.to_utf_wide()`.
    pub fn to_utf_wide(&self) -> UtfString<WideChar> {
        UtfString::from_unicode_units(&self.units, -1)
    }
}

mod method_contracts;
