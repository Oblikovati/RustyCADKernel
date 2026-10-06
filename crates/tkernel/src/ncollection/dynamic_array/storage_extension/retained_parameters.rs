//! Retained block parameter copying validates typed ownership before forgetting live prefixes.
use super::{DynamicArray, DynamicArrayStorageExtension, Failure};
use crate::ncollection::BaseAllocator;
use crate::ncollection::dynamic_array::AllocVec;

impl<T> DynamicArrayStorageExtension<'_, T> {
    /// Copies native parameters without freeing retained blocks or destroying their values.
    /// `unsafe { target.storage_extension().copy_parameters_retained(&source, true)? };`
    /// All blocks must fit the new stride and use compatible typed allocator layouts. Validation
    /// precedes every change. Physical capacities and addresses remain unchanged; logical size is zero.
    /// The safe `copy_parameters` convenience remains restricted to unallocated destinations.
    /// # Safety
    /// Callers assume cleanup ownership of every previously initialized value: this operation forgets
    /// all initialized lengths without invoking Drop or moving bytes. Retain raw slot addresses and
    /// their old initialized ranges to destroy or validly republish values before allocation release.
    /// No conflicting references may remain. A smaller stride can leave live values outside the new
    /// logical range. Do not destroy/re-publish a value twice or treat a destroyed slot as initialized.
    /// Detached raw allocations still require their original allocator, even after rebinding here.
    /// No ownership transfer occurs on validation error; values and metadata are unchanged.
    pub unsafe fn copy_parameters_retained(
        &mut self,
        source: &DynamicArray<T>,
        copy_allocator: bool,
    ) -> Result<(), Failure> {
        let selected = self.parameter_allocator(source, copy_allocator);
        self.validate_retained_parameters(source.increment, &selected)?;
        for block in &mut self.array.blocks {
            // Caller accepts cleanup of old live ranges. Compatibility preserves the original layout.
            unsafe { forget_and_rebind_block(block, &selected) };
        }
        self.apply_parameter_metadata(source, copy_allocator);
        Ok(())
    }
    pub(super) fn apply_parameter_metadata(
        &mut self,
        source: &DynamicArray<T>,
        copy_allocator: bool,
    ) {
        if copy_allocator {
            self.array.allocator = source.allocator.clone();
        }
        self.array.increment = source.increment;
        self.array.used = 0;
    }
    fn parameter_allocator(&self, source: &DynamicArray<T>, copy_allocator: bool) -> BaseAllocator {
        let allocator = if copy_allocator {
            &source.allocator
        } else {
            &self.array.allocator
        };
        allocator.clone().unwrap_or_default()
    }
    fn validate_retained_parameters(
        &self,
        increment: usize,
        allocator: &BaseAllocator,
    ) -> Result<(), Failure> {
        for (index, block) in self.array.blocks.iter().enumerate() {
            let fits = size_of::<T>() > 0 && increment <= block.capacity();
            if !fits || !allocator.can_free_from(block.allocator()) {
                return Err(retained_parameter_failure(
                    index, increment, block, allocator,
                ));
            }
        }
        Ok(())
    }
}
unsafe fn forget_and_rebind_block<T>(
    block: &mut AllocVec<T, BaseAllocator>,
    allocator: &BaseAllocator,
) {
    // set_len preserves object bytes at their original addresses, unlike popping/moving values.
    unsafe { block.set_len(0) };
    if block.allocator() == allocator {
        return;
    }
    let previous = std::mem::replace(block, AllocVec::new_in(allocator.clone()));
    let (pointer, length, capacity, owner) = previous.into_raw_parts_with_alloc();
    // Compatibility is identity or Common/Heap's identical typed layout; keep the original capacity.
    *block = unsafe { AllocVec::from_raw_parts_in(pointer, length, capacity, allocator.clone()) };
    drop(owner);
}
fn retained_parameter_failure<T>(
    index: usize,
    increment: usize,
    block: &AllocVec<T, BaseAllocator>,
    allocator: &BaseAllocator,
) -> Failure {
    let reason = format!(
        "block {index}, capacity {}, requested stride {increment}, item size {}, owner {:p}, selected {:p}: expected nonzero-sized T, fitting physical stride and compatible typed allocator",
        block.capacity(),
        size_of::<T>(),
        block.allocator().identity(),
        allocator.identity()
    );
    Failure::with_type("Standard_DomainError", &reason, "")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn owner_test_fixture() -> (DynamicArray<i32>, BaseAllocator, Vec<usize>) {
        let allocator = BaseAllocator::fresh_heap();
        let mut target = DynamicArray::with_allocator(2, Some(BaseAllocator::fresh_common()));
        for value in 1..=3 {
            target.append(value);
        }
        let capacities: Vec<_> = target.blocks.iter().map(AllocVec::capacity).collect();
        (target, allocator, capacities)
    }
    fn assert_block_owners(
        target: &DynamicArray<i32>,
        allocator: &BaseAllocator,
        capacities: &[usize],
    ) {
        assert!(
            target
                .blocks
                .iter()
                .all(|block| block.allocator() == allocator)
        );
        assert_eq!(
            target
                .blocks
                .iter()
                .map(AllocVec::capacity)
                .collect::<Vec<_>>(),
            capacities
        );
    }
    #[test]
    fn compatible_blocks_keep_capacity_and_adopt_actual_typed_owner() {
        let (mut target, allocator, capacities) = owner_test_fixture();
        let source = DynamicArray::with_allocator(2, Some(allocator.clone()));
        unsafe {
            target
                .storage_extension()
                .copy_parameters_retained(&source, true)
                .unwrap()
        };
        assert_block_owners(&target, &allocator, &capacities);
        unsafe { target.storage_extension().set_used_size(3).unwrap() };
    }
}
