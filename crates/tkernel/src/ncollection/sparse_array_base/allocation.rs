//! Pointer-table growth preserves object addresses and zeroes unpublished occupancy.
use super::{SparseArrayBase, SparseArrayBlock};
use std::{ffi::c_void, ptr::NonNull};
impl SparseArrayBase {
    // Rust Clone preserves retained table capacity even when no live item reaches its last block.
    pub(crate) unsafe fn reserve_clone_table(&self, count: usize) {
        if count > self.block_slots() {
            unsafe { self.grow_table(count - 1) };
        }
    }

    pub(super) unsafe fn ensure_block(&self, number: usize) {
        if number >= self.block_slots() {
            unsafe { self.grow_table(number) };
        }
        if !self.block_address(number).is_null() {
            return;
        }
        let bytes = SparseArrayBlock::size(self.block_size(), self.item_size());
        let address = self.allocate_zeroed(bytes);
        unsafe { *(*self.state.get()).table.add(number) = address.cast() };
    }
    fn allocate_zeroed(&self, bytes: usize) -> *mut u8 {
        let address = self
            .allocator
            .allocate_optimal_raw(bytes)
            .unwrap_or_else(|| {
                panic!("SparseArray allocation {bytes} bytes; expected available heap storage")
            });
        unsafe { address.as_ptr().write_bytes(0, bytes) };
        address.as_ptr()
    }
    unsafe fn grow_table(&self, number: usize) {
        let old_count = self.block_slots();
        let count = self.grown_table_count(number);
        let replacement = self
            .allocate_zeroed(count * size_of::<*mut c_void>())
            .cast::<*mut c_void>();
        let old = unsafe { (*self.state.get()).table };
        if old_count != 0 {
            unsafe { std::ptr::copy_nonoverlapping(old, replacement, old_count) };
        }
        unsafe { self.release_raw(old.cast()) };
        unsafe {
            (*self.state.get()).table = replacement;
            (*self.state.get()).block_slots = count;
        }
    }
    fn grown_table_count(&self, number: usize) -> usize {
        let mut count = if self.block_slots() == 0 {
            self.block_size()
        } else {
            self.block_slots()
        };
        assert!(
            count != 0,
            "SparseArray block size {count}; expected positive size for indexed access"
        );
        while number >= count {
            count = count
                .checked_mul(2)
                .expect("SparseArray table growth exceeds usize span");
        }
        count
    }
    pub(super) unsafe fn release_raw(&self, address: *mut u8) {
        if let Some(pointer) = NonNull::new(address) {
            unsafe { self.allocator.free_raw(pointer) }
                .expect("SparseArray owned allocation; expected matching resource");
        }
    }
}
