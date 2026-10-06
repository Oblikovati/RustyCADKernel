//! NCollection_AccAllocator's address-keyed blocks, filled downwards and freed when empty.
//!
//! Blocks are found by `address / block_size` exactly as upstream; the map stores block
//! descriptors by key and the `prevBlock` chain links keys instead of descriptor pointers.
use super::raw_heap::{
    MALLOC_ALIGNMENT, heap_allocate, heap_free, heap_payload_size, heap_reallocate,
};
use super::{AccAlignedSize, AccBlock, AccBlockKey};
use crate::standard::Failure;
use std::{collections::HashMap, ptr::NonNull};

/// `NCollection_AccAllocator::Align`: every size and address is rounded to 4 bytes.
pub const ACC_ALIGN: usize = 4;
/// `NCollection_AccAllocator::DefaultBlockSize`.
pub const ACC_DEFAULT_BLOCK_SIZE: usize = 24600;
/// `NCollection_AccAllocator::MaxLookupBlocks`: previous blocks scanned for free space.
pub const ACC_MAX_LOOKUP_BLOCKS: i32 = 16;
/// Native Free's message, including the whitespace its backslash-continued literal keeps.
const INVALID_FREE: &str = "NCollection_AccAllocator::Free:                                 Trying to free an invalid address";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AccBlockCreationError {
    OutOfMemory,
    KeyCollision(usize),
}

/// Native `AlignedSize`: `(value + 3) & ~3`; `None` where the addition would wrap.
fn aligned_size(value: usize) -> Option<usize> {
    Some(value.checked_add(ACC_ALIGN - 1)? & !(ACC_ALIGN - 1))
}

/// Observable state of one block, as the protected `Block` descriptor holds it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccBlockState {
    /// Block start address (`Block::address`).
    pub address: usize,
    /// Bytes still free below the allocation start (`Block::FreeSize`).
    pub free_size: usize,
    /// Live allocations in the block (`Block::allocCount`).
    pub alloc_count: i32,
}

/// Result of the protected `findBlock` operation; `key` retains the native output-key value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccBlockLookup {
    /// Key written by native `findBlock`, including its previous-key fallback on a miss.
    pub key: AccBlockKey,
    /// Snapshot of the matched block, if the address belongs to an allocated block.
    pub block: Option<AccBlockState>,
}

/// Native `Block`: allocation grows down from `alloc_start` towards `address`.
struct AccBlockSlot {
    payload: NonNull<u8>,
    descriptor: AccBlock,
}

impl AccBlockSlot {
    fn new(address: NonNull<u8>, size: usize, prev_block: Option<usize>) -> Self {
        let descriptor = AccBlock::with_previous(address.as_ptr().addr(), size, prev_block);
        Self {
            payload: address,
            descriptor,
        }
    }
    fn start(&self) -> usize {
        self.descriptor.address
    }
    /// Delegates cursor initialization to the same public record operation.
    fn set_free_size(&mut self, size: usize) {
        self.descriptor.set_free_size(size);
    }
    fn free_size(&self) -> usize {
        self.descriptor.free_size()
    }
    fn state(&self) -> AccBlockState {
        AccBlockState {
            address: self.start(),
            free_size: self.free_size(),
            alloc_count: self.descriptor.alloc_count,
        }
    }
    fn contains_owned(&self, address: usize) -> bool {
        // SAFETY: this slot retains the live original raw_heap pointer, including its header.
        let capacity = unsafe { heap_payload_size(self.payload) };
        address >= self.start() && address - self.start() <= capacity
    }
    fn reservation(&self, size: usize, align: usize) -> Option<usize> {
        let cursor = self.descriptor.alloc_start.as_ptr().addr();
        let aligned_start = cursor.checked_sub(size)? & !(align - 1);
        (aligned_start >= self.start()).then_some(cursor - aligned_start)
    }
    fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        let cursor = self.descriptor.allocate(AccAlignedSize::new(size));
        // SAFETY: selection guarantees the descriptor cursor stays within the owned block; the
        // slot's payload pointer supplies allocation provenance for the native numeric address.
        NonNull::new(self.payload.as_ptr().with_addr(cursor.as_ptr().addr()))
    }
}

