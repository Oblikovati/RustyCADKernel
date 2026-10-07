//! Preserve typed allocation layout while allowing registered slots whose T lifetime has ended.
use super::{AllocVec, BaseAllocator};
use std::mem::{ManuallyDrop, MaybeUninit};
pub(super) struct ClearRawBlock<T> {
    storage: ManuallyDrop<AllocVec<MaybeUninit<T>, BaseAllocator>>,
}
impl<T> ClearRawBlock<T> {
    pub(super) fn take_from(block: &mut AllocVec<T, BaseAllocator>) -> Self {
        let replacement = AllocVec::new_in(block.allocator().clone());
        let original = std::mem::replace(block, replacement);
        let (pointer, length, capacity, allocator) = original.into_raw_parts_with_alloc();
        // MaybeUninit<T> has T's layout; preserve the original allocation and allocator exactly.
        let storage =
            unsafe { AllocVec::from_raw_parts_in(pointer.cast(), length, capacity, allocator) };
        Self {
            storage: ManuallyDrop::new(storage),
        }
    }
    pub(super) fn pointer(&self, offset: usize) -> *const T {
        self.storage.as_ptr().wrapping_add(offset).cast()
    }
    pub(super) fn mutable_pointer(&mut self, offset: usize) -> *mut T {
        self.storage.as_mut_ptr().wrapping_add(offset).cast()
    }
    pub(super) fn registered_length(&self) -> usize {
        self.storage.len()
    }
    pub(super) fn into_initialized(self, length: usize) -> AllocVec<T, BaseAllocator> {
        let original = ManuallyDrop::into_inner(self.storage);
        let (pointer, _, capacity, allocator) = original.into_raw_parts_with_alloc();
        // Only the state engine calls this after all requested T slots are individually live.
        unsafe { AllocVec::from_raw_parts_in(pointer.cast(), length, capacity, allocator) }
    }
    pub(super) fn release_empty(self) {
        // The engine has ended every lifetime, or native zero-stride Clear intentionally omits it.
        drop(ManuallyDrop::into_inner(self.storage));
    }
    #[cfg(test)]
    pub(super) fn raw_ownership_for_test(&mut self) -> (*mut T, usize, usize, BaseAllocator) {
        (
            self.storage.as_mut_ptr().cast(),
            self.storage.len(),
            self.storage.capacity(),
            self.storage.allocator().clone(),
        )
    }
}
