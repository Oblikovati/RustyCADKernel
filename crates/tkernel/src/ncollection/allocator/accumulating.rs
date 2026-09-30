//! NCollection_AccAllocator's address-keyed blocks, filled downwards and freed when empty.
//!
//! Blocks are found by `address / block_size` exactly as upstream; the map stores block
//! descriptors by key and the `prevBlock` chain links keys instead of descriptor pointers.
use super::raw_heap::{MALLOC_ALIGNMENT, heap_allocate, heap_free, heap_reallocate};
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

/// Native `Block`: allocation grows down from `alloc_start` towards `address`.
struct AccBlock {
    address: NonNull<u8>,
    alloc_start: usize,
    prev_block: Option<usize>,
    alloc_count: i32,
}

impl AccBlock {
    fn new(address: NonNull<u8>, size: usize, prev_block: Option<usize>) -> Self {
        let mut block = Self {
            address,
            alloc_start: 0,
            prev_block,
            alloc_count: 0,
        };
        block.set_free_size(size);
        block
    }
    fn start(&self) -> usize {
        self.address.as_ptr() as usize
    }
    /// `SetFreeSize`: the AlignedPtr constructor masks the new start down to 4 bytes.
    fn set_free_size(&mut self, size: usize) {
        self.alloc_start = self.start().wrapping_add(size) & !(ACC_ALIGN - 1);
    }
    fn free_size(&self) -> usize {
        self.alloc_start - self.start()
    }
    fn state(&self) -> AccBlockState {
        AccBlockState {
            address: self.start(),
            free_size: self.free_size(),
            alloc_count: self.alloc_count,
        }
    }
}

