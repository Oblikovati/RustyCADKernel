//! Defer Rust destructor panics until the native engine has restored counts and freed blocks.
use super::layout::sparse_destroy;
use std::{
    any::Any,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};
type SparsePanic = Box<dyn Any + Send>;

pub(super) unsafe fn sparse_noop_destroy(_: *mut c_void) {
    // This base only owns bytes; its private typed owner drains objects before base field Drop.
}
#[derive(Default)]
struct SparseDestructorPanic(Option<SparsePanic>);
impl SparseDestructorPanic {
    fn destroy<T>(&mut self, slot: *mut c_void) {
        let result = catch_unwind(AssertUnwindSafe(|| unsafe { sparse_destroy::<T>(slot) }));
        if let Err(payload) = result {
            self.retain_first(payload);
        }
    }
    fn retain_first(&mut self, payload: SparsePanic) {
        if self.0.is_none() {
            self.0 = Some(payload);
        } else {
            discard_sparse_panic(payload);
        }
    }
    fn finish<R>(mut self, result: Result<R, SparsePanic>) -> R {
        match result {
            Ok(value) if self.0.is_none() => value,
            Ok(_) => resume_unwind(self.0.take().unwrap()),
            Err(payload) => {
                self.retain_first(payload);
                resume_unwind(self.0.take().unwrap())
            }
        }
    }
}
fn discard_sparse_panic(mut payload: SparsePanic) {
    // A discarded user payload may itself panic in Drop. Drain each resulting payload without
    // leaking it or allowing an unwind to escape through native destruction callbacks.
    loop {
        match catch_unwind(AssertUnwindSafe(|| drop(payload))) {
            Ok(()) => return,
            Err(nested) => payload = nested,
        }
    }
}
pub(super) fn sparse_with_destruction<T, R>(
    operation: impl FnOnce(&mut dyn FnMut(*mut c_void)) -> R,
) -> R {
    let mut panics = SparseDestructorPanic::default();
    let result = catch_unwind(AssertUnwindSafe(|| {
        operation(&mut |slot| panics.destroy::<T>(slot))
    }));
    panics.finish(result)
}
