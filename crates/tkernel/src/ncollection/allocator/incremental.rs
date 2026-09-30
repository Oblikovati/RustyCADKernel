//! NCollection_IncAllocator's block bookkeeping: sorted allocation heap, used heap, growth levels.
//!
//! Linked lists of `IBlock*` become lists of block indices; each block's payload is one
//! `raw_heap` allocation, exactly as the native block is one `Standard::AllocateOptimal` call.
use super::raw_heap::{MALLOC_ALIGNMENT, heap_allocate, heap_free};
use crate::standard::Failure;
use std::ptr::NonNull;

/// `NCollection_IncAllocator::THE_DEFAULT_BLOCK_SIZE` (12 KB).
pub const INC_DEFAULT_BLOCK_SIZE: usize = 1024 * 12;
/// `NCollection_IncAllocator::THE_MINIMUM_BLOCK_SIZE`; smaller requests fall back to the default.
pub const INC_MINIMUM_BLOCK_SIZE: usize = 1024 * 2;
const SMALL_BOUND_BLOCK_SIZE: u32 = (INC_DEFAULT_BLOCK_SIZE * 16) as u32; // 196 KB
const MEDIUM_BOUND_BLOCK_SIZE: u32 = (INC_DEFAULT_BLOCK_SIZE * 64) as u32; // 786 KB
const LARGE_BOUND_BLOCK_SIZE: u32 = (INC_DEFAULT_BLOCK_SIZE * 1024) as u32; // 12 MB
/// Blocks with less space than this leave the allocation heap (native literal 16).
const USED_BLOCK_THRESHOLD: usize = 16;

/// `NCollection_IncAllocator::IBlockSizeLevel`: how the next block grows, every fifth block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IncBlockSizeLevel {
    /// Below 12 KB: x8 growing.
    Min,
    /// Below 196 KB: x4 growing.
    Small,
    /// Below 786 KB: x2 growing.
    Medium,
    /// Below 12 MB: x1.5 growing.
    Large,
    /// No growing.
    Max,
}

impl IncBlockSizeLevel {
    /// The level of a block size, as the native file-local `computeLevel`.
    ///
    /// Example: `IncBlockSizeLevel::of(12 * 1024) == IncBlockSizeLevel::Small`.
    pub fn of(block_size: u32) -> Self {
        match block_size {
            size if size < INC_DEFAULT_BLOCK_SIZE as u32 => Self::Min,
            size if size < SMALL_BOUND_BLOCK_SIZE => Self::Small,
            size if size < MEDIUM_BOUND_BLOCK_SIZE => Self::Medium,
            size if size < LARGE_BOUND_BLOCK_SIZE => Self::Large,
            _ => Self::Max,
        }
    }

    /// The size of the next block at this level (`increaseBlockSize`).
    ///
    /// Example: `IncBlockSizeLevel::Min.grow(4096) == 32768`.
    pub fn grow(self, block_size: u32) -> u32 {
        match self {
            Self::Min => block_size.wrapping_mul(8),
            Self::Small => block_size.wrapping_mul(4),
            Self::Medium => block_size.wrapping_mul(2),
            // std::lround rounds halves away from zero, as f64::round does.
            Self::Large => (f64::from(block_size) * 1.5).round() as u32,
            Self::Max => block_size,
        }
    }
}

/// `NCollection_IncAllocator::IBlock`: bump cursor and remaining space of one block.
///
/// The native `CurPointer` is expressed as `cursor`, a byte offset from the block payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncBlock {
    /// Offset of the next allocation from the payload start.
    pub cursor: usize,
    /// Bytes still available after `cursor`.
    pub available_size: usize,
}

impl IncBlock {
    /// A fresh block of `size` payload bytes (native `IBlock(void*, size_t)`).
    ///
    /// Example: `IncBlock::new(64).available_size == 64`.
    pub fn new(size: usize) -> Self {
        Self {
            cursor: 0,
            available_size: size,
        }
    }
}

/// One block: payload, bump state and the two intrusive list links.
struct BlockSlot {
    payload: NonNull<u8>,
    state: IncBlock,
    next_block: Option<usize>,
    next_ordered_block: Option<usize>,
}

/// The allocator state behind `NCollection_IncAllocator`.
pub(super) struct IncEngine {
    block_size: u32,
    block_count: u32,
    blocks: Vec<BlockSlot>,
    allocation_heap: Option<usize>,
    used_heap: Option<usize>,
    ordered_blocks: Option<usize>,
    /// Rust-only count of RAII/typed allocations still borrowed; guards `reset`.
    live: usize,
}

// SAFETY: the engine exclusively owns every block payload; it is only reached through a Mutex.
unsafe impl Send for IncEngine {}

impl IncEngine {
    /// Native constructor: sizes below the minimum select the 12 KB default.
    pub(super) fn new(block_size: usize) -> Self {
        let size = if block_size < INC_MINIMUM_BLOCK_SIZE {
            INC_DEFAULT_BLOCK_SIZE
        } else {
            block_size
        };
        Self {
            block_size: size as u32, // static_cast<unsigned>, truncating like the native member
            block_count: 0,
            blocks: Vec::new(),
            allocation_heap: None,
            used_heap: None,
            ordered_blocks: None,
            live: 0,
        }
    }

