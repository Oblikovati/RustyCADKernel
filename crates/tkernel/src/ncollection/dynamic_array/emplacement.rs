use super::super::{ArrayError, BaseAllocator};
use super::{DynamicArray, checked_index};
use allocator_api2::vec::Vec as AllocVec;
use std::fmt::Debug;

struct EmplaceValueStage<'a, T> {
    array: &'a mut DynamicArray<T>,
    old_used: usize,
    index: usize,
    next_index: usize,
    committed: bool,
}
impl<T: Default> EmplaceValueStage<'_, T> {
    fn construct(mut self, construct: impl FnOnce() -> T) -> usize {
        self.append_gap_defaults();
        self.append_requested_value(construct());
        self.array.used = self.index + 1;
        self.committed = true;
        self.index
    }
    fn append_gap_defaults(&mut self) {
        while self.next_index < self.index {
            let index = self.next_index;
            self.array.write_staged_value(index, T::default());
            self.next_index += 1;
        }
    }
    fn append_requested_value(&mut self, value: T) {
        self.array.write_staged_value(self.index, value);
    }
}
impl<T> Drop for EmplaceValueStage<'_, T> {
    fn drop(&mut self) {
        if !self.committed {
            self.array.forget_staged_storage(self.old_used);
        }
    }
}
struct PendingEmplaceValues<T> {
    values: Vec<T>,
    committed: bool,
}
impl<T> PendingEmplaceValues<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            committed: false,
        }
    }
    fn push(&mut self, value: T) {
        self.values.push(value);
    }
    fn release(mut self) -> Vec<T> {
        self.committed = true;
        std::mem::take(&mut self.values)
    }
}
impl<T> Drop for PendingEmplaceValues<T> {
    fn drop(&mut self) {
        if !self.committed {
            // Native leaves constructed gaps outside UsedSize after a later throw.
            forget_vec_values(&mut self.values);
        }
    }
}
impl<T> DynamicArray<T> {
    fn write_staged_value(&mut self, index: usize, value: T) {
        let block = &mut self.blocks[index / self.increment];
        let offset = index % self.increment;
        if offset < block.len() {
            block[offset] = value;
        } else {
            block.push(value);
        }
    }
    fn forget_staged_storage(&mut self, length: usize) {
        for (block_index, block) in self.blocks.iter_mut().enumerate().rev() {
            let block_start = block_index * self.increment;
            let keep = length.saturating_sub(block_start).min(self.increment);
            forget_block_values_after(block, keep);
        }
    }
    fn has_initialized_tail(&self) -> bool {
        self.blocks.iter().enumerate().any(|(block_index, block)| {
            let block_start = block_index * self.increment;
            let used_in_block = self.used.saturating_sub(block_start).min(self.increment);
            block.len() > used_in_block
        })
    }
    fn emplace_direct(&mut self, index: usize, construct: impl FnOnce() -> T) -> usize
    where
        T: Default,
    {
        let old_used = self.used;
        EmplaceValueStage {
            array: self,
            old_used,
            index,
            next_index: old_used,
            committed: false,
        }
        .construct(construct)
    }
    fn emplace_with_initialized_tail(
        &mut self,
        index: usize,
        construct: impl FnOnce() -> T,
    ) -> usize
    where
        T: Default,
    {
        // Stage separately so a failed constructor preserves safe Rust IndexMut slots.
        let mut pending = PendingEmplaceValues::with_capacity(index - self.used + 1);
        self.stage_gap_defaults(self.used, index, &mut pending);
        pending.push(construct());
        self.commit_staged_values(self.used, pending.release());
        self.used = index + 1;
        index
    }
    fn stage_gap_defaults(&self, first: usize, end: usize, pending: &mut PendingEmplaceValues<T>)
    where
        T: Default,
    {
        for _ in first..end {
            pending.push(T::default());
        }
    }
    fn commit_staged_values(&mut self, first: usize, values: Vec<T>) {
        for (offset, value) in values.into_iter().enumerate() {
            self.write_staged_value(first + offset, value);
        }
    }
    fn emplace_existing(
        &mut self,
        index: usize,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, ArrayError> {
        let replacement = construct();
        let slot = self.change_value(index)?;
        *slot = replacement;
        Ok(slot)
    }
    fn emplace_new(&mut self, index: usize, construct: impl FnOnce() -> T) -> usize
    where
        T: Default,
    {
        if self.has_initialized_tail() {
            return self.emplace_with_initialized_tail(index, construct);
        }
        self.emplace_direct(index, construct)
    }
    /// Constructs gaps before the requested value; `array.emplace_value(5, || 42)?`.
    pub fn emplace_value<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, ArrayError>
    where
        T: Default,
    {
        let index = checked_index(index)?;
        self.ensure_storage_for_index(index);
        self.emplace_checked(index, construct)
    }
    fn emplace_checked(
        &mut self,
        index: usize,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, ArrayError>
    where
        T: Default,
    {
        if index < self.used {
            return self.emplace_existing(index, construct);
        }
        let final_index = self.emplace_new(index, construct);
        self.change_value(final_index)
    }
}
fn forget_block_values_after<T>(block: &mut AllocVec<T, BaseAllocator>, keep: usize) {
    while block.len() > keep {
        let Some(value) = block.pop() else {
            return;
        };
        std::mem::forget(value);
    }
}
fn forget_vec_values<T>(values: &mut Vec<T>) {
    while let Some(value) = values.pop() {
        std::mem::forget(value);
    }
}
