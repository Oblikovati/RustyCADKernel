// Rust translation of FoundationClasses/TKernel/Standard/Standard_MMgrOpt.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT's speed-optimized memory manager: size-rounded blocks with an 8-byte header,
//! small blocks carved from page pools, medium blocks recycled through free lists and
//! large blocks mapped directly.
use super::mmgr_root::{MMgrRoot, StorageAddress};
use super::{Failure, exceptions};
use address_space::{AddressSpace, WORD};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::RwLock;
mod address_space;
mod free_pools;
mod os_page;
mod pools;
mod purge;

/// Observer of every allocation and release: `(is_alloc, storage, round_size, size)`, where
/// `size` is the requested size for an allocation and 0 for a release;
/// `MMgrOpt::set_call_back_function(Some(log))`.
pub type OptCallBack = fn(bool, StorageAddress, usize, usize);

/// Opaque memory returned by the protected `AllocMemory` hook; `block.allocated_size()`.
/// Its address is only a numeric capability and cannot be dereferenced by safe Rust.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MMgrOptSystemBlock {
    address: StorageAddress,
    allocated_size: usize,
}

impl MMgrOptSystemBlock {
    /// Returns the real allocation base as an opaque address; `block.address().get() != 0`.
    pub fn address(&self) -> StorageAddress {
        self.address
    }

    /// Returns the size written by native `AllocMemory(size_t&)`; `block.allocated_size() >= 37`.
    pub fn allocated_size(&self) -> usize {
        self.allocated_size
    }
}

// Standard_MMgrOpt keeps the callback in one file-static pointer shared by every instance.
static CALL_BACK: RwLock<Option<OptCallBack>> = RwLock::new(None);

/// `Standard_MMgrOpt`; `MMgrOpt::new(true, true, 200, 1000, 40000).allocate(24)?`.
///
/// Every block is `ROUNDUP8(size)` bytes preceded by one `size_t` holding that size, or the
/// next free block while it sits in a free list. Blocks up to the (16-rounded) cell size
/// come from pools of `page_size * nb_pages` bytes; blocks below the threshold are recycled
/// by exact rounded size, newest first; larger blocks are mapped (or malloc'ed) and
/// released at once. Methods take `&mut self`: the native mutexes only serialize callers.
pub struct MMgrOpt {
    clear: bool,
    mmap: bool,
    cell_size: usize,
    nb_pages: i32,
    page_size: usize,
    threshold: usize,
    free_list_max: usize,
    free_lists: BTreeMap<usize, usize>,
    alloc_list: usize,
    next_addr: usize,
    end_block: usize,
    space: AddressSpace,
    live: BTreeSet<usize>,
}

impl MMgrOpt {
    /// Builds a manager and runs `Initialize`; `MMgrOpt::new(true, false, 200, 1000, 40000)`.
    /// Fewer than 100 pages means 1000; the cell size is rounded up to 16 bytes.
    pub fn new(clear: bool, mmap: bool, cell_size: usize, nb_pages: i32, threshold: usize) -> Self {
        let page_size = os_page::os_page_size();
        Self {
            clear,
            // Initialize turns mapping off when the page size is unknown (getpagesize() == 0).
            mmap: mmap && page_size != 0,
            cell_size: roundup16(cell_size),
            nb_pages: if nb_pages < 100 { 1000 } else { nb_pages },
            page_size,
            threshold,
            free_list_max: index_cell(roundup_cell(threshold.wrapping_sub(BLOCK_SHIFT))),
            free_lists: BTreeMap::new(),
            alloc_list: 0,
            next_addr: 0,
            end_block: 0,
            space: AddressSpace::default(),
            live: BTreeSet::new(),
        }
    }

    /// Installs or removes the process-wide observer; `MMgrOpt::set_call_back_function(None)`.
    pub fn set_call_back_function(callback: Option<OptCallBack>) {
        *CALL_BACK
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = callback;
    }

    /// The rounded cell size small blocks may not exceed; `MMgrOpt::default().cell_size() == 208`.
    pub fn cell_size(&self) -> usize {
        self.cell_size
    }

