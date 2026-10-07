// Rust translation of FoundationClasses/TKernel/Standard/Standard_HashUtils.lxx
// (namespace opencascade::MurmurHash::MurmurHashUtils) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `opencascade::MurmurHash::MurmurHashUtils`: the byte helpers of MurmurHash64A.
//!
//! The C++ helpers read through a `const char*` with no bound; here the bytes come as a
//! slice and a read past its end is a `Standard_OutOfRange` failure that names the
//! requested and the available byte counts, where C++ would read outside the object.

use crate::standard::{Failure, exceptions};
use bytemuck::AnyBitPattern;

/// `shift_mix(uint64_t)`: the MurmurHash64A finalization step `v ^ (v >> 47)`.
///
/// ```
/// use rustycad_tkernel::standard::hashing::murmur_hash::murmur_hash_utils::shift_mix;
/// assert_eq!(shift_mix(1 << 47), (1 << 47) | 1);
/// ```
pub fn shift_mix(value: u64) -> u64 {
    value ^ (value >> 47)
}

/// `load_bytes(const char*, int)`: the first `count` bytes as a little-endian word on every
/// platform (unsigned bytes, byte `i` shifted by `8 * i`).
///
/// Native quirk kept: the C++ `switch` has cases 1..7 only, so a `count` of 0, a negative
/// one or one above 7 loads nothing and returns 0 without reading.
///
/// ```
/// use rustycad_tkernel::standard::hashing::murmur_hash::murmur_hash_utils::load_bytes;
/// assert_eq!(load_bytes(&[0x01, 0x02], 2)?, 0x0201);
/// assert_eq!(load_bytes(&[0xFF; 8], 8)?, 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub fn load_bytes(bytes: &[u8], count: i32) -> Result<u64, Failure> {
    let Ok(length @ 1..=7) = usize::try_from(count) else {
        return Ok(0);
    };
    let loaded: &[u8] = leading_bytes(bytes, length, "load_bytes")?;
    Ok(little_endian_word(loaded))
}

/// The bytes as a little-endian word (at most 8 bytes); the infallible core of
/// [`load_bytes`] that MurmurHash64A applies to its 1..7 trailing bytes.
pub(crate) fn little_endian_word(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .rev()
        .fold(0_u64, |word, &byte| (word << 8) | u64::from(byte))
}

/// `unaligned_load<T>(const char*)`: copies `size_of::<T>()` leading bytes into a `T`
/// (`memcpy`, native byte order, no alignment requirement).
///
/// ```
/// use rustycad_tkernel::standard::hashing::murmur_hash::murmur_hash_utils::unaligned_load;
/// let bytes: [u8; 5] = [9, 1, 0, 0, 0];
/// assert_eq!(unaligned_load::<u32>(&bytes[1..])?, u32::from_ne_bytes([1, 0, 0, 0]));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub fn unaligned_load<T: AnyBitPattern>(bytes: &[u8]) -> Result<T, Failure> {
    let size: usize = std::mem::size_of::<T>();
    let loaded: &[u8] = leading_bytes(bytes, size, "unaligned_load")?;
    Ok(bytemuck::pod_read_unaligned(loaded))
}

/// The first `length` bytes, or the out-of-range failure C++ would have turned into an
/// out-of-bounds read.
fn leading_bytes<'a>(bytes: &'a [u8], length: usize, member: &str) -> Result<&'a [u8], Failure> {
    bytes.get(..length).ok_or_else(|| {
        exceptions::out_of_range(&format!(
            "MurmurHashUtils::{member}: needs {length} bytes, got {}",
            bytes.len()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_mix_folds_the_high_bits() {
        assert_eq!(shift_mix(0), 0);
        assert_eq!(shift_mix(u64::MAX), u64::MAX ^ (u64::MAX >> 47));
    }

    #[test]
    fn load_bytes_is_little_endian_for_one_to_seven_bytes() {
        let bytes: [u8; 7] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x87];
        assert_eq!(load_bytes(&bytes, 1).unwrap(), 0x01);
        assert_eq!(load_bytes(&bytes, 7).unwrap(), 0x0087_0605_0403_0201);
    }

    #[test]
    fn load_bytes_outside_the_switch_cases_is_zero() {
        let bytes: [u8; 9] = [0xFF; 9];
        for count in [-1, 0, 8, 9] {
            assert_eq!(load_bytes(&bytes, count).unwrap(), 0, "count {count}");
        }
    }

    #[test]
    fn load_bytes_past_the_slice_fails_with_the_counts() {
        let failure: Failure = load_bytes(&[1, 2], 3).unwrap_err();
        assert!(
            failure.to_string().contains("needs 3 bytes, got 2"),
            "{failure}"
        );
    }

    #[test]
    fn unaligned_load_copies_native_order_bytes() {
        let bytes: [u8; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let expected: u64 = u64::from_ne_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(unaligned_load::<u64>(&bytes[1..]).unwrap(), expected);
        assert!(unaligned_load::<u64>(&bytes[2..]).is_err());
    }
}