/// The allocator state behind `NCollection_AccAllocator`.
pub(super) struct AccEngine {
    block_size: usize,
    last_block: Option<usize>,
    blocks: HashMap<usize, AccBlockSlot>,
}

// SAFETY: the engine exclusively owns every block allocation; it is only reached through a Mutex.
unsafe impl Send for AccEngine {}

impl AccEngine {
    /// Native constructor: rounds the block size and allocates the first block at once.
    ///
    /// Upstream divides by the block size to find blocks, so a zero size crashes natively;
    /// Rust refuses it with `Standard_ProgramError`.
    pub(super) fn new(block_size: usize) -> Result<Self, Failure> {
        let size = aligned_size(block_size).filter(|size| *size != 0).ok_or_else(|| {
            Failure::with_type(
                "Standard_ProgramError",
                &format!("NCollection_AccAllocator block size {block_size}: expected 1..=usize::MAX-3"),
                "",
            )
        })?;
        let mut engine = Self {
            block_size: size,
            last_block: None,
            blocks: HashMap::new(),
        };
        engine.allocate_new_block(size).ok_or_else(|| {
            Failure::with_type(
                "Standard_OutOfMemory",
                &format!("block of {size} bytes"),
                "",
            )
        })?;
        Ok(engine)
    }

    fn key(&self, address: usize) -> usize {
        address / self.block_size
    }

