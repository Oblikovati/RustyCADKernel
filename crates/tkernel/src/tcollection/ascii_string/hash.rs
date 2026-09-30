use super::AsciiString;
use crate::standard::hashing::{HashWord, fnv_hash, hash_bytes};
impl AsciiString {
    /// `HashCode`: FNV-1a over signed `char` bytes below four bytes, Murmur otherwise, at the
    /// native pointer width; `AsciiString::new("abc").hash_code()`.
    pub fn hash_code(&self) -> usize {
        let bytes: &[u8] = self.as_bytes();
        if bytes.len() < 4 {
            // `FNVHash::hash_combine(*myString, myLength)`: the same engine as the C-string hasher.
            return fnv_hash::hash_combine(bytes, bytes.len(), usize::FNV_SEED);
        }
        hash_bytes(bytes)
    }
}
