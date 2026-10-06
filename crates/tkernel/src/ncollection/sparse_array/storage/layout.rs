//! Native inline layout covers ordinary T; over-aligned Rust values use an indirect slot.
use std::ffi::c_void;
fn sparse_indirect<T>() -> bool {
    align_of::<T>() > align_of::<usize>()
}
pub(in super::super) fn sparse_stride<T>() -> usize {
    if sparse_indirect::<T>() {
        size_of::<*mut T>()
    } else {
        // C++ has no zero-size objects; a Rust ZST still requires an aligned address per slot.
        size_of::<T>().max(align_of::<T>())
    }
}
pub(in super::super) unsafe fn sparse_project<T>(slot: *mut c_void) -> *mut T {
    if sparse_indirect::<T>() {
        unsafe { slot.cast::<*mut T>().read() }
    } else {
        slot.cast()
    }
}
pub(in super::super) unsafe fn sparse_construct<T>(slot: *mut c_void, value: T) {
    if sparse_indirect::<T>() {
        unsafe { slot.cast::<*mut T>().write(Box::into_raw(Box::new(value))) };
    } else {
        unsafe { slot.cast::<T>().write(value) };
    }
}
pub(in super::super) unsafe fn sparse_destroy<T>(slot: *mut c_void) {
    if sparse_indirect::<T>() {
        unsafe { drop(Box::from_raw(sparse_project::<T>(slot))) };
    } else {
        unsafe { slot.cast::<T>().drop_in_place() };
    }
}
pub(in super::super) fn sparse_checked_bytes(items: usize, stride: usize) -> usize {
    items
        .checked_mul(stride)
        .and_then(|bytes| bytes.checked_add(items.div_ceil(8)))
        .and_then(|bytes| bytes.checked_add(size_of::<usize>()))
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .unwrap_or_else(|| {
            panic!(
                "SparseArray block {items} items of stride {stride}; expected addressable layout"
            )
        })
}
