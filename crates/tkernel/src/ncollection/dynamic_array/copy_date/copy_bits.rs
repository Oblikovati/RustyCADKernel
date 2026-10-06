//! The selected trivial policy copies complete object representations without calling Clone.
use super::{AllocVec, BaseAllocator};

pub(super) fn copy_bitwise_values<T: Copy>(values: &[T], block: &mut AllocVec<T, BaseAllocator>) {
    assert!(
        block.is_empty(),
        "block length {}: expected empty copy destination",
        block.len()
    );
    assert!(
        block.capacity() >= values.len(),
        "block capacity {}: expected at least {} source values",
        block.capacity(),
        values.len()
    );
    // The block is newly allocated and cannot overlap the source. Copy permits duplicate ownership;
    // copy_nonoverlapping preserves initialized bytes and padding like native trivial memcpy.
    unsafe {
        std::ptr::copy_nonoverlapping(values.as_ptr(), block.as_mut_ptr(), values.len());
        block.set_len(values.len());
    }
}
