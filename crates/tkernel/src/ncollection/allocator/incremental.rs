//! NCollection_IncAllocator's block bookkeeping: sorted allocation heap, used heap, growth levels.
//!
//! Actual stable `IBlock` headers and intrusive links are the sole block topology.
//! Each header and its payload share one `raw_heap` allocation, as native `AllocateOptimal`.
use super::raw_heap::MALLOC_ALIGNMENT;
mod block;
mod operations;
use crate::standard::Failure;
use block::IncBlockLink;
pub use block::{IncAllocatorBlock, IncBlockObservation};
pub use operations::IncAllocatorObservation;
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

/// The allocator state behind `NCollection_IncAllocator`.
pub(super) struct IncEngine {
    block_size: u32,
    block_count: u32,
    thread_safe_policy: bool,
    allocation_heap: Option<IncBlockLink>,
    used_heap: Option<IncBlockLink>,
    ordered_blocks: Option<IncBlockLink>,
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
            thread_safe_policy: false,
            allocation_heap: None,
            used_heap: None,
            ordered_blocks: None,
            live: 0,
        }
    }

    /// Keep native fast-path topology while the outer Rust mutex protects all modes.
    pub(super) fn set_thread_safe(&mut self, enabled: bool) {
        self.thread_safe_policy = enabled;
    }

    /// `AllocateOptimal`/`Allocate`: enabled native fast paths leave list order unchanged.
    pub(super) fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        if self.thread_safe_policy
            && let Some(head) = self.available_head(size)
        {
            return Some(head.bump(size));
        }
        self.allocate_slow(size)
    }

    /// Protected allocateSlow always repairs lists, even when public allocation enables fast bumps.
    pub(super) fn allocate_slow(&mut self, size: usize) -> Option<NonNull<u8>> {
        let block = match self.available_head(size) {
            Some(head) => head,
            None => self.push_block(size)?,
        };
        let address = block.bump(size);
        self.reorder(block);
        Some(address)
    }

    fn available_head(&self, size: usize) -> Option<IncBlockLink> {
        self.allocation_heap
            .filter(|&head| head.available() >= size)
    }

    /// Protected increaseBlockSize updates the real capacity used for the next allocated block.
    pub(super) fn increase_block_size(&mut self) {
        self.block_size = IncBlockSizeLevel::of(self.block_size).grow(self.block_size);
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
        let padding = head.current().as_ptr().align_offset(align);
        match size.checked_add(padding) {
            Some(total) if total <= head.available() => padding,
            _ => fresh_padding,
        }
    }

    /// Allocates a new block from the heap and makes it the allocation-heap head.
    fn push_block(&mut self, size: usize) -> Option<IncBlockLink> {
        // The source seam also proves native state retained when the underlying allocation fails.
        self.push_block_with(size, IncBlockLink::allocate)
    }

    fn push_block_with(
        &mut self,
        size: usize,
        allocate: impl FnOnce(usize) -> Option<IncBlockLink>,
    ) -> Option<IncBlockLink> {
        let requested = u32::try_from(size).ok()?; // native truncation would corrupt the block
        self.block_count = self.block_count.wrapping_add(1);
        if self.block_count.is_multiple_of(5) {
            self.increase_block_size();
        }
        self.block_size = self.block_size.max(requested);
        let block = allocate(self.block_size as usize)?;
        block.set_next(self.allocation_heap);
        block.set_ordered_next(self.ordered_blocks);
        self.ordered_blocks = Some(block);
        self.allocation_heap = Some(block);
        Some(block)
    }

    /// Keeps the allocation heap sorted by decreasing space; nearly full blocks become used.
    fn reorder(&mut self, block: IncBlockLink) {
        if block.available() < USED_BLOCK_THRESHOLD {
            self.allocation_heap = block.next();
            block.set_next(self.used_heap);
            self.used_heap = Some(block);
            return;
        }
        let Some(after) = self.insertion_point(block) else {
            return;
        };
        let next = after.next();
        after.set_next(Some(block));
        self.allocation_heap = block.next();
        block.set_next(next);
    }

    /// The last heap block with strictly more space than `block`, searching after it.
    fn insertion_point(&self, block: IncBlockLink) -> Option<IncBlockLink> {
        let available = block.available();
        let mut iterator = block.next();
        let mut after = None;
        while let Some(index) = iterator {
            if index.available() <= available {
                break;
            }
            after = Some(index);
            iterator = index.next();
        }
        after
    }

    /// `Reset`: rewind every block (release=false) or free them all (release=true).
    ///
    /// Rust refuses while RAII or typed allocations are alive: rewinding would alias them.
    pub(super) fn reset(&mut self, release: bool) -> Result<(), Failure> {
        self.require_no_live("Reset")?;
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
            iterator = index.ordered_next();
            index.set_next(iterator);
            index.reset();
        }
        self.allocation_heap = self.ordered_blocks;
        self.used_heap = None;
    }

    /// Native `clean()`: frees every block and, as upstream does, forgets the constructor's
    /// block size in favour of the 12 KB default.
    fn clean(&mut self) {
        let mut iterator = self.ordered_blocks;
        while let Some(block) = iterator {
            iterator = block.ordered_next();
            // SAFETY: no payload owner remains; ordered links visit each allocation exactly once.
            unsafe { block.release() };
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

#[cfg(test)]
#[path = "incremental/protected_tests.rs"]
mod protected_tests;

#[cfg(test)]
mod failure_tests;
