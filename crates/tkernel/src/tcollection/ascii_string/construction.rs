//! Constructors, ownership transfer and the shared empty string.
use super::{AsciiString, c_string_length};
use std::sync::OnceLock;

impl AsciiString {
    /// Creates a single nonzero byte, or an empty string for NUL (`TCollection_AsciiString(char)`);
    /// `AsciiString::from_char(b'A').as_bytes() == b"A"`.
    pub fn from_char(byte: u8) -> Self {
        if byte == 0 {
            Self::default()
        } else {
            Self::from_raw(&[byte])
        }
    }
    /// Creates `length` copies of `byte`, NUL fillers included (`TCollection_AsciiString(int, char)`);
    /// `AsciiString::filled(3, b'*').as_bytes() == b"***"`.
    pub fn filled(length: usize, byte: u8) -> Self {
        if length == 0 {
            return Self::default();
        }
        let mut terminated = Vec::with_capacity(super::storage::padded_byte_capacity(length));
        terminated.resize(length, byte);
        terminated.push(0);
        Self { terminated }
    }
    /// Copies at most `maximum` bytes, stopping at the first NUL or the span end
    /// (`TCollection_AsciiString(const char*, int)`); `AsciiString::with_max_length("hello", 3)`.
    pub fn with_max_length(text: impl AsRef<[u8]>, maximum: i32) -> Self {
        let text = text.as_ref();
        let limit = usize::try_from(maximum).unwrap_or(0).min(text.len());
        Self::from_bytes(&text[..limit])
    }
    /// Copies a view through its first NUL, as the `std::string_view` constructor does;
    /// `AsciiString::from_view(b"a\0b").len() == 1`.
    pub fn from_view(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes)
    }
    /// `TCollection_AsciiString(const TCollection_AsciiString&, char)`: appends the byte even
    /// when it is NUL, unlike `AssignCat(char)`; `AsciiString::joined_with_char(&text, b'!')`.
    pub fn joined_with_char(text: &Self, byte: u8) -> Self {
        let mut result = text.clone();
        result.append_bytes(&[byte]);
        result
    }
    /// `TCollection_AsciiString(const TCollection_AsciiString&, const char*)`: appends the C
    /// string through its first NUL; `AsciiString::joined_with_c_string(&text, "tail")`.
    pub fn joined_with_c_string(text: &Self, suffix: impl AsRef<[u8]>) -> Self {
        let suffix = suffix.as_ref();
        let mut result = text.clone();
        result.append_bytes(&suffix[..c_string_length(suffix)]);
        result
    }
    /// `TCollection_AsciiString(const TCollection_AsciiString&, const TCollection_AsciiString&)`:
    /// joins both full spans; `AsciiString::joined(&head, &tail)`.
    pub fn joined(head: &Self, tail: &Self) -> Self {
        let mut result = head.clone();
        result.append(tail);
        result
    }
    /// `TCollection_AsciiString(const wchar_t*)`: encodes UTF-32 through its first zero as
    /// UTF-8, dropping surrogates and values above U+10FFFF; `AsciiString::from_wide(&[0x20AC, 0])`.
    pub fn from_wide(units: &[u32]) -> Self {
        let mut result = Self::default();
        result.assign_cat_wide(units);
        result
    }
    /// The shared empty string (`EmptyString`); `AsciiString::empty_string().is_empty()`.
    pub fn empty_string() -> &'static Self {
        static EMPTY: OnceLock<AsciiString> = OnceLock::new();
        EMPTY.get_or_init(Self::default)
    }
    /// Moves storage while resetting the source; `AsciiString::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// `Move`: takes the source's storage and leaves it empty; `text.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = Self::move_from(source);
    }
    /// `Swap`: exchanges both strings' storage; `first.swap(&mut second)`.
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// `Clear`: releases the storage and becomes empty; `text.clear()`.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
