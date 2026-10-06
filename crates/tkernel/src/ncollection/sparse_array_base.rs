//! Erased callback storage matching OCCT 3d097a's sparse block layout and publication order.
#![allow(unsafe_code)]
use super::BaseAllocator;
use std::{cell::UnsafeCell, ffi::c_void, rc::Rc};
mod allocation;
mod assignment;
mod block;
mod cursor;
mod editing;
pub use block::SparseArrayBlock;
pub use cursor::SparseArrayBaseIterator;

/// Native copy-construction callback; `unsafe fn create(dst: *mut c_void, src: *mut c_void)`.
pub type SparseCreateItemFunc = unsafe fn(*mut c_void, *mut c_void);
/// Native in-place assignment callback, including self-assignment; `unsafe fn copy(dst: *mut c_void, src: *mut c_void)`.
pub type SparseCopyItemFunc = unsafe fn(*mut c_void, *mut c_void);
/// Native object destructor callback; `unsafe fn destroy(item: *mut c_void)`.
pub type SparseDestroyItemFunc = unsafe fn(*mut c_void);

/// Protected native fields, exposed only through an unsafe raw pointer; `unsafe { (*base.native_state()).size }`.
pub struct SparseArrayBaseState {
    /// Native byte stride between adjacent objects.
    pub item_size: usize,
    /// Number of object slots in each raw block.
    pub block_size: usize,
    /// Length of the allocated block-pointer table.
    pub block_slots: usize,
    /// Number of published objects across all blocks.
    pub size: usize,
    /// Owned pointer table; null before its first allocation.
    pub table: *mut *mut c_void,
    /// Callback retained for destruction of published objects.
    pub destroy: SparseDestroyItemFunc,
}

/// Owns the native table and count/items/bitmap blocks; `SparseArrayBase::new(4, 8, destroy)`.
/// Mutations are unsafe because callbacks establish object lifetimes inside erased storage.
pub struct SparseArrayBase {
    // Drop receives &mut Self; shared allocation keeps callback observer pointers valid during Drop.
    state: Rc<UnsafeCell<SparseArrayBaseState>>,
    allocator: BaseAllocator,
}

impl SparseArrayBase {
    /// Constructs empty storage with the common resource; `SparseArrayBase::new(4, 8, destroy)`.
    pub fn new(item_size: usize, block_size: usize, destroy: SparseDestroyItemFunc) -> Self {
        Self::with_allocator(item_size, block_size, destroy, BaseAllocator::common())
    }

    /// Injects an allocation resource (Rust extension); `SparseArrayBase::with_allocator(4, 8, destroy, resource)`.
    pub fn with_allocator(
        item_size: usize,
        block_size: usize,
        destroy: SparseDestroyItemFunc,
        allocator: BaseAllocator,
    ) -> Self {
        Self {
            state: Rc::new(UnsafeCell::new(SparseArrayBaseState {
                item_size,
                block_size,
                block_slots: 0,
                size: 0,
                table: std::ptr::null_mut(),
                destroy,
            })),
            allocator,
        }
    }

    /// Returns published item count; `base.size()`.
    pub fn size(&self) -> usize {
        unsafe { (*self.state.get()).size }
    }
    /// Returns pointer-table index span; `base.capacity()`.
    pub fn capacity(&self) -> usize {
        self.block_slots() * self.block_size()
    }
    /// Returns native item stride; `base.item_size()`.
    pub fn item_size(&self) -> usize {
        unsafe { (*self.state.get()).item_size }
    }
    /// Returns items per block; `base.block_size()`.
    pub fn block_size(&self) -> usize {
        unsafe { (*self.state.get()).block_size }
    }
    /// Returns allocated pointer slots; `base.block_slots()`.
    pub fn block_slots(&self) -> usize {
        unsafe { (*self.state.get()).block_slots }
    }
    /// Provides the protected native state for derived containers; `unsafe { base.native_state() }`.
    /// # Safety
    /// Writes must preserve allocation ownership, callback validity, counts and all block invariants.
    pub unsafe fn native_state(&self) -> *mut SparseArrayBaseState {
        self.state.get()
    }

    /// Returns a block pointer or null beyond the table; `base.block_address(0)`.
    pub fn block_address(&self, number: usize) -> *mut c_void {
        if number >= self.block_slots() {
            return std::ptr::null_mut();
        }
        unsafe { *(*self.state.get()).table.add(number) }
    }

    /// Checks a constructed slot; `base.has_value(5)`.
    pub fn has_value(&self, index: usize) -> bool {
        let address = self.block_address(index / self.block_size());
        if address.is_null() {
            return false;
        }
        unsafe { self.get_block(address).is_set(index % self.block_size()) != 0 }
    }

    /// Projects a raw native block; `unsafe { base.get_block(base.block_address(0)) }`.
    /// # Safety
    /// Address must identify a live block with this base's item stride and block size.
    pub unsafe fn get_block(&self, address: *mut c_void) -> SparseArrayBlock {
        unsafe { SparseArrayBlock::new(address, self.block_size(), self.item_size()) }
    }

    /// Computes the native item address; `unsafe { base.get_item(block, 2) }`.
    /// # Safety
    /// Block must be live and index within its allocated item span.
    pub unsafe fn get_item(&self, block: SparseArrayBlock, index: usize) -> *mut c_void {
        unsafe {
            block
                .array
                .cast::<u8>()
                .add(index * self.item_size())
                .cast()
        }
    }

    /// Returns an occupied raw object address; `base.get_value(5)?`.
    pub fn get_value(&self, index: usize) -> Result<*mut c_void, crate::standard::Failure> {
        if !self.has_value(index) {
            return Err(crate::standard::exceptions::out_of_range(&format!(
                "SparseArray index {index}; expected an occupied slot"
            )));
        }
        unsafe {
            Ok(self.get_item(
                self.get_block(self.block_address(index / self.block_size())),
                index % self.block_size(),
            ))
        }
    }

    /// Exchanges geometry, ownership and destruction callbacks; `unsafe { a.exchange(&mut b) }`.
    /// # Safety
    /// No live item references or iterators may be invalidated; each state must remain internally valid.
    pub unsafe fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
}

impl Drop for SparseArrayBase {
    fn drop(&mut self) {
        let destroy = unsafe { (*self.state.get()).destroy };
        // SAFETY: insertion's contract preserves the stored destructor's validity for all live items.
        unsafe { self.clear_items(destroy) };
    }
}