    /// The system page size the pools are measured in; `manager.page_size()`.
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// The pool size in pages after `Initialize`'s correction; `manager.nb_pages() >= 100`.
    pub fn nb_pages(&self) -> i32 {
        self.nb_pages
    }

    /// The size from which blocks are no longer recycled; `manager.threshold() == 40000`.
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// Exposes `AllocMemory` as a checked region; `let block = manager.allocate_system_memory(&mut size)?`.
    /// Mapped requests are page-aligned and update `size`; heap requests retain it. Rust zeroes
    /// heap storage even with `clear=false`, since reading uninitialized bytes is not safe.
    pub fn allocate_system_memory(
        &mut self,
        size: &mut usize,
    ) -> Result<MMgrOptSystemBlock, Failure> {
        let (start, allocated_size) = self.alloc_memory(*size)?;
        let Some(address) = StorageAddress::new(start) else {
            self.free_memory(start);
            return Err(header_overflow(start, *size));
        };
        *size = allocated_size;
        Ok(MMgrOptSystemBlock {
            address,
            allocated_size,
        })
    }

    /// Borrows the whole raw region; `manager.system_memory_bytes(&block)?[0]`.
    pub fn system_memory_bytes(&self, block: &MMgrOptSystemBlock) -> Result<&[u8], Failure> {
        self.validate_system_block(block)?;
        self.space
            .bytes(block.address.get(), block.allocated_size)
            .ok_or_else(|| invalid_system_block(block.address, block.allocated_size))
    }

    /// Borrows the whole raw region for writing; `manager.system_memory_bytes_mut(&block)?[0] = 1`.
    pub fn system_memory_bytes_mut(
        &mut self,
        block: &MMgrOptSystemBlock,
    ) -> Result<&mut [u8], Failure> {
        self.validate_system_block(block)?;
        self.space
            .bytes_mut(block.address.get(), block.allocated_size)
            .ok_or_else(|| invalid_system_block(block.address, block.allocated_size))
    }

    /// Releases a raw region with its exact returned size; `manager.free_system_memory(&block, size)?`.
    /// Native `FreeMemory` permits smaller mapped sizes that partially unmap a region; this safe
    /// facade rejects those calls because its `AddressSpace` owns complete regions.
    pub fn free_system_memory(
        &mut self,
        block: &MMgrOptSystemBlock,
        size: usize,
    ) -> Result<(), Failure> {
        if size != block.allocated_size {
            return Err(invalid_system_size(
                block.address,
                size,
                block.allocated_size,
            ));
        }
        self.validate_system_block(block)?;
        self.free_memory(block.address.get());
        Ok(())
    }

    fn validate_system_block(&self, block: &MMgrOptSystemBlock) -> Result<(), Failure> {
        if self.space.region_len(block.address.get()) == Some(block.allocated_size) {
            return Ok(());
        }
        Err(invalid_system_block(block.address, block.allocated_size))
    }

    /// Allocates `size` bytes, rounded up to 8; `manager.allocate(10)?` holds 16 usable bytes.
    /// A `size` within 7 of `usize::MAX` wraps to a zero-sized pool block, as natively.
    pub fn allocate(&mut self, size: usize) -> Result<StorageAddress, Failure> {
        let round_size = roundup_cell(size);
        let index = index_cell(round_size);
        let storage = if index <= self.free_list_max {
            self.allocate_recyclable(round_size, index)?
        } else {
            self.allocate_large(round_size)?
        };
        let storage = self.hand_out(storage)?;
        notify(true, storage, round_size, size);
        Ok(storage)
    }

    /// Releases a block into its free list, or to the system when it is large;
    /// `manager.free(Some(storage))?`. A null storage is ignored.
    pub fn free(&mut self, storage: Option<StorageAddress>) -> Result<(), Failure> {
        let Some(storage) = storage else {
            return Ok(());
        };
        let round_size = self.claim(storage)?;
        notify(false, storage, round_size, 0);
        let block = storage.get() - WORD * BLOCK_SHIFT;
        let index = index_cell(round_size);
        if index > self.free_list_max {
            self.free_memory(block);
            return Ok(());
        }
        self.push_free(index, block)
    }

