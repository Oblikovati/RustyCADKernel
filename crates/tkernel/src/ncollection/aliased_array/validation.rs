use super::AliasedArray;
use crate::standard::Failure;

impl AliasedArray<'_> {
    /// Apply native positive signed-length validation; `AliasedArray::checked_size(3)?`.
    pub fn checked_size(count: i32) -> Result<usize, Failure> {
        if count <= 0 {
            return Err(Failure::with_message(
                "Standard_RangeError",
                "NCollection_AliasedArray, length should be positive",
            ));
        }
        Ok(count as usize)
    }

    /// Validate before native uint32 narrowing; `AliasedArray::checked_stride(4)?`.
    /// Positive inputs above `u32::MAX` wrap, including to zero, as in OCCT.
    pub fn checked_stride(stride: usize) -> Result<u32, Failure> {
        if stride == 0 {
            return Err(Failure::with_message(
                "Standard_RangeError",
                "NCollection_AliasedArray, stride should be positive",
            ));
        }
        Ok(stride as u32)
    }
}

// The overload name and message-only constructor are observable in pinned OCCT 3d097a03.
pub(super) fn access_failure(method: &str, detail: &str, kind: &'static str) -> Failure {
    Failure::with_message(
        kind,
        &format!("NCollection_AliasedArray::{method}(), {detail}"),
    )
}
