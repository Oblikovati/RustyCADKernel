//! Native adoption keeps a logical byte count independent of the allocation extent.
use super::{AllocatedBytes, BaseAllocator, Buffer, Failure};

impl Buffer {
    /// Adopts an initialized prefix without changing the allocation address or owner.
    ///
    /// Models `NCollection_Buffer(allocator, size, data)` with `size <= data.len()`.
    /// The complete allocation is still released by its original allocator, including
    /// when the selected allocator is null; the native null-allocator leak is not reproduced.
    /// Example: `Buffer::with_data_size(Some(a.clone()), a.allocate(16).unwrap(), 4)?`.
    pub fn with_data_size(
        allocator: Option<BaseAllocator>,
        allocation: AllocatedBytes,
        size: usize,
    ) -> Result<Self, Failure> {
        let capacity = allocation.as_slice().len();
        if size > capacity {
            return Err(Failure::out_of_range(&format!(
                "buffer size {size} exceeds allocation extent {capacity}; expected 0..={capacity}"
            )));
        }
        let mut buffer = Self::with_data(allocator, allocation);
        buffer.logical_size = size;
        Ok(buffer)
    }
}
