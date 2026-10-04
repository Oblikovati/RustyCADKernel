//! Value operations exposed by the protected native `NCollection_AccAllocator::Block` record.

use super::{ACC_ALIGN, AccAlignedPtr, AccAlignedSize};

/// Safe value-level counterpart of protected `NCollection_AccAllocator::Block`.
///
/// Addresses are integer identities: methods mirror native cursor/count operations but never
/// dereference memory. Example: `AccBlock::new(0x1000, 16).free_size() == 16`.
/// This record does not make `BaseAllocator` subclassable or expose allocator-owned blocks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccBlock {
    /// Native `address` identity; the pointed-to storage remains caller-owned.
    pub address: usize,
    /// Native allocation cursor, rounded down to four-byte alignment.
    pub alloc_start: AccAlignedPtr,
    /// Previous descriptor identity, if present; it is never dereferenced by this value type.
    pub prev_block: Option<usize>,
    /// Number of live allocations counted by native `Allocate` and `Free`.
    pub alloc_count: i32,
}

impl AccBlock {
    /// Constructs an empty block; `AccBlock::new(0x1000, 16).is_empty()`.
    pub fn new(address: usize, size: usize) -> Self {
        Self::with_previous(address, size, None)
    }

    /// Constructs a block linked to a previous descriptor identity.
    /// Example: `AccBlock::with_previous(0x1000, 16, Some(0x800)).prev_block == Some(0x800)`.
    pub fn with_previous(address: usize, size: usize, previous: Option<usize>) -> Self {
        let mut block = Self {
            address,
            alloc_start: AccAlignedPtr::default(),
            prev_block: previous,
            alloc_count: 0,
        };
        block.set_free_size(size);
        block
    }

    /// Sets the native cursor to `address + size`, rounded down to four bytes.
    /// Example: `let mut b = AccBlock::new(0x1000, 8); b.set_free_size(13);`.
    pub fn set_free_size(&mut self, size: usize) {
        let cursor = aligned_cursor(self.address, size);
        self.alloc_start = AccAlignedPtr::from_ptr(cursor as *mut u8);
    }

    /// Returns bytes between the block address and cursor; `AccBlock::new(0x1000, 16).free_size()`.
    pub fn free_size(&self) -> usize {
        let cursor = self.alloc_start.as_ptr() as usize;
        cursor.wrapping_sub(self.address)
    }

    /// Counts one allocation and moves the cursor down by its rounded size.
    /// Example: `let mut block = AccBlock::new(0x1000, 16); let ptr = block.allocate(AccAlignedSize::new(5));`.
    pub fn allocate(&mut self, size: AccAlignedSize) -> AccAlignedPtr {
        self.alloc_count = self.alloc_count.wrapping_add(1);
        self.alloc_start = self.alloc_start.wrapping_sub(size);
        self.alloc_start
    }

    /// Decrements the live allocation count after a prior allocation.
    /// Example: `let mut b = AccBlock::new(0x1000, 16); b.allocate(AccAlignedSize::new(1)); b.free();`.
    pub fn free(&mut self) {
        self.alloc_count = self.alloc_count.wrapping_sub(1);
    }

    /// Reports whether no allocation is counted; `AccBlock::new(0x1000, 16).is_empty()`.
    pub const fn is_empty(&self) -> bool {
        self.alloc_count == 0
    }
}

/// Computes native `SetFreeSize`'s aligned cursor without dereferencing the address.
pub(crate) fn aligned_cursor(address: usize, size: usize) -> usize {
    address.wrapping_add(size) & !(ACC_ALIGN - 1)
}
