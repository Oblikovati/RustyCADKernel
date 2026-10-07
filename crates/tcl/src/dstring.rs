//! `Tcl_DString` and the encoding conversions that fill one.
//!
//! A `Tcl_DString` may point into its own inline buffer, so it lives in a `Box` and never
//! moves between `Tcl_DStringInit` and `Tcl_DStringFree`.
use crate::text::decode_tcl_utf8;
use crate::{TclError, sys};
use std::ffi::{c_char, c_int};
use std::mem::MaybeUninit;

/// An owned dynamic string of the C library.
///
/// ```
/// let mut text = rustycad_tcl::DString::new();
/// text.append(b"pu").unwrap();
/// text.append(b"ts").unwrap();
/// assert_eq!(text.value(), b"puts");
/// ```
pub struct DString {
    raw: Box<sys::Tcl_DString>,
}

impl DString {
    /// `Tcl_DStringInit`: an empty string.
    pub fn new() -> Self {
        crate::initialize_library();
        let mut storage: Box<MaybeUninit<sys::Tcl_DString>> = Box::new(MaybeUninit::uninit());
        // SAFETY: Tcl_DStringInit writes every field the other calls read.
        unsafe { sys::Tcl_DStringInit(storage.as_mut_ptr()) };
        // SAFETY: initialized just above; the box keeps the address stable.
        let raw = unsafe { storage.assume_init() };
        Self { raw }
    }

    /// The raw pointer for calls that fill the string.
    pub(crate) fn as_mut_ptr(&mut self) -> *mut sys::Tcl_DString {
        &mut *self.raw
    }

    /// `Tcl_DStringAppend(ds, bytes, length)`.
    pub fn append(&mut self, bytes: &[u8]) -> Result<(), TclError> {
        let length = c_length(bytes.len())?;
        // SAFETY: initialized string; `bytes` holds `length` readable bytes (NULs allowed).
        unsafe {
            sys::Tcl_DStringAppend(self.as_mut_ptr(), bytes.as_ptr().cast::<c_char>(), length)
        };
        Ok(())
    }

    /// `Tcl_DStringValue` as bytes (without the terminating NUL).
    pub fn value(&self) -> &[u8] {
        let length = usize::try_from(self.raw.length).unwrap_or(0);
        // SAFETY: `string` holds `length` initialized bytes while `self` is borrowed.
        unsafe { std::slice::from_raw_parts(self.raw.string.cast::<u8>(), length) }
    }

    /// The value decoded from Tcl's UTF-8.
    pub fn to_text(&self) -> String {
        decode_tcl_utf8(self.value()).into_owned()
    }

    /// `Tcl_DStringFree`: empties the string (it stays usable, as in C).
    pub fn clear(&mut self) {
        // SAFETY: initialized string; Tcl_DStringFree re-initializes it.
        unsafe { sys::Tcl_DStringFree(self.as_mut_ptr()) };
    }
}

impl Default for DString {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DString {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Converts a Rust length to the `int` length of the C API.
pub(crate) fn c_length(length: usize) -> Result<c_int, TclError> {
    c_int::try_from(length).map_err(|_| TclError::TooLong { length })
}

/// `Tcl_ExternalToUtfDString(NULL, ...)`: converts bytes in the system encoding to
/// Tcl's UTF-8, appending to `target` as the C function does.
///
/// ```
/// let mut text = rustycad_tcl::DString::new();
/// rustycad_tcl::external_to_utf(b"set a 1", &mut text).unwrap();
/// assert_eq!(text.to_text(), "set a 1");
/// ```
pub fn external_to_utf(bytes: &[u8], target: &mut DString) -> Result<(), TclError> {
    let length = c_length(bytes.len())?;
    // SAFETY: NULL selects the system encoding; `bytes` has `length` bytes; the target
    // is an initialized string.
    unsafe {
        sys::Tcl_ExternalToUtfDString(
            std::ptr::null_mut(),
            bytes.as_ptr().cast::<c_char>(),
            length,
            target.as_mut_ptr(),
        )
    };
    Ok(())
}

/// `Tcl_UtfToUniCharDString` then `Tcl_UniCharLen`: the UTF-16 units of `text`.
///
/// ```
/// assert_eq!(rustycad_tcl::utf_to_unichars("aé").unwrap(), vec![0x61, 0xE9]);
/// ```
pub fn utf_to_unichars(text: &str) -> Result<Vec<u16>, TclError> {
    let length = c_length(text.len())?;
    let mut target = DString::new();
    // SAFETY: `text` has `length` bytes of UTF-8; the result points into `target`.
    let units = unsafe {
        sys::Tcl_UtfToUniCharDString(text.as_ptr().cast::<c_char>(), length, target.as_mut_ptr())
    };
    // SAFETY: the result is NUL-terminated and lives as long as `target`.
    let count = usize::try_from(unsafe { sys::Tcl_UniCharLen(units) }).unwrap_or(0);
    // SAFETY: `count` units precede the terminator.
    Ok(unsafe { std::slice::from_raw_parts(units, count) }.to_vec())
}