    /// `AllocateOptimal`/`Allocate`: bump `size` bytes; `None` only on exhaustion.
    pub(super) fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        let block = match self.allocation_heap {
            Some(head) if self.blocks[head].state.available_size >= size => head,
            _ => self.push_block(size)?,
        };
        let slot = &mut self.blocks[block];
        // SAFETY: cursor + size <= block payload size, by the available-size check above.
        let address = unsafe { slot.payload.add(slot.state.cursor) };
        slot.state.cursor += size;
        slot.state.available_size -= size;
        self.reorder(block);
        Some(address)
    }

    /// Allocation honouring a Rust layout alignment (typed allocations only).
    ///
    /// Payloads start 16-byte aligned, so alignments up to 16 with sizes that are multiples of
    /// the alignment need no padding and match the native byte offsets exactly.
    pub(super) fn allocate_aligned(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let padding = self.head_padding(size, align);
        let request = size.checked_add(padding)?;
        let raw = self.allocate(request)?;
        let offset = raw.as_ptr().align_offset(align);
        // SAFETY: `request` reserves `offset + size` bytes: `padding` is exact for the head
        // block, and a new block starts 16-aligned so it needs at most `align - 16` bytes.
        Some(unsafe { raw.add(offset) })
    }

    /// Bytes needed to align the next allocation of the head block, or of a new block.
    fn head_padding(&self, size: usize, align: usize) -> usize {
        let fresh_padding = align.saturating_sub(MALLOC_ALIGNMENT);
        let Some(head) = self.allocation_heap else {
            return fresh_padding;
        };
        let slot = &self.blocks[head];
        let padding = slot
            .payload
            .as_ptr()
            .wrapping_add(slot.state.cursor)
            .align_offset(align);
        match size.checked_add(padding) {
            Some(total) if total <= slot.state.available_size => padding,
            _ => fresh_padding,
        }
    }

    /// Allocates a new block from the heap and makes it the allocation-heap head.
    fn push_block(&mut self, size: usize) -> Option<usize> {
        let requested = u32::try_from(size).ok()?; // native truncation would corrupt the block
        self.block_count = self.block_count.wrapping_add(1);
        if self.block_count.is_multiple_of(5) {
            self.block_size = IncBlockSizeLevel::of(self.block_size).grow(self.block_size);
        }
        self.block_size = self.block_size.max(requested);
        let payload_size = self.block_size as usize;
        let payload = heap_allocate(payload_size, MALLOC_ALIGNMENT, true)?;
        self.blocks.push(BlockSlot {
            payload,
            state: IncBlock::new(payload_size),
            next_block: self.allocation_heap,
            next_ordered_block: self.ordered_blocks,
        });
        let index = self.blocks.len() - 1;
        self.ordered_blocks = Some(index);
        self.allocation_heap = Some(index);
        Some(index)
    }

    /// Keeps the allocation heap sorted by decreasing space; nearly full blocks become used.
    fn reorder(&mut self, block: usize) {
        if self.blocks[block].state.available_size < USED_BLOCK_THRESHOLD {
            self.allocation_heap = self.blocks[block].next_block;
            self.blocks[block].next_block = self.used_heap;
            self.used_heap = Some(block);
            return;
        }
        let Some(after) = self.insertion_point(block) else {
            return;
        };
        let next = self.blocks[after].next_block;
        self.blocks[after].next_block = Some(block);
        self.allocation_heap = self.blocks[block].next_block;
        self.blocks[block].next_block = next;
    }

    /// The last heap block with strictly more space than `block`, searching after it.
    fn insertion_point(&self, block: usize) -> Option<usize> {
        let available = self.blocks[block].state.available_size;
        let mut iterator = self.blocks[block].next_block;
        let mut after = None;
        while let Some(index) = iterator {
            if self.blocks[index].state.available_size <= available {
                break;
            }
            after = Some(index);
            iterator = self.blocks[index].next_block;
        }
        after
    }

    /// `Reset`: rewind every block (release=false) or free them all (release=true).
    ///
    /// Rust refuses while RAII or typed allocations are alive: rewinding would alias them.
    pub(super) fn reset(&mut self, release: bool) -> Result<(), Failure> {
        if self.live != 0 {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                &format!(
                    "{} live allocations: expected 0 before NCollection_IncAllocator::Reset",
                    self.live
                ),
                "",
            ));
        }
        if release {
            self.clean();
            return Ok(());
        }
        self.rewind();
        Ok(())
    }

    /// Relinks the allocation heap in creation-reverse (ordered) order with empty blocks.
    fn rewind(&mut self) {
        let mut iterator = self.ordered_blocks;
        while let Some(index) = iterator {
            let slot = &mut self.blocks[index];
            iterator = slot.next_ordered_block;
            slot.next_block = iterator;
            slot.state.available_size += slot.state.cursor;
            slot.state.cursor = 0;
        }
        self.allocation_heap = self.ordered_blocks;
        self.used_heap = None;
    }

    /// Native `clean()`: frees every block and, as upstream does, forgets the constructor's
    /// block size in favour of the 12 KB default.
    fn clean(&mut self) {
        for slot in self.blocks.drain(..) {
            // SAFETY: each payload came from heap_allocate and is freed exactly once here.
            unsafe { heap_free(slot.payload) };
        }
        self.ordered_blocks = None;
        self.allocation_heap = None;
        self.used_heap = None;
        self.block_count = 0;
        self.block_size = INC_DEFAULT_BLOCK_SIZE as u32;
    }

    /// Records one RAII/typed allocation.
    pub(super) fn acquire(&mut self) {
        self.live += 1;
    }

    /// Releases one RAII/typed allocation (native `Free` does nothing else).
    pub(super) fn release(&mut self) {
        self.live = self
            .live
            .checked_sub(1)
            .expect("deallocation requires a live incremental allocation");
    }
}

impl Drop for IncEngine {
    fn drop(&mut self) {
        self.clean();
    }
}
