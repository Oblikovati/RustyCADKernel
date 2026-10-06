//! Complete native destructor callbacks before resuming the first Rust panic.
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};
type DestructorPanic = Box<dyn Any + Send>;
#[derive(Default)]
pub(super) struct DeferredDestructorPanic(Option<DestructorPanic>);
impl DeferredDestructorPanic {
    pub(super) fn capture(&mut self, destroy: impl FnOnce()) {
        if let Err(payload) = catch_unwind(AssertUnwindSafe(destroy)) {
            self.retain_first(payload);
        }
    }
    fn retain_first(&mut self, payload: DestructorPanic) {
        if self.0.is_none() {
            self.0 = Some(payload);
        } else {
            discard_destructor_panic(payload);
        }
    }
    pub(super) fn finish<R>(mut self, result: Result<R, DestructorPanic>) -> R {
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
fn discard_destructor_panic(mut payload: DestructorPanic) {
    // A discarded user payload may itself panic in Drop. Drain each resulting payload without
    // leaking it or allowing an unwind to escape through native destruction callbacks.
    loop {
        match catch_unwind(AssertUnwindSafe(|| drop(payload))) {
            Ok(()) => return,
            Err(nested) => payload = nested,
        }
    }
}