/// The allocator state behind `NCollection_AccAllocator`.
pub(super) struct AccEngine {
    block_size: usize,
    last_block: Option<usize>,
    blocks: HashMap<usize, AccBlock>,
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
        let size = aligned_size(size)?;
        let key = self.select_block(size)?;
        let block = self.blocks.get_mut(&key).expect("selected block is bound");
        block.alloc_count += 1;
        block.alloc_start -= size;
        let offset = block.alloc_start - block.start();
        // SAFETY: alloc_start stays within [address, address + block size].
        Some(unsafe { block.address.add(offset) })
    }

    /// Allocation honouring a Rust layout alignment (typed allocations only).
    ///
    /// Upstream guarantees 4 bytes; a stricter alignment reserves `align - 4` extra bytes and
    /// aligns the address up inside the reservation, so the block still owns the address.
    pub(super) fn allocate_aligned(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        if align <= ACC_ALIGN {
            return self.allocate(size);
        }
        let raw = self.allocate(aligned_size(size)?.checked_add(align - ACC_ALIGN)?)?;
        let offset = raw.as_ptr().align_offset(align);
        // SAFETY: the reservation starts 4-aligned, so `offset <= align - 4` bytes of it pad.
        Some(unsafe { raw.add(offset) })
    }

    fn select_block(&mut self, size: usize) -> Option<usize> {
        let last = self
            .last_block
            .expect("an AccAllocator always owns a block");
        if size <= self.blocks[&last].free_size() {
            return Some(last);
        }
        if size > self.block_size {
            return self.allocate_new_block(size);
        }
        match self.lookup_previous(last, size) {
            Some(key) => Some(key),
            None => self.allocate_new_block(self.block_size),
        }
    }

    /// The native `for (b = last->prev; b && --rest; b = b->prev)` scan.
    fn lookup_previous(&self, last: usize, size: usize) -> Option<usize> {
        let mut rest = ACC_MAX_LOOKUP_BLOCKS;
        let mut iterator = self.blocks[&last].prev_block;
        while let Some(key) = iterator {
            rest -= 1;
            if rest == 0 {
                return None;
            }
            if size <= self.blocks[&key].free_size() {
                return Some(key);
            }
            iterator = self.blocks[&key].prev_block;
        }
        None
    }

    /// `allocateNewBlock`: `Standard::Allocate` (zeroed) of the rounded size, bound by key.
    fn allocate_new_block(&mut self, size: usize) -> Option<usize> {
        let rounded = aligned_size(size)?;
        let address = heap_allocate(rounded, MALLOC_ALIGNMENT, true)?;
        let key = self.key(address.as_ptr() as usize);
        let block = AccBlock::new(address, size, self.last_block);
        self.blocks.insert(key, block);
        self.last_block = Some(key);
        Some(key)
    }

    /// `findBlock`: the block keyed by the address, or by the key before it.
    fn find_block(&self, address: usize) -> Option<usize> {
        let key = self.key(address);
        if self
            .blocks
            .get(&key)
            .is_some_and(|block| block.start() <= address)
        {
            return Some(key);
        }
        let previous = key.checked_sub(1)?;
        let block = self.blocks.get(&previous)?;
        (block.start().wrapping_add(self.block_size) > address).then_some(previous)
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
            iterator = self.blocks[&key].prev_block;
        }
        states
    }

    /// The block `Free(address)` would count down, or its `Standard_ProgramError`: no block
    /// owns the address (a null address, or a zero-size allocation at a fresh block's end,
    /// which `findBlock` cannot reach) or the block has no live allocation.
    pub(super) fn freeable_block(&self, address: usize) -> Result<usize, Failure> {
        self.find_block(address)
            .filter(|key| self.blocks[key].alloc_count != 0)
            .ok_or_else(|| Failure::with_type("Standard_ProgramError", INVALID_FREE, ""))
    }

    /// `Free`: count the release and give an emptied block back (or shrink the last one).
    ///
    /// Releasing may free a block, so the public entry points keep the counts honest: RAII
    /// owners free exactly once, and `BaseAllocator::free_raw` is `unsafe`.
    pub(super) fn free(&mut self, address: usize) -> Result<(), Failure> {
        let key = self.freeable_block(address)?;
        let block = self.blocks.get_mut(&key).expect("found block is bound");
        block.alloc_count -= 1;
        if block.alloc_count != 0 {
            return Ok(());
        }
        if self.blocks.len() > 1 {
            self.release_block(key);
        } else {
            self.shrink_sole_block(key);
        }
        Ok(())
    }

    /// Frees an empty block and unlinks it from the `prevBlock` chain.
    fn release_block(&mut self, key: usize) {
        let block = self.blocks.remove(&key).expect("released block is bound");
        // SAFETY: the block payload came from heap_allocate/heap_reallocate and is freed once.
        unsafe { heap_free(block.address) };
        if self.last_block == Some(key) {
            self.last_block = block.prev_block;
            return;
        }
        let mut iterator = self.last_block;
        while let Some(current) = iterator {
            let slot = self
                .blocks
                .get_mut(&current)
                .expect("chained block is bound");
            if slot.prev_block == Some(key) {
                slot.prev_block = block.prev_block;
                return;
            }
            iterator = slot.prev_block;
        }
    }

    /// The last block empties: reallocate it to the default size, rebinding on a new key.
    fn shrink_sole_block(&mut self, key: usize) {
        let rounded =
            aligned_size(self.block_size).expect("block size was rounded at construction");
        let address = self.blocks[&key].address;
        // SAFETY: the block payload is live and owned by this engine.
        let Some(moved) = (unsafe { heap_reallocate(address, rounded) }) else {
            return; // native realloc failure throws before touching the block
        };
        let new_key = self.key(moved.as_ptr() as usize);
        let block = self.blocks.get_mut(&key).expect("sole block is bound");
        if moved == address || new_key == key {
            block.address = moved;
            block.set_free_size(self.block_size);
            return;
        }
        self.blocks.clear();
        self.blocks
            .insert(new_key, AccBlock::new(moved, self.block_size, None));
        self.last_block = Some(new_key);
    }
}

impl Drop for AccEngine {
    fn drop(&mut self) {
        for (_, block) in self.blocks.drain() {
            // SAFETY: each payload is owned by this engine and freed exactly once here.
            unsafe { heap_free(block.address) };
        }
    }
}
