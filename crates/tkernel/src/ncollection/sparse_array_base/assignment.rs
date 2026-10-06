//! Native assignment preserves completed mutations when a copy callback unwinds.
use super::{
    SparseArrayBase, SparseArrayBlock, SparseCopyItemFunc, SparseCreateItemFunc,
    SparseDestroyItemFunc,
};
use std::ffi::c_void;
struct SparseItemCallbacks<'a> {
    create: &'a mut dyn FnMut(*mut c_void, *mut c_void),
    destroy: &'a mut dyn FnMut(*mut c_void),
    copy: &'a mut dyn FnMut(*mut c_void, *mut c_void),
}
impl SparseArrayBase {
    /// Assigns native erased objects with explicit callbacks; `unsafe { target.assign(&source, create, destroy, copy) }`.
    /// # Safety
    /// Both bases must hold the same type/stride, callbacks must match its lifecycle, no mutation
    /// aliases may exist, and destruction must not unwind. Callback queries are permitted.
    pub unsafe fn assign(
        &self,
        source: &Self,
        create: SparseCreateItemFunc,
        destroy: SparseDestroyItemFunc,
        copy: SparseCopyItemFunc,
    ) {
        unsafe {
            self.assign_with(
                source,
                |a, b| create(a, b),
                |a| destroy(a),
                |a, b| copy(a, b),
            )
        };
    }
    /// Adapts native assignment with borrowed callback contexts; `unsafe { target.assign_with(&source, create, destroy, copy) }`.
    /// # Safety
    /// Same requirements as `assign`; source and destination may be identical for the native no-op.
    pub unsafe fn assign_with(
        &self,
        source: &Self,
        mut create: impl FnMut(*mut c_void, *mut c_void),
        mut destroy: impl FnMut(*mut c_void),
        mut copy: impl FnMut(*mut c_void, *mut c_void),
    ) {
        let mut callbacks = SparseItemCallbacks {
            create: &mut create,
            destroy: &mut destroy,
            copy: &mut copy,
        };
        unsafe { self.assign_callbacks(source, &mut callbacks) };
    }
    unsafe fn assign_callbacks(&self, source: &Self, callbacks: &mut SparseItemCallbacks<'_>) {
        if std::ptr::eq(self, source) {
            return;
        }
        unsafe { self.prepare_assignment(source, callbacks) };
        for number in 0..source.block_slots() {
            unsafe { self.assign_block(source, number, callbacks) };
        }
        for number in source.block_slots()..self.block_slots() {
            unsafe { self.free_block_with(number, &mut callbacks.destroy) };
        }
    }
    unsafe fn prepare_assignment(&self, source: &Self, callbacks: &mut SparseItemCallbacks<'_>) {
        assert_eq!(
            self.item_size(),
            source.item_size(),
            "SparseArray source stride {}; expected {}",
            source.item_size(),
            self.item_size()
        );
        if self.block_size() == source.block_size() {
            return;
        }
        unsafe {
            self.clear_with(&mut callbacks.destroy);
            (*self.state.get()).block_size = source.block_size();
        }
    }
    unsafe fn assign_block(
        &self,
        source: &Self,
        number: usize,
        callbacks: &mut SparseItemCallbacks<'_>,
    ) {
        let original_address = source.block_address(number);
        if original_address.is_null() {
            unsafe { self.free_block_with(number, &mut callbacks.destroy) };
            return;
        }
        unsafe { self.ensure_block(number) };
        let original = unsafe { source.get_block(original_address) };
        let target = unsafe { self.get_block(self.block_address(number)) };
        for offset in 0..self.block_size() {
            unsafe { self.assign_slot(original, target, offset, callbacks) };
        }
    }
    unsafe fn assign_slot(
        &self,
        original: SparseArrayBlock,
        target: SparseArrayBlock,
        offset: usize,
        callbacks: &mut SparseItemCallbacks<'_>,
    ) {
        let address = unsafe { self.get_item(target, offset) };
        if unsafe { original.is_set(offset) } != 0 {
            let from = unsafe { self.get_item(original, offset) };
            unsafe { self.copy_or_construct(target, offset, address, from, callbacks) };
        } else if unsafe { target.is_set(offset) } != 0 {
            unsafe {
                target.unset(offset);
                *target.count -= 1;
                (*self.state.get()).size -= 1;
            }
            (callbacks.destroy)(address);
        }
    }
    unsafe fn copy_or_construct(
        &self,
        target: SparseArrayBlock,
        offset: usize,
        address: *mut c_void,
        from: *mut c_void,
        callbacks: &mut SparseItemCallbacks<'_>,
    ) {
        if unsafe { target.is_set(offset) } != 0 {
            (callbacks.copy)(address, from);
            return;
        }
        (callbacks.create)(address, from);
        unsafe { self.publish_slot(target, offset) };
    }
}
