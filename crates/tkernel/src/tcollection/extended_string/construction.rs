//! The TCollection_ExtendedString constructors.
use super::{
    ExtendedString,
    errors::{leading_units, null_object, out_of_range},
    utf,
};
use crate::{standard::Failure, tcollection::AsciiString};
use std::sync::LazyLock;
impl ExtendedString {
    /// Converts a C string: UTF-8 when `is_multi_byte` and the bytes decode, otherwise each
    /// byte widened; reading stops at the first NUL. `ExtendedString::from_c_string(b"\xE2\x82\xAC", true)`.
    pub fn from_c_string(bytes: &[u8], is_multi_byte: bool) -> Self {
        let text = utf::c_string_prefix(bytes);
        if is_multi_byte && let Some(units) = utf::utf8_to_utf16(text) {
            return Self::from_raw(&units);
        }
        Self::widen(text)
    }
    /// Decodes UTF-8 (the `const char*` constructor with `theIsMultiByte = true`);
    /// `ExtendedString::from_utf8("é")`.
    pub fn from_utf8(text: &str) -> Self {
        Self::from_c_string(text.as_bytes(), true)
    }
    /// Converts an AsciiString with UTF-8 detection (the default `theIsMultiByte = true`);
    /// `ExtendedString::from_ascii(&AsciiString::new("abc"))`.
    pub fn from_ascii(ascii: &AsciiString) -> Self {
        Self::from_ascii_string(ascii, true)
    }
    /// Converts an AsciiString: its C string as UTF-8 when `is_multi_byte` and it decodes,
    /// otherwise every byte of its length widened, embedded NULs included;
    /// `ExtendedString::from_ascii_string(&AsciiString::new("abc"), false)`.
    pub fn from_ascii_string(ascii: &AsciiString, is_multi_byte: bool) -> Self {
        let bytes = ascii.as_bytes();
        if is_multi_byte && let Some(units) = utf::utf8_to_utf16(utf::c_string_prefix(bytes)) {
            return Self::from_raw(&units);
        }
        Self::widen(bytes)
    }
    /// Creates a one-unit string, or an empty one for the zero unit; `ExtendedString::from_char(0x20AC)`.
    pub fn from_char(unit: u16) -> Self {
        Self::from_utf16(&[unit])
    }
    /// Creates `length` copies of `unit` (zero units included) or raises Standard_OutOfRange
    /// for a negative length; `ExtendedString::filled(5, 88)?`.
    pub fn filled(length: i32, unit: u16) -> Result<Self, Failure> {
        let Ok(count) = usize::try_from(length) else {
            return Err(out_of_range(
                "TCollection_ExtendedString : negative length",
                format_args!("theLength {length}; expected >= 0"),
            ));
        };
        if count == 0 {
            return Ok(Self::default());
        }
        let mut terminated = Vec::with_capacity(super::storage::padded_unit_capacity(count));
        terminated.resize(count + 1, unit);
        terminated[count] = 0;
        Ok(Self { terminated })
    }
    /// Converts UTF-32 `wchar_t` units through the first zero, dropping surrogates and values
    /// above U+10FFFF; `ExtendedString::from_wide(&[0x1F600, 0])`.
    pub fn from_wide(units: &[u32]) -> Self {
        Self::from_raw(&utf::utf32_to_utf16(units))
    }
    /// Formats with C's `%d`; `ExtendedString::from_integer(42)`.
    pub fn from_integer(value: i32) -> Self {
        Self::widen(value.to_string().as_bytes())
    }
    /// Formats with C's `%g` (six significant digits); `ExtendedString::from_real(3.14)`.
    pub fn from_real(value: f64) -> Self {
        Self::widen(AsciiString::from_real(value).as_bytes())
    }
    /// Copies an explicit span, zero units included (the `std::u16string_view` constructor);
    /// `ExtendedString::from_raw(&[65, 0, 66])`.
    pub fn from_raw(units: &[u16]) -> Self {
        if units.is_empty() {
            return Self::default();
        }
        let mut terminated = Vec::with_capacity(super::storage::padded_unit_capacity(units.len()));
        terminated.extend_from_slice(units);
        terminated.push(0);
        Self { terminated }
    }
    /// The (pointer, length) constructor: a null pointer raises Standard_NullObject, a
    /// negative length Standard_OutOfRange; `ExtendedString::from_span(Some(&[65, 66]), 2)?`.
    pub fn from_span(units: Option<&[u16]>, length: i32) -> Result<Self, Failure> {
        let Some(units) = units else {
            return Err(null_object(
                "TCollection_ExtendedString : null parameter ",
                format_args!(
                    "theString null with theLength {length}; expected a code-unit pointer"
                ),
            ));
        };
        let Ok(count) = usize::try_from(length) else {
            return Err(out_of_range(
                "TCollection_ExtendedString : negative length",
                format_args!("theLength {length}; expected >= 0"),
            ));
        };
        Ok(Self::from_raw(leading_units(units, count)))
    }
    /// The shared empty string (`EmptyString`); `ExtendedString::empty_string().is_empty()`.
    pub fn empty_string() -> &'static Self {
        static EMPTY: LazyLock<ExtendedString> = LazyLock::new(ExtendedString::default);
        &EMPTY
    }
    /// Widens bytes with `ToExtCharacter` (zero extension of the unsigned byte).
    pub(super) fn widen(bytes: &[u8]) -> Self {
        Self::from_raw(
            &bytes
                .iter()
                .map(|&byte| u16::from(byte))
                .collect::<Vec<_>>(),
        )
    }
}