    /// `Allocate`: last block, a dedicated block for oversize requests, or a scan of the
    /// previous `MaxLookupBlocks - 1` blocks before a new block; `None` only on exhaustion.
    pub(super) fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        self.allocate_aligned(size, ACC_ALIGN)
    }

    /// Reserves only actual downward padding; aligned native geometries consume native bytes.
    pub(super) fn allocate_aligned(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let size = aligned_size(size)?;
        let align = align.max(ACC_ALIGN);
        let key = self.select_block(size, align)?;
        let slot = self.blocks.get_mut(&key).expect("selected block is bound");
        let reservation = slot.reservation(size, align)?;
        slot.allocate(reservation)
    }

    fn select_block(&mut self, size: usize, align: usize) -> Option<usize> {
        let last = self
            .last_block
            .expect("an AccAllocator always owns a block");
        if self.blocks[&last].reservation(size, align).is_some() {
            return Some(last);
        }
        if size > self.block_size {
            return self.allocate_aligned_block(size, align);
        }
        match self.lookup_previous(last, size, align) {
            Some(key) => Some(key),
            None => self.allocate_aligned_block(size, align),
        }
    }

    fn allocate_aligned_block(&mut self, size: usize, align: usize) -> Option<usize> {
        // raw_heap's Layout and header offset guarantee MALLOC_ALIGNMENT for every payload.
        let capacity = size.checked_add(align.saturating_sub(MALLOC_ALIGNMENT))?;
        self.allocate_new_block(capacity.max(self.block_size))
    }

    /// The native `for (b = last->prev; b && --rest; b = b->prev)` scan.
    fn lookup_previous(&self, last: usize, size: usize, align: usize) -> Option<usize> {
        let mut rest = ACC_MAX_LOOKUP_BLOCKS;
        let mut iterator = self.blocks[&last].descriptor.prev_block;
        while let Some(key) = iterator {
            rest -= 1;
            if rest == 0 {
                return None;
            }
            if self.blocks[&key].reservation(size, align).is_some() {
                return Some(key);
            }
            iterator = self.blocks[&key].descriptor.prev_block;
        }
        None
    }

    /// `allocateNewBlock`: `Standard::Allocate` (zeroed) of the rounded size, bound by key.
    fn allocate_new_block(&mut self, size: usize) -> Option<usize> {
        self.create_new_block(size).ok()
    }

    fn create_new_block(&mut self, size: usize) -> Result<usize, AccBlockCreationError> {
        let rounded = aligned_size(size).ok_or(AccBlockCreationError::OutOfMemory)?;
        let address = heap_allocate(rounded, MALLOC_ALIGNMENT, true)
            .ok_or(AccBlockCreationError::OutOfMemory)?;
        let key = self.key(address.as_ptr() as usize);
        if !self.bind_block(address, size, key) {
            // SAFETY: this colliding allocation was never inserted into the engine map.
            unsafe { heap_free(address) };
            return Err(AccBlockCreationError::KeyCollision(key));
        }
        Ok(key)
    }

    fn bind_block(&mut self, address: NonNull<u8>, size: usize, key: usize) -> bool {
        if self.blocks.contains_key(&key) {
            return false;
        }
        let block = AccBlockSlot::new(address, size, self.last_block);
        self.blocks.insert(key, block);
        self.last_block = Some(key);
        true
    }

    /// `findBlock`: the block keyed by the address, or by the key before it.
    fn find_block(&self, address: usize) -> Option<usize> {
        self.lookup_block(address).1
    }

    /// Mirrors both the native output key and returned block identity.
    pub(super) fn lookup_block(&self, address: usize) -> (usize, Option<usize>) {
        let key = self.key(address);
        if self
            .blocks
            .get(&key)
            .is_some_and(|block| block.start() <= address)
        {
            return (key, Some(key));
        }
        let previous = key.wrapping_sub(1);
        let found = self
            .blocks
            .get(&previous)
            .filter(|block| block.start().wrapping_add(self.block_size) > address)
            .map(|_| previous);
        (previous, found)
    }

    /// Allocates from the requested existing block while preserving its owned pointer provenance.
    pub(super) fn allocate_from_block(
        &mut self,
        block_address: usize,
        size: usize,
    ) -> Option<NonNull<u8>> {
        let size = aligned_size(size)?;
        let key = self.find_block(block_address)?;
        let block = self.blocks.get_mut(&key)?;
        if size > block.free_size() {
            return None;
        }
        block.allocate(size)
    }

    /// Reports the block created by native `allocateNewBlock` or a native out-of-memory failure.
    pub(super) fn allocate_new_block_state(
        &mut self,
        size: usize,
    ) -> Result<AccBlockState, Failure> {
        let key = self.create_new_block(size).map_err(|error| match error {
            AccBlockCreationError::OutOfMemory => Failure::with_type(
                "Standard_OutOfMemory",
                &format!("block of {size} bytes"),
                "",
            ),
            AccBlockCreationError::KeyCollision(key) => Failure::with_type(
                "Standard_ProgramError",
                &format!("NCollection_AccAllocator block key {key} is already bound"),
                "",
            ),
        })?;
        Ok(self.blocks[&key].state())
    }

    /// Returns an internal block snapshot by its map key.
    pub(super) fn block_state_for_key(&self, key: usize) -> AccBlockState {
        self.blocks[&key].state()
    }

    /// The protected `findBlock` as observable state; `None` for a foreign address.
    pub(super) fn block_state(&self, address: usize) -> Option<AccBlockState> {
        self.find_block(address)
            .map(|key| self.blocks[&key].state())
    }

    /// Every block from the last one back along `prevBlock`.
    pub(super) fn chain(&self) -> Vec<AccBlockState> {
        let mut states = Vec::new();
        let mut iterator = self.last_block;
        while let Some(key) = iterator {
            states.push(self.blocks[&key].state());
            iterator = self.blocks[&key].descriptor.prev_block;
        }
        states
    }

    /// The block `Free(address)` would count down, or its `Standard_ProgramError`: no block
    /// owns the address (a null address, or a zero-size allocation at a fresh block's end,
    /// which `findBlock` cannot reach) or the block has no live allocation.
    pub(super) fn freeable_block(&self, address: usize) -> Result<usize, Failure> {
        self.find_block(address)
            .filter(|key| self.blocks[key].descriptor.alloc_count != 0)
            .ok_or_else(|| Failure::with_type("Standard_ProgramError", INVALID_FREE, ""))
    }

    /// `Free`: count the release and give an emptied block back (or shrink the last one).
    ///
    /// Releasing may free a block, so the public entry points keep the counts honest: RAII
    /// owners free exactly once, and `BaseAllocator::free_raw` is `unsafe`.
    pub(super) fn free(&mut self, address: usize) -> Result<(), Failure> {
        let key = self.freeable_block(address)?;
        self.release_counted_block(key);
        Ok(())
    }

    /// Rust-owned pointers may exceed native findBlock's window or equal a zero-byte endpoint.
    pub(super) fn free_owned(&mut self, address: usize) -> Result<(), Failure> {
        let key = self
            .find_block(address)
            .filter(|key| self.blocks[key].contains_owned(address));
        let key = key
            .or_else(|| {
                self.blocks
                    .iter()
                    .find_map(|(&key, slot)| slot.contains_owned(address).then_some(key))
            })
            .filter(|key| self.blocks[key].descriptor.alloc_count != 0)
            .ok_or_else(|| Failure::with_type("Standard_ProgramError", INVALID_FREE, ""))?;
        self.release_counted_block(key);
        Ok(())
    }

    fn release_counted_block(&mut self, key: usize) {
        let block = self.blocks.get_mut(&key).expect("found block is bound");
        block.descriptor.free();
        if block.descriptor.alloc_count != 0 {
            return;
        }
        if self.blocks.len() > 1 {
            self.release_block(key);
        } else {
            self.shrink_sole_block(key);
        }
    }

    /// Frees an empty block and unlinks it from the `prevBlock` chain.
    fn release_block(&mut self, key: usize) {
        let block = self.blocks.remove(&key).expect("released block is bound");
        // SAFETY: the block payload came from heap_allocate/heap_reallocate and is freed once.
        unsafe { heap_free(block.payload) };
        if self.last_block == Some(key) {
            self.last_block = block.descriptor.prev_block;
            return;
        }
        let mut iterator = self.last_block;
        while let Some(current) = iterator {
            let slot = self
                .blocks
                .get_mut(&current)
                .expect("chained block is bound");
            if slot.descriptor.prev_block == Some(key) {
                slot.descriptor.prev_block = block.descriptor.prev_block;
                return;
            }
            iterator = slot.descriptor.prev_block;
        }
    }

    /// The last block empties: reallocate it to the default size, rebinding on a new key.
    fn shrink_sole_block(&mut self, key: usize) {
        let rounded =
            aligned_size(self.block_size).expect("block size was rounded at construction");
        let address = self.blocks[&key].payload;
        // SAFETY: the block payload is live and owned by this engine.
        let Some(moved) = (unsafe { heap_reallocate(address, rounded) }) else {
            return; // native realloc failure throws before touching the block
        };
        let new_key = self.key(moved.as_ptr() as usize);
        let block = self.blocks.get_mut(&key).expect("sole block is bound");
        if moved == address || new_key == key {
            block.payload = moved;
            block.descriptor.address = moved.as_ptr() as usize;
            block.set_free_size(self.block_size);
            return;
        }
        self.blocks.clear();
        self.blocks
            .insert(new_key, AccBlockSlot::new(moved, self.block_size, None));
        self.last_block = Some(new_key);
    }
}

impl Drop for AccEngine {
    fn drop(&mut self) {
        for (_, block) in self.blocks.drain() {
            // SAFETY: each payload is owned by this engine and freed exactly once here.
            unsafe { heap_free(block.payload) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ACC_ALIGN, AccBlockSlot, AccEngine};
    use super::{MALLOC_ALIGNMENT, heap_allocate};
    use std::{collections::HashMap, ptr::NonNull};

    #[test]
    fn duplicate_block_key_does_not_replace_or_relink_existing_block() {
        let key = 17;
        let existing_address = heap_allocate(16, MALLOC_ALIGNMENT, true).unwrap();
        let existing = AccBlockSlot::new(existing_address, 16, None);
        let mut engine = AccEngine {
            block_size: 16,
            last_block: Some(key),
            blocks: HashMap::from([(key, existing)]),
        };

        assert!(!engine.bind_block(NonNull::dangling(), ACC_ALIGN * 2, key));
        assert_eq!(engine.last_block, Some(key));
        assert_eq!(engine.blocks.len(), 1);
        assert_eq!(engine.blocks[&key].descriptor.alloc_count, 0);
    }
}

#[cfg(test)]
#[path = "accumulating/alignment_tests.rs"]
mod alignment_tests;
