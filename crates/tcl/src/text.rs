//! Conversions between Rust strings and the NUL-terminated strings of the C API.
//!
//! Tcl keeps strings in its internal UTF-8 form, which spells U+0000 as the overlong pair
//! `C0 80` so that a C string never contains a zero byte. Text coming from Tcl is decoded
//! with that rule; any other invalid byte becomes U+FFFD (Tcl itself never produces one).
use crate::TclError;
use std::borrow::Cow;
use std::ffi::{CStr, CString, c_char};

/// Copies `text` into a C string; text with a NUL byte cannot cross the C API.
pub(crate) fn to_c_string(text: &str) -> Result<CString, TclError> {
    CString::new(text).map_err(|_| TclError::InteriorNul {
        text: text.to_string(),
    })
}

/// Decodes the C string at `pointer` (Tcl's UTF-8); `None` for a null pointer.
///
/// # Safety
/// `pointer` must be null or point to a NUL-terminated string that stays valid during
/// the call.
pub(crate) unsafe fn from_c_pointer(pointer: *const c_char) -> Option<String> {
    if pointer.is_null() {
        return None;
    }
    // SAFETY: non-null, and the caller guarantees a valid NUL-terminated string.
    let bytes = unsafe { CStr::from_ptr(pointer) }.to_bytes();
    Some(decode_tcl_utf8(bytes).into_owned())
}

/// Decodes Tcl's internal UTF-8, mapping the `C0 80` pair back to U+0000.
///
/// ```text
/// assert_eq!(decode_tcl_utf8(b"a\xC0\x80b"), "a\0b");
/// ```
pub(crate) fn decode_tcl_utf8(bytes: &[u8]) -> Cow<'_, str> {
    if !bytes.windows(2).any(|pair| pair == [0xC0, 0x80]) {
        return String::from_utf8_lossy(bytes);
    }
    let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(&[0xC0, 0x80]) {
            decoded.push(0);
            index += 2;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    Cow::Owned(String::from_utf8_lossy(&decoded).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_overlong_nul_pair() {
        assert_eq!(decode_tcl_utf8(b"a\xC0\x80b"), "a\0b");
        assert_eq!(decode_tcl_utf8("plain é".as_bytes()), "plain é");
    }

    #[test]
    fn rejects_interior_nul_with_the_text() {
        let error = to_c_string("a\0b").unwrap_err();
        assert_eq!(
            error.to_string(),
            "text \"a\\0b\" contains a NUL byte; the Tcl C API takes NUL-terminated strings"
        );
    }

    #[test]
    fn null_pointer_decodes_to_none() {
        // SAFETY: a null pointer is explicitly allowed.
        assert_eq!(unsafe { from_c_pointer(std::ptr::null()) }, None);
        let text = CString::new("value").unwrap();
        // SAFETY: `text` is a live NUL-terminated string.
        assert_eq!(
            unsafe { from_c_pointer(text.as_ptr()) }.as_deref(),
            Some("value")
        );
    }
}
