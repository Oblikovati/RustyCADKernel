// Rust translation of Standard_MMgrOpt::FreePools at pinned OCCT revision.

//! Safely releases the small-block pools while invalidating any addresses they contained.
use super::{Failure, MMgrOpt};

impl MMgrOpt {
    /// Releases every small-block pool; handles into those pools become invalid, while
    /// medium and large allocations remain live. Example: `manager.free_pools()?`.
    /// This safe Rust operation also clears dangling pool free-list links and resets the
    /// active-pool cursor; native `FreePools` is protected and used only during destruction.
    pub fn free_pools(&mut self) -> Result<(), Failure> {
        let pools = self.pool_chain();
        let pool_size = self.pool_size();
        self.remove_pool_free_blocks(&pools, pool_size)?;
        self.invalidate_pool_handles(&pools, pool_size);
        self.release_pool_regions(&pools);
        self.reset_pool_cursor();
        Ok(())
    }

    fn pool_size(&self) -> usize {
        self.page_size.wrapping_mul(self.nb_pages as usize)
    }

    fn remove_pool_free_blocks(
        &mut self,
        pools: &[usize],
        pool_size: usize,
    ) -> Result<(), Failure> {
        let indexes: Vec<usize> = self.free_lists.keys().copied().collect();
        for index in indexes {
            self.rebuild_free_list_without_pools(index, pools, pool_size)?;
        }
        Ok(())
    }

    fn rebuild_free_list_without_pools(
        &mut self,
        index: usize,
        pools: &[usize],
        pool_size: usize,
    ) -> Result<(), Failure> {
        let mut blocks = self.free_list_blocks(index)?;
        blocks.retain(|block| !is_in_pool(*block, pools, pool_size));
        self.free_lists.remove(&index);
        for block in blocks.into_iter().rev() {
            self.push_free(index, block)?;
        }
        Ok(())
    }

    fn free_list_blocks(&self, index: usize) -> Result<Vec<usize>, Failure> {
        let mut blocks = Vec::new();
        let mut block = self.free_lists.get(&index).copied().unwrap_or(0);
        while block != 0 {
            blocks.push(block);
            block = self
                .space
                .read_word(block)
                .ok_or_else(|| self.invalid_pool_link(block))?;
        }
        Ok(blocks)
    }

    fn invalid_pool_link(&self, block: usize) -> Failure {
        super::header_overflow(block, 0)
    }

    fn invalidate_pool_handles(&mut self, pools: &[usize], pool_size: usize) {
        self.live
            .retain(|storage| !is_in_pool(*storage, pools, pool_size));
    }

    fn release_pool_regions(&mut self, pools: &[usize]) {
        for pool in pools {
            self.free_memory(*pool);
        }
    }

    fn reset_pool_cursor(&mut self) {
        self.alloc_list = 0;
        self.next_addr = 0;
        self.end_block = 0;
    }
}

fn is_in_pool(address: usize, pools: &[usize], pool_size: usize) -> bool {
    pools
        .iter()
        .any(|pool| address.wrapping_sub(*pool) < pool_size)
}
