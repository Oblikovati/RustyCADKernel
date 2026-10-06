//! Native iterator endpoints are byte pointers, including null unallocated ranges.
use super::Buffer;

impl Buffer {
    /// Returns the mutable allocation address (`ChangeData` and `begin`).
    /// Example: `buffer.data_mut_ptr() == buffer.data_ptr().cast_mut()`.
    pub fn data_mut_ptr(&mut self) -> *mut u8 {
        self.data_ptr().cast_mut()
    }

    /// Returns the one-past byte address (`end` and `cend`), preserving null empty ranges.
    /// Example: `buffer.data_end_ptr() == buffer.data_ptr().wrapping_add(buffer.size())`.
    pub fn data_end_ptr(&self) -> *const u8 {
        self.data_ptr().wrapping_add(self.size())
    }

    /// Returns the mutable one-past byte address (`end`); never dereference this endpoint.
    /// Example: `buffer.data_end_mut_ptr() == buffer.data_end_ptr().cast_mut()`.
    pub fn data_end_mut_ptr(&mut self) -> *mut u8 {
        self.data_end_ptr().cast_mut()
    }
}
