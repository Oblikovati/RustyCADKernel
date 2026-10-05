//! Compiler fixture for the slice/array trait argument shapes emitted by rustdoc 61.

/// Owns bytes for conversion fixtures; `ByteCapture::from(&[1_u8, 2])`.
pub struct ByteCapture(pub Vec<u8>);

impl<const N: usize> From<&[u8; N]> for ByteCapture {
    fn from(bytes: &[u8; N]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<&[u8]> for ByteCapture {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<&mut [u8; 4]> for ByteCapture {
    fn from(bytes: &mut [u8; 4]) -> Self {
        Self(bytes.to_vec())
    }
}
