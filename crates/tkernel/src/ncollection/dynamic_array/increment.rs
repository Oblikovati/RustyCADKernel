//! Unsigned overflow in native roundUpPow2 is a defined empty metadata state, not an allocation size.
use super::{ArrayError, BaseAllocator, DynamicArray, LinearVector};
impl<T> DynamicArray<T> {
    pub(super) fn with_rounded_increment(
        increment: usize,
        allocator: Option<BaseAllocator>,
    ) -> Self {
        Self {
            blocks: LinearVector::default(),
            allocator,
            increment,
            used: 0,
            needs_tail_scan: false,
        }
    }
    pub(super) fn require_element_stride(&self) {
        assert!(
            self.increment != 0,
            "block stride {}: expected nonzero stride for element access/allocation; reset the empty array with SetIncrement",
            self.increment
        );
    }
    pub(super) fn validate_element_index(&self, index: usize) -> Result<(), ArrayError> {
        if self.increment == 0 {
            return Err(self.index_error(index));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests;
