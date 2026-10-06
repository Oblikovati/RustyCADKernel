//! Native layout has no alignment padding between count, objects and bitmap.
use std::ffi::c_void;

/// Non-owning native block projection; `unsafe { SparseArrayBlock::new(address, 8, 4) }`.
#[derive(Clone, Copy)]
pub struct SparseArrayBlock {
    /// Address of the native block occupancy counter.
    pub count: *mut usize,
    /// Address of the first raw object slot, immediately after the counter.
    pub array: *mut c_void,
    /// Address of the occupancy bitmap after the object span.
    pub bits: *mut u8,
}

impl SparseArrayBlock {
    /// Returns the native cell width; `SparseArrayBlock::bits_per_cell() == 8`.
    pub const fn bits_per_cell() -> usize {
        u8::BITS as usize
    }
    /// Computes header/items/bitmap storage bytes; `SparseArrayBlock::size(9, 12) == 118` on 64-bit.
    pub fn size(items: usize, stride: usize) -> usize {
        size_of::<usize>() + items.div_ceil(Self::bits_per_cell()) + items * stride
    }
    /// Projects a block without accessing its unconstructed items; `unsafe { SparseArrayBlock::new(address, 8, 4) }`.
    /// # Safety
    /// Address must own `size(items, stride)` bytes aligned for usize and all item addresses.
    pub unsafe fn new(address: *mut c_void, items: usize, stride: usize) -> Self {
        let array = unsafe { Self::to_array(address) };
        Self {
            count: address.cast(),
            array,
            bits: unsafe { array.cast::<u8>().add(items * stride) },
        }
    }
    /// Returns the native object-array start; `unsafe { SparseArrayBlock::to_array(address) }`.
    /// # Safety
    /// Address must identify a live block including its usize header.
    pub unsafe fn to_array(address: *mut c_void) -> *mut c_void {
        unsafe { address.cast::<u8>().add(size_of::<usize>()).cast() }
    }
    /// Sets an occupancy bit and returns one if newly set; `unsafe { block.set(3) }`.
    /// # Safety
    /// Index must be in the block and its bitmap must permit exclusive mutation.
    /// Restore object-lifetime/count/occupancy agreement before the owner next uses or drops the block.
    pub unsafe fn set(self, index: usize) -> u8 {
        let cell = unsafe { self.bits.add(index / 8) };
        let mask = 1u8 << (index % 8);
        let old = unsafe { *cell & mask };
        unsafe { *cell |= mask };
        u8::from(old == 0)
    }
    /// Returns the bit mask of an occupied slot; `unsafe { block.is_set(3) }`.
    /// # Safety
    /// Index must be within a readable initialized bitmap.
    pub unsafe fn is_set(self, index: usize) -> u8 {
        unsafe { *self.bits.add(index / 8) & (1u8 << (index % 8)) }
    }
    /// Clears and returns the old occupancy mask; `unsafe { block.unset(3) }`.
    /// # Safety
    /// Index must be in the block and its bitmap must permit exclusive mutation.
    /// Restore object-lifetime/count/occupancy agreement before the owner next uses or drops the block.
    pub unsafe fn unset(self, index: usize) -> u8 {
        let cell = unsafe { self.bits.add(index / 8) };
        let mask = 1u8 << (index % 8);
        let old = unsafe { *cell & mask };
        unsafe { *cell &= !mask };
        old
    }
}
