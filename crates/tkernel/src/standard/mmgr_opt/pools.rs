// Rust translation of FoundationClasses/TKernel/Standard/Standard_MMgrOpt.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Small-block pools, medium `malloc` blocks and large mapped blocks (`AllocMemory`).
use super::address_space::WORD;
use super::{BLOCK_SHIFT, MMgrOpt, header_overflow, index_cell, page_align, rounddown_cell};
use crate::standard::{Failure, exceptions::out_of_memory};

/// glibc `malloc` alignment on the supported 64-bit targets (two pointers).
const MALLOC_ALIGNMENT: usize = 2 * WORD;
/// errno values `mmap` reports (identical on Linux, macOS and the BSDs).
const ENOMEM: i32 = 12;
const EINVAL: i32 = 22;

impl MMgrOpt {
    /// Carves a small block from the active pool, opening a new pool when it is full;
    /// `manager.allocate_in_pool(16)`. The unused tail of the old pool joins a free list.
    pub(super) fn allocate_in_pool(&mut self, round_size: usize) -> Result<usize, Failure> {
        let needed = self
            .next_addr
            .wrapping_add(WORD * (BLOCK_SHIFT + round_size / WORD));
        let block = if needed > self.end_block {
            self.open_pool()?
        } else {
            self.next_addr
        };
        let storage = self.install(block, round_size, false)?;
        self.next_addr = storage + round_size;
        Ok(storage)
    }

    fn open_pool(&mut self) -> Result<usize, Failure> {
        let requested = self.page_size.wrapping_mul(self.nb_pages as usize);
        let (pool, size) = self.alloc_memory(requested)?;
        if self.end_block > self.next_addr {
            self.recycle_pool_tail()?;
        }
        self.end_block = pool + rounddown_cell(size);
        self.space
            .write_word(pool, self.alloc_list)
            .ok_or_else(|| header_overflow(pool, 0))?;
        self.alloc_list = pool;
        Ok(pool + WORD * BLOCK_SHIFT)
    }

    /// Pushes the rest of the exhausted pool (minus a header) as one free block.
    fn recycle_pool_tail(&mut self) -> Result<(), Failure> {
        let tail_size = self.end_block - (self.next_addr + WORD * BLOCK_SHIFT);
        let index = index_cell(rounddown_cell(tail_size));
        if index > 0 && index <= self.free_list_max {
            self.push_free(index, self.next_addr)?;
        }
        Ok(())
    }

    /// Allocates a recyclable block above the cell size with `calloc`/`malloc`, purging
    /// once on failure; `manager.allocate_medium(512)`.
    pub(super) fn allocate_medium(&mut self, round_size: usize) -> Result<usize, Failure> {
        let words = round_size / WORD + BLOCK_SHIFT;
        let mut block = self.space.reserve(words * WORD, MALLOC_ALIGNMENT);
        // The retry after a successful purge always uses calloc, even without clearing.
        if block.is_none() && self.purge(false) != 0 {
            block = self.space.reserve(words * WORD, MALLOC_ALIGNMENT);
        }
        let block =
            block.ok_or_else(|| out_of_memory("Standard_MMgrOpt::Allocate(): malloc failed"))?;
        self.install(block, round_size, false)
    }

    /// Allocates a block above the threshold through `AllocMemory`; `manager.allocate_large(1 << 20)`.
    /// The header word is included in the size, which wraps to 0 for sizes near `usize::MAX`.
    pub(super) fn allocate_large(&mut self, round_size: usize) -> Result<usize, Failure> {
        let (block, _) = self.alloc_memory(round_size.wrapping_add(WORD))?;
        self.install(block, round_size, false)
    }

    /// `AllocMemory`: maps page-aligned memory or mallocs it, purging and retrying while a
    /// purge releases something; returns the block and the size actually obtained.
    pub(super) fn alloc_memory(&mut self, size: usize) -> Result<(usize, usize), Failure> {
        loop {
            let attempt = if self.mmap {
                self.map_memory(size)
            } else {
                self.space
                    .reserve(size, MALLOC_ALIGNMENT)
                    .map(|block| (block, size))
                    .ok_or_else(|| out_of_memory("Standard_MMgrOpt::Allocate(): malloc failed"))
            };
            match attempt {
                Ok(obtained) => return Ok(obtained),
                Err(failure) if self.purge(false) == 0 => return Err(failure),
                Err(_) => continue,
            }
        }
    }

    /// Anonymous private mapping of `PAGE_ALIGN(size)` zeroed bytes; `mmap` refuses a zero
    /// length with EINVAL and an unsatisfiable one with ENOMEM, reported via `strerror`.
    fn map_memory(&mut self, size: usize) -> Result<(usize, usize), Failure> {
        let aligned = page_align(size, self.page_size);
        if aligned == 0 {
            return Err(out_of_memory(&os_error_text(EINVAL)));
        }
        self.space
            .reserve_mapped(aligned, self.page_size)
            .map(|block| (block, aligned))
            .ok_or_else(|| out_of_memory(&os_error_text(ENOMEM)))
    }

    /// `FreeMemory`: unmaps or frees a system block. Native `munmap` passes
    /// `PAGE_ALIGN(round_size)`, one page short when the header crossed a page boundary;
    /// that leaked page is not observable and the port releases the whole region.
    pub(super) fn free_memory(&mut self, block: usize) {
        let released = self.space.release(block);
        debug_assert!(
            released,
            "block {block:#x} is not a system block of this manager"
        );
    }
}

/// `strerror(code)` as the C runtime words it; `os_error_text(12) == "Cannot allocate memory"`.
fn os_error_text(code: i32) -> String {
    let text = std::io::Error::from_raw_os_error(code).to_string();
    let suffix = format!(" (os error {code})");
    text.strip_suffix(&suffix).unwrap_or(&text).to_owned()
}
