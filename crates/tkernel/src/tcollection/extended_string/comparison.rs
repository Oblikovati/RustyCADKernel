//! Whole-string comparisons; the (pointer, length) forms live in `spans`.
use super::{ExtendedString, utf};
impl ExtendedString {
    /// Compares full code-unit content; `text.is_equal(&other)`.
    pub fn is_equal(&self, other: impl AsRef<[u16]>) -> bool {
        self.as_utf16() == other.as_ref()
    }
    /// Compares the NUL-terminated UTF-16 prefix; `text.is_equal_c_units(Some(&[65, 0, 66]))`.
    pub fn is_equal_c_units(&self, other: Option<&[u16]>) -> bool {
        self.is_equal(nul_terminated_units_or_empty(other))
    }
    /// The static two-string IsEqual; `ExtendedString::is_equal_strings(&first, &second)`.
    pub fn is_equal_strings(first: &Self, second: &Self) -> bool {
        first.is_equal(second)
    }
    /// Checks unequal content; `text.is_different(&other)`.
    pub fn is_different(&self, other: impl AsRef<[u16]>) -> bool {
        !self.is_equal(other)
    }
    /// Checks inequality against a bounded UTF-16 C string; `text.is_different_c_units(None)`.
    pub fn is_different_c_units(&self, other: Option<&[u16]>) -> bool {
        self.is_different(nul_terminated_units_or_empty(other))
    }
    /// Orders by unsigned code units, a proper prefix first; `text.is_less(&other)`.
    pub fn is_less(&self, other: impl AsRef<[u16]>) -> bool {
        self.as_utf16() < other.as_ref()
    }
    /// Tests whether this string sorts before a UTF-16 C string; `text.is_less_c_units(None)`.
    pub fn is_less_c_units(&self, other: Option<&[u16]>) -> bool {
        self.is_less(nul_terminated_units_or_empty(other))
    }
    /// Orders by unsigned code units, a proper prefix first; `text.is_greater(&other)`.
    pub fn is_greater(&self, other: impl AsRef<[u16]>) -> bool {
        self.as_utf16() > other.as_ref()
    }
    /// Tests whether this string sorts after a UTF-16 C string; `text.is_greater_c_units(None)`.
    pub fn is_greater_c_units(&self, other: Option<&[u16]>) -> bool {
        self.is_greater(nul_terminated_units_or_empty(other))
    }
    /// Tests a complete prefix; an empty prefix always matches; `text.starts_with(&prefix)`.
    pub fn starts_with(&self, prefix: impl AsRef<[u16]>) -> bool {
        self.as_utf16().starts_with(prefix.as_ref())
    }
    /// Tests a prefix through the first NUL; `text.starts_with_c_units(Some(&[65, 0, 66]))`.
    pub fn starts_with_c_units(&self, prefix: Option<&[u16]>) -> bool {
        self.starts_with(nul_terminated_units_or_empty(prefix))
    }
    /// Tests a complete suffix; an empty suffix always matches; `text.ends_with(&suffix)`.
    pub fn ends_with(&self, suffix: impl AsRef<[u16]>) -> bool {
        self.as_utf16().ends_with(suffix.as_ref())
    }
    /// Tests a suffix through the first NUL; `text.ends_with_c_units(Some(&[65, 0, 66]))`.
    pub fn ends_with_c_units(&self, suffix: Option<&[u16]>) -> bool {
        self.ends_with(nul_terminated_units_or_empty(suffix))
    }
    /// Compares, folding only ASCII A-Z when not case sensitive;
    /// `text.is_same_string(&other, false)`.
    pub fn is_same_string(&self, other: &Self, case_sensitive: bool) -> bool {
        if case_sensitive {
            return self.is_equal(other);
        }
        self.len() == other.len()
            && self
                .as_utf16()
                .iter()
                .zip(other.as_utf16())
                .all(|(&first, &second)| lower_ascii(first) == lower_ascii(second))
    }
}
fn lower_ascii(unit: u16) -> u16 {
    if (65..=90).contains(&unit) {
        unit + 32
    } else {
        unit
    }
}

/// Null native pointers have length zero; bounded buffers stop at their first NUL.
fn nul_terminated_units_or_empty(units: Option<&[u16]>) -> &[u16] {
    units.map_or(&[], utf::c_units_prefix)
}
