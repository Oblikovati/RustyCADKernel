//! Pointer overloads use strlen; explicit-length and string-view comparisons keep full spans.
use super::AsciiString;
use crate::standard::c_string::c_str_prefix;

impl AsciiString {
    /// Compares a C-string prefix; null equals only an empty string.
    /// Example: `AsciiString::new("a").is_equal_c_string(Some(b"a\0tail"))`.
    pub fn is_equal_c_string(&self, other: Option<&[u8]>) -> bool {
        other.map_or_else(
            || self.is_empty(),
            |bytes| self.is_equal(c_str_prefix(bytes)),
        )
    }
    /// Negates native C-string equality; `AsciiString::new("a").is_different_c_string(None)`.
    pub fn is_different_c_string(&self, other: Option<&[u8]>) -> bool {
        !self.is_equal_c_string(other)
    }
    /// Compares against the C-string prefix, returning false for null.
    /// Example: `AsciiString::new("a").is_less_c_string(Some(b"b\0tail"))`.
    pub fn is_less_c_string(&self, other: Option<&[u8]>) -> bool {
        other.is_some_and(|bytes| self.is_less(c_str_prefix(bytes)))
    }
    /// Compares against a C-string prefix; every nonempty string is greater than null.
    /// Example: `AsciiString::new("b").is_greater_c_string(Some(b"a\0tail"))`.
    pub fn is_greater_c_string(&self, other: Option<&[u8]>) -> bool {
        other.map_or_else(
            || !self.is_empty(),
            |bytes| self.is_greater(c_str_prefix(bytes)),
        )
    }
    /// Checks a C-string prefix; null matches only an empty string, unlike an empty C string.
    /// Example: `AsciiString::new("abc").starts_with_c_string(Some(b"a\0tail"))`.
    pub fn starts_with_c_string(&self, other: Option<&[u8]>) -> bool {
        other.map_or_else(
            || self.is_empty(),
            |bytes| self.starts_with(c_str_prefix(bytes)),
        )
    }
    /// Checks the suffix from a native char-array argument through its first NUL.
    /// Example: `AsciiString::new("abc").ends_with_literal(b"c\0tail")`.
    pub fn ends_with_literal<const N: usize>(&self, other: &[u8; N]) -> bool {
        self.ends_with(c_str_prefix(other))
    }
}