    /// Keeps the block when `size` fits its rounded size, otherwise moves it; `reallocate(Some(s), 64)?`.
    /// A moved block copies the old rounded size, frees the old block, then (with clearing)
    /// zeroes `old_round_size..size`.
    pub fn reallocate(
        &mut self,
        storage: Option<StorageAddress>,
        size: usize,
    ) -> Result<StorageAddress, Failure> {
        let Some(storage) = storage else {
            return self.allocate(size);
        };
        let old_size = self.round_size_of(storage)?;
        if size <= old_size {
            return Ok(storage);
        }
        let moved = self.allocate(size)?;
        let contents = self.bytes(storage)?.to_vec();
        self.bytes_mut(moved)?[..old_size].copy_from_slice(&contents);
        self.free(Some(storage))?;
        if self.clear {
            self.bytes_mut(moved)?[old_size..size].fill(0);
        }
        Ok(moved)
    }

    /// Borrows the rounded usable area of a live block; `manager.bytes(storage)?.len() % 8 == 0`.
    pub fn bytes(&self, storage: StorageAddress) -> Result<&[u8], Failure> {
        let round_size = self.round_size_of(storage)?;
        self.space
            .bytes(storage.get(), round_size)
            .ok_or_else(|| foreign_storage(storage))
    }

    /// Borrows the rounded usable area of a live block for writing; `manager.bytes_mut(s)?.fill(1)`.
    pub fn bytes_mut(&mut self, storage: StorageAddress) -> Result<&mut [u8], Failure> {
        let round_size = self.round_size_of(storage)?;
        self.space
            .bytes_mut(storage.get(), round_size)
            .ok_or_else(|| foreign_storage(storage))
    }

    fn allocate_recyclable(&mut self, round_size: usize, index: usize) -> Result<usize, Failure> {
        if let Some(block) = self.pop_free(index)? {
            return self.install(block, round_size, self.clear);
        }
        if round_size <= self.cell_size {
            return self.allocate_in_pool(round_size);
        }
        self.allocate_medium(round_size)
    }

    /// Writes the header and returns the user area, optionally clearing it; `install(block, 16, true)`.
    fn install(&mut self, block: usize, round_size: usize, clear: bool) -> Result<usize, Failure> {
        let storage = block + WORD * BLOCK_SHIFT;
        let space = &mut self.space;
        space
            .write_word(block, round_size)
            .ok_or_else(|| header_overflow(block, round_size))?;
        if clear {
            space
                .bytes_mut(storage, round_size)
                .ok_or_else(|| header_overflow(block, round_size))?
                .fill(0);
        }
        Ok(storage)
    }

    fn pop_free(&mut self, index: usize) -> Result<Option<usize>, Failure> {
        let Some(&block) = self.free_lists.get(&index) else {
            return Ok(None);
        };
        let next = self.read_link(block)?;
        self.set_head(index, next);
        Ok(Some(block))
    }

    fn push_free(&mut self, index: usize, block: usize) -> Result<(), Failure> {
        let head = self.free_lists.get(&index).copied().unwrap_or(0);
        self.space
            .write_word(block, head)
            .ok_or_else(|| header_overflow(block, 0))?;
        self.set_head(index, block);
        Ok(())
    }

    fn set_head(&mut self, index: usize, block: usize) {
        if block == 0 {
            self.free_lists.remove(&index);
        } else {
            self.free_lists.insert(index, block);
        }
    }

    fn read_link(&self, block: usize) -> Result<usize, Failure> {
        self.space
            .read_word(block)
            .ok_or_else(|| header_overflow(block, 0))
    }

    fn hand_out(&mut self, storage: usize) -> Result<StorageAddress, Failure> {
        let address = StorageAddress::new(storage).ok_or_else(|| header_overflow(0, 0))?;
        self.live.insert(storage);
        Ok(address)
    }

    /// Takes a live block back from the caller, returning its rounded size.
    fn claim(&mut self, storage: StorageAddress) -> Result<usize, Failure> {
        let round_size = self.round_size_of(storage)?;
        self.live.remove(&storage.get());
        Ok(round_size)
    }

