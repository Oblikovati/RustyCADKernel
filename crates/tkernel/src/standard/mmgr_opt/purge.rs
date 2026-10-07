// Rust translation of FoundationClasses/TKernel/Standard/Standard_MMgrOpt.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_MMgrOpt::Purge`: releases cached medium blocks and fully free pools.
use super::address_space::WORD;
use super::{BLOCK_SHIFT, MMgrOpt, index_cell, rounddown_cell, roundup_cell};

impl MMgrOpt {
    /// Frees every cached medium block and every pool whose blocks are all free, returning
    /// the number of blocks taken out of the free lists; `manager.purge(false)`.
    /// `is_destroyed` is ignored (the native TODO). The pool tally never reaches the pool
    /// size (the pool's link word is never a free block), so pools are never released.
    /// Reading a missing head table panics instead of performing native null-pointer access.
    pub fn purge(&mut self, is_destroyed: bool) -> i32 {
        let _ = is_destroyed;
        let first = index_cell(roundup_cell(self.cell_size + BLOCK_SHIFT));
        let mut freed = self.release_medium_lists(first);
        freed += self.release_free_pools();
        freed
    }

    fn release_medium_lists(&mut self, first: usize) -> i32 {
        // Production has no release observer; tests exercise this same canonical path.
        self.release_medium_lists_observed(first, |_, _, _| {})
    }

    pub(super) fn release_medium_lists_observed(
        &mut self,
        first: usize,
        mut observe: impl FnMut(usize, usize, usize),
    ) -> i32 {
        let mut freed = 0;
        for index in first..=self.free_list_max {
            freed += self.release_size_class(index, &mut observe);
        }
        freed
    }

    fn release_size_class(
        &mut self,
        index: usize,
        observe: &mut impl FnMut(usize, usize, usize),
    ) -> i32 {
        let mut block = self
            .free_lists
            .head(index)
            .unwrap_or_else(|failure| panic!("{failure}"));
        let mut freed = 0;
        while block != 0 {
            block = self.release_node_observed(index, block, observe);
            freed += 1;
        }
        // Native Purge clears the head only after every node has reached system free.
        self.set_head(index, 0)
            .unwrap_or_else(|failure| panic!("{failure}"));
        freed
    }

    fn release_node_observed(
        &mut self,
        index: usize,
        block: usize,
        observe: &mut impl FnMut(usize, usize, usize),
    ) -> usize {
        let next = self.space.read_word(block).unwrap_or(0);
        observe(index, block, self.free_lists.head(index).unwrap());
        self.free_memory(block);
        next
    }

    /// Releases pools whose free small blocks add up to the whole pool. Native code scans
    /// pools in windows of 512; the set of released pools and the count are the same.
    fn release_free_pools(&mut self) -> i32 {
        let pool_size = rounddown_cell(self.page_size.wrapping_mul(self.nb_pages as usize));
        let pools = self.pool_chain();
        let free: Vec<usize> = pools
            .iter()
            .copied()
            .filter(|pool| roundup_cell(self.free_tally(*pool, pool_size)) == pool_size)
            .collect();
        if free.is_empty() {
            return 0;
        }
        let freed = self.unlink_blocks_in(&free, pool_size);
        self.relink_pools(&pools, &free);
        free.iter().for_each(|pool| self.free_memory(*pool));
        freed
    }

    /// Pools from the newest (`myAllocList`) to the oldest, following each pool's link word.
    pub(super) fn pool_chain(&self) -> Vec<usize> {
        let mut pools = Vec::new();
        let mut pool = self.alloc_list;
        while pool != 0 {
            pools.push(pool);
            pool = self.space.read_word(pool).unwrap_or(0);
        }
        pools
    }

    /// Bytes of the small free blocks (header included) lying inside `pool`.
    fn free_tally(&self, pool: usize, pool_size: usize) -> usize {
        let mut tally = 0usize;
        for (index, block) in self.small_free_blocks() {
            if (pool..pool + pool_size).contains(&block) {
                tally += BLOCK_SHIFT * WORD + roundup_cell(1) * index;
            }
        }
        tally
    }

    fn small_free_blocks(&self) -> Vec<(usize, usize)> {
        let cells = index_cell(self.cell_size);
        let mut blocks = Vec::new();
        for (index, head) in self
            .free_lists
            .heads()
            .unwrap_or_else(|failure| panic!("{failure}"))
            .take(cells.saturating_add(1))
        {
            let mut block = head;
            while block != 0 {
                blocks.push((index, block));
                block = self.space.read_word(block).unwrap_or(0);
            }
        }
        blocks
    }

    /// Removes the free blocks of the released pools from their lists, counting them.
    fn unlink_blocks_in(&mut self, free: &[usize], pool_size: usize) -> i32 {
        let mut removed = 0;
        let mut kept: Vec<(usize, usize)> = Vec::new();
        for (index, block) in self.small_free_blocks() {
            if super::free_pools::is_in_pool(block, free, pool_size) {
                removed += 1;
            } else {
                kept.push((index, block));
            }
        }
        self.free_lists
            .clear_through(index_cell(self.cell_size))
            .unwrap_or_else(|failure| panic!("{failure}"));
        for (index, block) in kept.into_iter().rev() {
            // Rebuilding newest-first from the reversed survivors keeps each list's order.
            let _ = self.push_free(index, block);
        }
        removed
    }

    fn relink_pools(&mut self, pools: &[usize], free: &[usize]) {
        let kept: Vec<usize> = pools
            .iter()
            .copied()
            .filter(|pool| !free.contains(pool))
            .collect();
        for pair in kept.windows(2) {
            let _ = self.space.write_word(pair[0], pair[1]);
        }
        if let Some(last) = kept.last() {
            let _ = self.space.write_word(*last, 0);
        }
        self.alloc_list = kept.first().copied().unwrap_or(0);
    }
}
