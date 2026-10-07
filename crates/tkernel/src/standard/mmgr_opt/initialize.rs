//! The protected Initialize resets free-list heads while retaining allocated pools and live blocks.
use super::{
    AddressSpace, BLOCK_SHIFT, BTreeSet, MMgrOpt, NativeFreeListSlots, index_cell, os_page,
    roundup_cell, roundup16,
};

impl MMgrOpt {
    /// Reinitializes native free-list state; `allocator.initialize_native_free_lists()`.
    ///
    /// Recycled blocks become unreachable through free lists; pool cursors and live bytes survive.
    /// Rust retains abandoned regions and retired table backing until owner drop, and uses anonymous
    /// mappings. Native post-drop leaks and descriptor replacement are
    /// not reproduced. Failed table allocation is retained as a checked null-table state.
    /// Exclusive borrowing replaces native caller-controlled synchronization.
    pub fn initialize_native_free_lists(&mut self) {
        self.nb_pages = if self.nb_pages < 100 {
            1000
        } else {
            self.nb_pages
        };
        self.page_size = os_page::os_page_size();
        // Initialize turns mapping off when the page size is unknown (getpagesize() == 0).
        self.mmap &= self.page_size != 0;
        self.free_list_max = index_cell(roundup_cell(self.threshold.wrapping_sub(BLOCK_SHIFT)));
        self.free_lists.replace_with_zeroed(self.free_list_max);
        self.cell_size = roundup16(self.cell_size);
    }

    pub(super) fn before_initialize(clear: bool, mmap: bool) -> Self {
        Self {
            clear,
            mmap,
            cell_size: 0,
            nb_pages: 0,
            page_size: 0,
            threshold: 0,
            free_list_max: 0,
            free_lists: NativeFreeListSlots::default(),
            alloc_list: 0,
            next_addr: 0,
            end_block: 0,
            space: AddressSpace::default(),
            live: BTreeSet::new(),
        }
    }
}

#[cfg(test)]
mod tests;
