//! RAII owners pairing an `Allocate` with its `Free`, so safe code never frees twice.
use super::{AllocatorKind, BaseAllocator};
use allocator_api2::boxed::Box as AllocBox;
use std::ptr::NonNull;

/// Owned bytes freed through the creating allocator; `allocation.as_slice()`.
pub struct AllocatedBytes {
    allocator: BaseAllocator,
    address: NonNull<u8>,
    size: usize,
}
// SAFETY: the bytes are exclusively owned plain data; the allocator handle is Send + Sync.
unsafe impl Send for AllocatedBytes {}
// SAFETY: shared access only reads plain bytes.
unsafe impl Sync for AllocatedBytes {}

/// A typed allocation which retains its allocator; `allocator.allocate_value(42)`.
pub struct AllocatedBox<T> {
    storage: AllocBox<T, BaseAllocator>,
}

/// Contiguous typed storage retaining the allocation owner; `array.as_slice()`.
pub struct AllocatedArray<T> {
    storage: allocator_api2::vec::Vec<T, BaseAllocator>,
}

impl BaseAllocator {
    /// `Allocate(theSize)` owned until drop, which performs the matching `Free`.
    ///
    /// Bytes read as zero: upstream zeroes only the common allocator (calloc), but Rust cannot
    /// expose uninitialized or reused memory as `u8`. `None` where upstream returns NULL or
    /// throws. Example: `allocator.allocate(16).unwrap().as_slice().len() == 16`.
    pub fn allocate(&self, size: usize) -> Option<AllocatedBytes> {
        let address = self.allocate_policy(size, true)?;
        match &*self.inner {
            AllocatorKind::Incremental(engine) => super::locked(engine).acquire(),
            AllocatorKind::Accumulating(_) => {}
            _ => return Some(self.owned_bytes(address, size)),
        }
        // SAFETY: arena ranges of `size` bytes are exclusively ours; reused bytes are zeroed.
        unsafe { address.write_bytes(0, size) };
        Some(self.owned_bytes(address, size))
    }

    fn owned_bytes(&self, address: NonNull<u8>, size: usize) -> AllocatedBytes {
        AllocatedBytes {
            allocator: self.clone(),
            address,
            size,
        }
    }

    /// Allocates correctly aligned typed storage; `*allocator.allocate_value(42) == 42`.
    pub fn allocate_value<T>(&self, value: T) -> AllocatedBox<T> {
        AllocatedBox {
            storage: AllocBox::new_in(value, self.clone()),
        }
    }

    /// Constructs contiguous elements with this allocator; `allocator.allocate_array(5, |i| i)`.
    pub fn allocate_array<T>(
        &self,
        count: usize,
        constructor: impl FnMut(usize) -> T,
    ) -> AllocatedArray<T> {
        let mut storage = allocator_api2::vec::Vec::with_capacity_in(count, self.clone());
        storage.extend((0..count).map(constructor));
        AllocatedArray { storage }
    }
}

impl AllocatedBytes {
    /// Borrows allocated bytes; `bytes.as_slice().len()`.
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: `size` initialized bytes at `address` are owned by self.
        unsafe { std::slice::from_raw_parts(self.address.as_ptr(), self.size) }
    }
    /// Mutates allocated bytes; `bytes.as_mut_slice().fill(0xAB)`.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: `size` initialized bytes at `address` are exclusively owned by self.
        unsafe { std::slice::from_raw_parts_mut(self.address.as_ptr(), self.size) }
    }
    /// The allocation address, even for zero bytes (`Buffer::Data()`); `bytes.as_ptr()`.
    pub fn as_ptr(&self) -> *const u8 {
        self.address.as_ptr()
    }
    /// The allocator that performs the `Free`; `bytes.allocator() == &BaseAllocator::common()`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.allocator
    }
}

impl Drop for AllocatedBytes {
    fn drop(&mut self) {
        // SAFETY: the address came from this allocator's policy and is released exactly once.
        unsafe { self.allocator.free_typed(self.address) };
    }
}

impl<T> AllocatedArray<T> {
    /// Borrows constructed elements; `array.as_slice()[0]`.
    pub fn as_slice(&self) -> &[T] {
        &self.storage
    }
    /// Borrows elements for mutation; `array.as_mut_slice().reverse()`.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.storage
    }
}

impl<T> std::ops::Deref for AllocatedBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.storage
    }
}
impl<T> std::ops::DerefMut for AllocatedBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.storage
    }
}
