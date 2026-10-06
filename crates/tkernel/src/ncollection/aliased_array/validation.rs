use super::{AliasedArray, invalid_shape};
use crate::standard::Failure;

impl AliasedArray<'_> {
    /// Apply native positive signed-length validation; `AliasedArray::checked_size(3)?`.
    pub fn checked_size(count: i32) -> Result<usize, Failure> {
        if count <= 0 {
            return Err(Failure::with_type(
                "Standard_RangeError",
                &format!("AliasedArray length {count}; expected a positive signed length"),
                "",
            ));
        }
        Ok(count as usize)
    }

    /// Validate before native uint32 narrowing; `AliasedArray::checked_stride(4)?`.
    /// Positive inputs above `u32::MAX` wrap, including to zero, as in OCCT.
    pub fn checked_stride(stride: usize) -> Result<u32, Failure> {
        if stride == 0 {
            return Err(invalid_shape(stride, 0));
        }
        Ok(stride as u32)
    }
}
