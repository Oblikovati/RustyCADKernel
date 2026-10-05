use super::AsciiString;
impl AsRef<[u8]> for AsciiString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
impl From<&str> for AsciiString {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}
impl From<&AsciiString> for AsciiString {
    fn from(text: &AsciiString) -> Self {
        text.clone()
    }
}
impl From<u8> for AsciiString {
    fn from(byte: u8) -> Self {
        Self::from_char(byte)
    }
}
impl From<i32> for AsciiString {
    fn from(value: i32) -> Self {
        Self::from_integer(value)
    }
}
impl From<f64> for AsciiString {
    fn from(value: f64) -> Self {
        Self::from_real(value)
    }
}
impl PartialEq<&str> for AsciiString {
    fn eq(&self, other: &&str) -> bool {
        self.is_equal_c_string(Some(other.as_bytes()))
    }
}
impl PartialOrd for AsciiString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for AsciiString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}
impl From<&[u8]> for AsciiString {
    fn from(bytes: &[u8]) -> Self {
        Self::from_raw(bytes)
    }
}
/// `std::hash<TCollection_AsciiString>` feeds [`AsciiString::hash_code`] to the hasher.
impl std::hash::Hash for AsciiString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_code());
    }
}
