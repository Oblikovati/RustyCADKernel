//! One real contiguous allocation, with payload lifetime independent from block lifetime.
#![allow(unsafe_code)]
use crate::ncollection::{AllocatedBox, BaseAllocator, destructor_unwind::DeferredDestructorPanic};
use std::{
    any::Any,
    marker::PhantomData,
    mem::MaybeUninit,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    ptr::NonNull,
};
pub(in crate::ncollection::ubtree) struct TreePairAllocation<N> {
    pointer: Option<NonNull<N>>,
    allocator: BaseAllocator,
    live: [bool; 2],
    ownership: PhantomData<AllocatedBox<N>>,
}
impl<N> TreePairAllocation<N> {
    pub(in crate::ncollection::ubtree) fn new_with(
        allocator: &BaseAllocator,
        construct: impl FnMut(usize) -> N,
    ) -> Self {
        Self::construct_with(allocator, construct, Self::cleanup_failed)
    }
    pub(in crate::ncollection::ubtree) fn new_native_with(
        allocator: &BaseAllocator,
        construct: impl FnMut(usize) -> N,
    ) -> Self {
        Self::construct_with(allocator, construct, Self::abandon_failed)
    }
    fn construct_with(
        allocator: &BaseAllocator,
        mut construct: impl FnMut(usize) -> N,
        failed: fn(Self, Box<dyn Any + Send>) -> !,
    ) -> Self {
        let mut pair = Self::allocate(allocator);
        let result = catch_unwind(AssertUnwindSafe(|| {
            for index in 0..2 {
                pair.initialize(index, construct(index));
            }
        }));
        if let Err(error) = result {
            failed(pair, error);
        }
        pair
    }
    pub(in crate::ncollection::ubtree) fn edit_native(
        mut self,
        assign: impl FnOnce(&mut Self),
    ) -> Self {
        if let Err(error) = catch_unwind(AssertUnwindSafe(|| assign(&mut self))) {
            Self::abandon_failed(self, error);
        }
        self
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
    fn cleanup_failed(pair: Self, error: Box<dyn Any + Send>) -> ! {
        let mut pending = DeferredDestructorPanic::default();
        pending.capture(|| resume_unwind(error));
        pending.capture(|| drop(pair));
        pending.finish(Ok(()));
        unreachable!("failed pair construction must retain its original panic")
    }
    fn abandon_failed(mut pair: Self, error: Box<dyn Any + Send>) -> ! {
        // OCCT 3d097a0328 Gemmate has no owner guarding unpublished raw children.
        // Detach every projection before releasing the lease: reset may reclaim bytes afterward.
        pair.pointer.take();
        pair.live = [false; 2];
        if std::mem::size_of::<[N; 2]>() != 0 {
            // SAFETY: the unique failed allocation is intentionally leaked, with no safe
            // reference or future payload destructor access. Only the raw allocator retains it.
            unsafe { pair.allocator.relinquish_typed_to_raw() };
        }
        drop(pair);
        resume_unwind(error)
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
        let mut pending = DeferredDestructorPanic::default();
        for index in 0..2 {
            pending.capture(|| self.destroy(index));
        }
        self.deallocate();
        pending.finish(Ok(()));
    }
}

#[cfg(test)]
mod failed_contracts;
