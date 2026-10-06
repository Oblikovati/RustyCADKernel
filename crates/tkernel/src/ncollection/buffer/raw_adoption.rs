//! Raw-policy adoption transfers a real allocation and protects safe byte slices from arena reset.
#![allow(unsafe_code)]
use super::{BaseAllocator, Buffer, Failure};
use std::ptr::NonNull;

impl Buffer {
    /// Adopts native raw-policy storage without allocating, freeing or changing its bytes.
    ///
    /// Models `NCollection_Buffer(allocator, size, data)` for a non-null allocator. A null `data`
    /// follows the normal allocation path. Non-null storage becomes exclusively owned until
    /// `free`, replacement or drop; its complete allocation is released even for a logical prefix.
    /// Example: `unsafe { Buffer::from_raw_data(a, 0, pointer) }?` adopts a zero-length view.
    ///
    /// # Safety
    /// A non-null `data` must start a live allocation returned by this allocator's `allocate_raw`
    /// or `allocate_optimal_raw`, never a typed `allocate_value`/`allocate_array`/allocator-api2
    /// allocation or foreign heap pointer. The first `size` bytes must be initialized, exclusively
    /// owned, inside that allocation, and at most `isize::MAX` bytes long. Transfer all ownership:
    /// no other owner may free, access, or invalidate the bytes by resetting the arena. Keep them valid
    /// throughout this call; in particular prevent concurrent arena reset until adoption finishes.
    pub unsafe fn from_raw_data(
        allocator: BaseAllocator,
        size: usize,
        data: *mut u8,
    ) -> Result<Self, Failure> {
        let Some(address) = NonNull::new(data) else {
            return Self::new(Some(allocator), size);
        };
        // SAFETY: the caller transfers initialized storage of this exact raw policy.
        let allocation = unsafe { allocator.adopt_raw_bytes(address, size) };
        Ok(Self::with_data(Some(allocator), allocation))
    }
}
