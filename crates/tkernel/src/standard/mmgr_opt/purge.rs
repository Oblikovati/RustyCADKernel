// Rust translation of FoundationClasses/TKernel/Standard/Standard_MMgrOpt.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_MMgrOpt::Purge`: releases cached medium blocks and fully free pools.
use super::address_space::WORD;
use super::{BLOCK_SHIFT, MMgrOpt, index_cell, rounddown_cell, roundup_cell};

impl MMgrOpt {
    /// Frees every cached medium block and every pool whose blocks are all free, returning
    /// the number of blocks taken out of the free lists; `manager.purge(false)`.
    /// `is_destroyed` is ignored (the native TODO). The pool tally never reaches the pool
    /// size (the pool's link word is never a free block), so pools are never released.
    pub fn purge(&mut self, is_destroyed: bool) -> i32 {
        let _ = is_destroyed;
        let first = index_cell(roundup_cell(self.cell_size + BLOCK_SHIFT));
        let mut freed = self.release_medium_lists(first);
        freed += self.release_free_pools();
        freed
    }

    fn release_medium_lists(&mut self, first: usize) -> i32 {
        let indices: Vec<usize> = self
            .free_lists
            .range(first..=self.free_list_max.max(first))
            .map(|(index, _)| *index)
            .collect();
        let mut freed = 0;
        for index in indices {
            let mut block = self.free_lists.remove(&index).unwrap_or(0);
            while block != 0 {
                let next = self.space.read_word(block).unwrap_or(0);
                self.free_memory(block);
                freed += 1;
                block = next;
            }
        }
        freed
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
        for (&index, &head) in self.free_lists.range(..=cells) {
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
        let inside = |block: usize| {
            free.iter()
                .any(|pool| (*pool..*pool + pool_size).contains(&block))
        };
        let mut removed = 0;
        let mut kept: Vec<(usize, usize)> = Vec::new();
        for (index, block) in self.small_free_blocks() {
            if inside(block) {
                removed += 1;
            } else {
                kept.push((index, block));
            }
        }
        self.free_lists
            .retain(|index, _| *index > index_cell(self.cell_size));
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
