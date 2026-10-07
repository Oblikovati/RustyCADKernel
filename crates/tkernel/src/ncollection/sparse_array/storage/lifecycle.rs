//! Typed destruction callbacks for the raw sparse engine, which must not unwind mid-update.
use super::layout::sparse_destroy;
use crate::ncollection::native_noexcept::terminate_on_unwind;
use std::ffi::c_void;
pub(super) unsafe fn sparse_noop_destroy(_: *mut c_void) {
    // This base only owns bytes; its private typed owner drains objects before base field Drop.
}
pub(super) fn sparse_with_destruction<T, R>(
    operation: impl FnOnce(&mut dyn FnMut(*mut c_void)) -> R,
) -> R {
    // Native item destructors are noexcept; the engine's counts assume they return.
    operation(&mut |slot| terminate_on_unwind(|| unsafe { sparse_destroy::<T>(slot) }))
}
