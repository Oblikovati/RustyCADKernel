//! Unsigned overflow defines zero-stride metadata; retained typed storage is tracked separately.
use super::{BaseAllocator, DynamicArray, LinearVector};
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
    pub(super) fn element_coordinates(&self, index: usize) -> (usize, usize) {
        if self.increment == 0 {
            // Native log2Pow2(0)=0 and the unsigned mask wraps to usize::MAX.
            return (index, index);
        }
        (index / self.increment, index % self.increment)
    }
}
#[cfg(test)]
mod tests;
