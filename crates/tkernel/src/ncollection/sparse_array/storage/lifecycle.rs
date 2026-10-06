//! Defer Rust destructor panics until the native engine has restored counts and freed blocks.
use super::layout::sparse_destroy;
use crate::ncollection::destructor_unwind::DeferredDestructorPanic;
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};
pub(super) unsafe fn sparse_noop_destroy(_: *mut c_void) {
    // This base only owns bytes; its private typed owner drains objects before base field Drop.
}
pub(super) fn sparse_with_destruction<T, R>(
    operation: impl FnOnce(&mut dyn FnMut(*mut c_void)) -> R,
) -> R {
    let mut panics = DeferredDestructorPanic::default();
    let result = catch_unwind(AssertUnwindSafe(|| {
        operation(&mut |slot| panics.capture(|| unsafe { sparse_destroy::<T>(slot) }))
    }));
    panics.finish(result)
}