    fn round_size_of(&self, storage: StorageAddress) -> Result<usize, Failure> {
        if !self.live.contains(&storage.get()) {
            return Err(foreign_storage(storage));
        }
        self.read_link(storage.get() - WORD * BLOCK_SHIFT)
    }
}

impl Default for MMgrOpt {
    /// The header's default arguments; `MMgrOpt::default()`.
    fn default() -> Self {
        Self::new(true, true, 200, 10000, 40000)
    }
}

impl MMgrRoot for MMgrOpt {
    fn allocate(&mut self, size: usize) -> Result<StorageAddress, Failure> {
        MMgrOpt::allocate(self, size)
    }
    fn reallocate(
        &mut self,
        storage: Option<StorageAddress>,
        size: usize,
    ) -> Result<StorageAddress, Failure> {
        MMgrOpt::reallocate(self, storage, size)
    }
    fn free(&mut self, storage: Option<StorageAddress>) -> Result<(), Failure> {
        MMgrOpt::free(self, storage)
    }
    fn purge(&mut self, is_destroyed: bool) -> i32 {
        MMgrOpt::purge(self, is_destroyed)
    }
    fn bytes(&self, storage: StorageAddress) -> Result<&[u8], Failure> {
        MMgrOpt::bytes(self, storage)
    }
    fn bytes_mut(&mut self, storage: StorageAddress) -> Result<&mut [u8], Failure> {
        MMgrOpt::bytes_mut(self, storage)
    }
}

fn notify(is_alloc: bool, storage: StorageAddress, round_size: usize, size: usize) {
    let callback = *CALL_BACK
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(callback) = callback {
        callback(is_alloc, storage, round_size, size);
    }
}

/// Native UB (freeing or reading a pointer this manager did not hand out) becomes an error.
fn foreign_storage(storage: StorageAddress) -> Failure {
    exceptions::program_error(&format!(
        "Standard_MMgrOpt: storage {:#x} is not a live block of this manager; expected an address it allocated and has not freed",
        storage.get()
    ))
}

/// Native UB (a header written past a too-small system block) becomes an error.
fn header_overflow(block: usize, round_size: usize) -> Failure {
    exceptions::program_error(&format!(
        "Standard_MMgrOpt: block {block:#x} cannot hold its header and {round_size} bytes; expected a system block of at least {} bytes",
        round_size.saturating_add(WORD)
    ))
}

fn invalid_system_block(address: StorageAddress, size: usize) -> Failure {
    exceptions::program_error(&format!(
        "Standard_MMgrOpt: raw region {:#x} of {size} bytes is not live in this manager; expected its exact AllocMemory result",
        address.get()
    ))
}

fn invalid_system_size(address: StorageAddress, provided: usize, expected: usize) -> Failure {
    exceptions::program_error(&format!(
        "Standard_MMgrOpt: FreeMemory size {provided} for region {:#x} does not match its AllocMemory size; expected {expected}",
        address.get()
    ))
}

/// Header size in words (`BLOCK_SHIFT`; 2 only on IRIX and Solaris).
const BLOCK_SHIFT: usize = 1;

/// `ROUNDUP_CELL`: 8-byte granularity; wraps near `usize::MAX` like the macro.
fn roundup_cell(size: usize) -> usize {
    size.wrapping_add(7) & !7
}

/// `ROUNDDOWN_CELL`; `rounddown_cell(15) == 8`.
fn rounddown_cell(size: usize) -> usize {
    size & !7
}

/// `INDEX_CELL`: free-list index of a rounded size; `index_cell(16) == 2`.
fn index_cell(round_size: usize) -> usize {
    round_size >> 3
}

/// `ROUNDUP16`; `roundup16(200) == 208`.
fn roundup16(size: usize) -> usize {
    size.wrapping_add(15) & !15
}

/// `PAGE_ALIGN`; wraps to 0 near `usize::MAX` like the macro; `page_align(1, 4096) == 4096`.
fn page_align(size: usize, page_size: usize) -> usize {
    size.wrapping_add(page_size).wrapping_sub(1) & !page_size.wrapping_sub(1)
}
