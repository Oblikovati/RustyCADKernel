//! The byte-order helpers of `FSD_BinaryFile`, independent of host endianness.
use super::BinaryFile;

impl BinaryFile {
    /// Reverse every byte of a signed integer, including sign/payload bits.
    /// Example: `BinaryFile::inverse_int(0 as i32)` returns zero.
    pub const fn inverse_int(value: i32) -> i32 {
        value.swap_bytes()
    }

    /// Reverse every byte of a UTF-16 code unit, including sign/payload bits.
    /// Example: `BinaryFile::inverse_ext_char(0 as u16)` returns zero.
    pub const fn inverse_ext_char(value: u16) -> u16 {
        value.swap_bytes()
    }

    /// Reverse every byte of a unsigned 64-bit integer, including sign/payload bits.
    /// Example: `BinaryFile::inverse_uint64(0 as u64)` returns zero.
    pub const fn inverse_uint64(value: u64) -> u64 {
        value.swap_bytes()
    }

    /// Reverse every byte of a native-sized integer, including sign/payload bits.
    /// Example: `BinaryFile::inverse_size(0 as usize)` returns zero.
    pub const fn inverse_size(value: usize) -> usize {
        value.swap_bytes()
    }

    /// Reverse every byte of a binary64 representation, including sign/payload bits.
    /// Example: `BinaryFile::inverse_real(0 as f64)` returns zero.
    pub const fn inverse_real(value: f64) -> f64 {
        f64::from_bits(value.to_bits().swap_bytes())
    }

    /// Reverse every byte of a binary32 representation, including sign/payload bits.
    /// Example: `BinaryFile::inverse_short_real(0 as f32)` returns zero.
    pub const fn inverse_short_real(value: f32) -> f32 {
        f32::from_bits(value.to_bits().swap_bytes())
    }
}
