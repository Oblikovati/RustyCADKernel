//! `TCollection_AsciiString`: an owned, NUL-terminated byte string with one-based access.
//!
//! The byte buffer always ends with one NUL that is not part of the length, as OCCT's
//! `myString`/`myLength` pair does; embedded NUL bytes are allowed and kept wherever the
//! C++ member copies by explicit length, and dropped wherever it copies a C string.

/// An owned C-compatible byte string; `AsciiString::new("hello").as_bytes() == b"hello"`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsciiString {
    terminated: Vec<u8>,
}
impl Default for AsciiString {
    fn default() -> Self {
        Self {
            terminated: vec![0],
        }
    }
}
impl AsciiString {
    /// Copies a C string through its first NUL, `TCollection_AsciiString(const char*)`;
    /// `AsciiString::new("abc\0def").as_bytes() == b"abc"`.
    pub fn new(text: &str) -> Self {
        Self::from_bytes(text)
    }
    /// Copies arbitrary byte values through the first NUL, as the C-string constructor
    /// does; `AsciiString::from_bytes([255, 0]).as_bytes() == [255]`.
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        Self::from_raw(&bytes[..c_string_length(bytes)])
    }
    /// Copies an explicit byte span including embedded NULs; `AsciiString::from_raw(b"a\0b").len() == 3`.
    pub fn from_raw(bytes: &[u8]) -> Self {
        let mut terminated = Vec::with_capacity(bytes.len() + 1);
        terminated.extend_from_slice(bytes);
        terminated.push(0);
        Self { terminated }
    }
    /// Borrows content excluding the terminator (`operator std::string_view`);
    /// `AsciiString::new("abc").as_bytes() == b"abc"`.
    pub fn as_bytes(&self) -> &[u8] {
        &self.terminated[..self.terminated.len() - 1]
    }
    /// Borrows content including its terminator (`ToCString`);
    /// `AsciiString::new("abc").as_bytes_with_nul() == b"abc\0"`.
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &self.terminated
    }
    /// Returns byte length, excluding the terminator; `AsciiString::new("abc").len() == 3`.
    pub fn len(&self) -> usize {
        self.terminated.len() - 1
    }
    /// Returns OCCT's signed byte length (`Length`); `AsciiString::new("abc").length() == 3`.
    pub fn length(&self) -> i32 {
        self.len() as i32
    }
    /// Checks for zero stored length (`IsEmpty`); `AsciiString::default().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Appends another string's full span, embedded NULs included; `text.append(&AsciiString::new("suffix"))`.
    pub fn append(&mut self, suffix: &Self) {
        self.append_bytes(suffix.as_bytes());
    }
    /// Appends raw bytes before the terminator; shared by every concatenating member.
    fn append_bytes(&mut self, bytes: &[u8]) {
        self.terminated.pop();
        self.terminated.extend_from_slice(bytes);
        self.terminated.push(0);
    }
    /// Inserts raw bytes at a zero-based offset that the caller already validated.
    fn splice_bytes(&mut self, offset: usize, bytes: &[u8]) {
        self.terminated
            .splice(offset..offset, bytes.iter().copied());
    }
    /// Sets the length, keeping the terminator, after the caller validated `length <= len()`.
    fn set_length(&mut self, length: usize) {
        self.terminated.truncate(length + 1);
        self.terminated[length] = 0;
    }
}
/// `strlen` over a byte span: the offset of the first NUL, or the span length.
pub(crate) fn c_string_length(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len())
}
/// The bytes a `const char*` argument designates: the span through its first NUL.
pub(crate) fn c_string_prefix(bytes: &[u8]) -> &[u8] {
    &bytes[..c_string_length(bytes)]
}
mod c_string_comparison;
mod case_adjust;
mod comparison;
mod concatenation;
mod construction;
mod editing;
mod hash;
mod numeric;
mod raise;
mod search;
mod stream;
mod traits;
mod utf8_scan;
pub use concatenation::CatOperand;
pub(crate) use raise::negative_width;
