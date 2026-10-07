//! A conservative gate avoids scanning canonical prefixes without mirroring ownership counts.
use super::DynamicArray;

impl<T> DynamicArray<T> {
    pub(super) fn allocated_slot_with(
        &mut self,
        index: usize,
        construct: impl FnMut() -> T,
    ) -> &mut T {
        if self.increment == 0 {
            self.require_retained_first_index(index);
            return self
                .change_value(index)
                .expect("zero-stride index 0: expected an initialized first slot");
        }
        self.allocated_positive_slot_with(index, construct)
    }
    fn allocated_positive_slot_with(
        &mut self,
        index: usize,
        construct: impl FnMut() -> T,
    ) -> &mut T {
        let block = index / self.increment;
        let offset = index % self.increment;
        assert!(
            block < self.blocks.len(),
            "index {index}: expected allocated capacity {}",
            self.blocks.len() * self.increment
        );
        if self.blocks[block].len() <= offset {
            self.needs_tail_scan = true;
            self.blocks[block].resize_with(offset + 1, construct);
        }
        &mut self.blocks[block][offset]
    }

    pub(super) fn has_initialized_tail(&self) -> bool {
        self.initialized_tail_in(self.blocks.iter().map(|block| block.len()))
    }
    fn initialized_tail_in(&self, lengths: impl Iterator<Item = usize>) -> bool {
        if !self.needs_tail_scan {
            return false;
        }
        // Keep the original per-block predicate for tails and holes after caught Rust panics.
        lengths.enumerate().any(|(block_index, length)| {
            let block_start = block_index * self.increment;
            let used_in_block = self.used.saturating_sub(block_start).min(self.increment);
            length > used_in_block
        })
    }
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests;
