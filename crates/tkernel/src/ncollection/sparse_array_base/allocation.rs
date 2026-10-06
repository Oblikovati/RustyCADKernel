//! Pointer-table growth preserves object addresses and zeroes unpublished occupancy.
use super::{SparseArrayBase, SparseArrayBlock};
use allocator_api2::alloc::{Allocator, Layout};
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
        // Injected arenas must honor header alignment and retain reset leases until deallocation.
        let allocation = Allocator::allocate(&self.allocator, sparse_allocation_layout(bytes))
            .unwrap_or_else(|_| {
                panic!("SparseArray allocation {bytes} bytes; expected available heap storage")
            });
        let address = allocation.cast::<u8>().as_ptr();
        unsafe { address.write_bytes(0, bytes) };
        address
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
        unsafe { self.release_storage(old.cast(), old_count * size_of::<*mut c_void>()) };
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
    pub(super) unsafe fn release_storage(&self, address: *mut u8, bytes: usize) {
        if let Some(pointer) = NonNull::new(address) {
            unsafe {
                self.allocator
                    .deallocate(pointer, sparse_allocation_layout(bytes))
            };
        }
    }
}

fn sparse_allocation_layout(bytes: usize) -> Layout {
    let alignment = align_of::<usize>().max(align_of::<*mut c_void>());
    Layout::from_size_align(bytes, alignment).unwrap_or_else(|_| {
        panic!("SparseArray allocation {bytes} bytes, alignment {alignment}; expected a valid Rust layout")
    })
}

#[cfg(test)]
mod tests {
    use super::sparse_allocation_layout;

    #[test]
    fn sparse_layout_keeps_native_bytes_and_header_alignment() {
        let layout = sparse_allocation_layout(25);
        assert_eq!(layout.size(), 25);
        assert!(layout.align() >= align_of::<usize>());
    }

    #[test]
    #[should_panic(expected = "expected a valid Rust layout")]
    fn sparse_layout_rejects_unrepresentable_allocation_size() {
        sparse_allocation_layout(usize::MAX);
    }
}
