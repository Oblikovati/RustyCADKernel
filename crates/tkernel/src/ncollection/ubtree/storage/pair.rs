//! One real contiguous allocation, with payload lifetime independent from block lifetime.
#![allow(unsafe_code)]
use crate::ncollection::{AllocatedBox, BaseAllocator};
use std::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
pub(in crate::ncollection::ubtree) struct TreePairAllocation<N> {
    pointer: Option<NonNull<N>>,
    allocator: BaseAllocator,
    live: [bool; 2],
    ownership: PhantomData<AllocatedBox<N>>,
}
impl<N> TreePairAllocation<N> {
    /// One native `[N; 2]` block; a panicking `construct` drops the initialised slot and
    /// releases the block through `Drop` (memory safety only, D1 class B2).
    pub(in crate::ncollection::ubtree) fn new_with(
        allocator: &BaseAllocator,
        mut construct: impl FnMut(usize) -> N,
    ) -> Self {
        let mut pair = Self::allocate(allocator);
        for index in 0..2 {
            pair.initialize(index, construct(index));
        }
        pair
    }
    fn allocate(allocator: &BaseAllocator) -> Self {
        let allocation = allocator.allocate_value(MaybeUninit::<[N; 2]>::uninit());
        let (pointer, allocator) = allocation.into_raw_with_allocator();
        Self {
            pointer: NonNull::new(pointer.cast()),
            allocator,
            live: [false; 2],
            ownership: PhantomData,
        }
    }
    fn initialize(&mut self, index: usize, value: N) {
        assert!(
            !self.live[index],
            "pair slot {index}; expected uninitialized payload"
        );
        // SAFETY: this uniquely owned slot has storage but no live payload yet.
        unsafe { self.pointer(index).write(value) };
        self.live[index] = true;
    }
    pub(in crate::ncollection::ubtree) fn pointer(&self, index: usize) -> *mut N {
        assert!(index < 2, "pair slot {index}; expected 0 or 1");
        // SAFETY: both slots lie in the original typed [N; 2] allocation.
        unsafe { self.pointer.unwrap().as_ptr().add(index) }
    }
    pub(in crate::ncollection::ubtree) fn node_mut(&mut self, index: usize) -> &mut N {
        assert!(
            self.live[index],
            "pair slot {index}; expected initialized payload"
        );
        // SAFETY: exclusive owner borrow permits access to this initialized slot.
        unsafe { &mut *self.pointer(index) }
    }
    pub(super) fn destroy(&mut self, index: usize) {
        if !std::mem::replace(&mut self.live[index], false) {
            return;
        }
        // SAFETY: the arena has retired this slot; clear its live bit before user Drop.
        unsafe { std::ptr::drop_in_place(self.pointer(index)) };
    }
    pub(super) fn is_empty(&self) -> bool {
        !self.live[0] && !self.live[1]
    }
    fn deallocate(&mut self) {
        let pointer = self.pointer.take().unwrap().as_ptr().cast();
        // SAFETY: every payload is dead; MaybeUninit preserves the exact original layout.
        let allocation = unsafe {
            AllocatedBox::<MaybeUninit<[N; 2]>>::from_raw_in(pointer, self.allocator.clone())
        };
        drop(allocation);
    }
}
impl<N> Drop for TreePairAllocation<N> {
    fn drop(&mut self) {
        if self.pointer.is_none() {
            return;
        }
        // A panicking payload Drop leaks the rest of the block: never a double drop.
        for index in 0..2 {
            self.destroy(index);
        }
        self.deallocate();
    }
}
