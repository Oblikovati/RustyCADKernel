//! Callback boundaries preserve OCCT's observable occupancy and count publication order.
use super::{
    SparseArrayBase, SparseArrayBlock, SparseCopyItemFunc, SparseCreateItemFunc,
    SparseDestroyItemFunc,
};
use std::ffi::c_void;
impl SparseArrayBase {
    /// Constructs or assigns through native callbacks; `unsafe { base.set_value(5, source, create, copy) }`.
    /// # Safety
    /// Source/callbacks must match item stride/alignment and stored destructor. No concurrent or
    /// reentrant mutations or outstanding item references are allowed. Destructors must not unwind.
    pub unsafe fn set_value(
        &self,
        index: usize,
        source: *mut c_void,
        create: SparseCreateItemFunc,
        copy: SparseCopyItemFunc,
    ) -> *mut c_void {
        unsafe { self.set_value_with(index, source, |a, b| create(a, b), |a, b| copy(a, b)) }
    }
    /// Adapts construction/assignment callbacks with borrowed context; `unsafe { base.set_value_with(5, source, create, copy) }`.
    /// # Safety
    /// Same requirements as `set_value`; callbacks may observe valid base state but must not mutate it.
    pub unsafe fn set_value_with(
        &self,
        index: usize,
        source: *mut c_void,
        mut create: impl FnMut(*mut c_void, *mut c_void),
        mut copy: impl FnMut(*mut c_void, *mut c_void),
    ) -> *mut c_void {
        let number = index / self.block_size();
        unsafe { self.ensure_block(number) };
        let block = unsafe { self.get_block(self.block_address(number)) };
        let offset = index % self.block_size();
        let item = unsafe { self.get_item(block, offset) };
        if unsafe { block.is_set(offset) } != 0 {
            copy(item, source);
            return item;
        }
        create(item, source);
        unsafe { self.publish_slot(block, offset) };
        item
    }
    /// Removes a constructed item through its destructor; `unsafe { base.unset_value(5, destroy) }`.
    /// # Safety
    /// No references may alias mutation; destructor must match live objects and must not unwind or mutate the base.
    pub unsafe fn unset_value(&self, index: usize, destroy: SparseDestroyItemFunc) -> bool {
        unsafe { self.unset_with(index, |item| destroy(item)) }
    }
    /// Adapts removal with borrowed callback context; `unsafe { base.unset_with(5, destroy) }`.
    /// # Safety
    /// Same requirements as `unset_value`.
    pub unsafe fn unset_with(&self, index: usize, mut destroy: impl FnMut(*mut c_void)) -> bool {
        if !self.has_value(index) {
            return false;
        }
        let number = index / self.block_size();
        let block = unsafe { self.get_block(self.block_address(number)) };
        let offset = index % self.block_size();
        unsafe { block.unset(offset) };
        destroy(unsafe { self.get_item(block, offset) });
        unsafe {
            *block.count -= 1;
            (*self.state.get()).size -= 1;
        }
        if unsafe { *block.count } == 0 {
            unsafe { self.free_block_with(number, &mut destroy) };
        }
        true
    }
    /// Clears all objects and the table; `unsafe { base.clear_items(destroy) }`.
    /// # Safety
    /// Destructor must match every live object, must not unwind or mutate this base, and no live references may remain.
    pub unsafe fn clear_items(&self, destroy: SparseDestroyItemFunc) {
        unsafe { self.clear_with(|item| destroy(item)) };
    }
    /// Clears through a context-carrying callback; `unsafe { base.clear_with(destroy) }`.
    /// # Safety
    /// Same requirements as `clear_items`.
    pub unsafe fn clear_with(&self, mut destroy: impl FnMut(*mut c_void)) {
        for number in 0..self.block_slots() {
            unsafe { self.free_block_with(number, &mut destroy) };
        }
        unsafe { self.release_raw((*self.state.get()).table.cast()) };
        unsafe {
            (*self.state.get()).table = std::ptr::null_mut();
            (*self.state.get()).block_slots = 0;
            (*self.state.get()).size = 0;
        }
    }
    pub(super) unsafe fn free_block_with(
        &self,
        number: usize,
        destroy: &mut impl FnMut(*mut c_void),
    ) {
        let address = self.block_address(number);
        if address.is_null() {
            return;
        }
        let block = unsafe { self.get_block(address) };
        unsafe { self.destroy_block_objects(block, destroy) };
        unsafe {
            self.release_raw(address.cast());
            *(*self.state.get()).table.add(number) = std::ptr::null_mut();
        }
    }
    pub(super) unsafe fn publish_slot(&self, block: SparseArrayBlock, offset: usize) {
        unsafe {
            block.set(offset);
            *block.count += 1;
            (*self.state.get()).size += 1;
        }
    }
    unsafe fn destroy_block_objects(
        &self,
        block: SparseArrayBlock,
        destroy: &mut impl FnMut(*mut c_void),
    ) {
        for offset in 0..self.block_size() {
            if unsafe { block.is_set(offset) } == 0 {
                continue;
            }
            destroy(unsafe { self.get_item(block, offset) });
            unsafe {
                (*self.state.get()).size -= 1;
            }
        }
    }
}
