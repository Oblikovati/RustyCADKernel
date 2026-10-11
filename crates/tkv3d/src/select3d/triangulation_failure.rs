//! Checked native exceptions cross nonfallible BVH traits as typed panic payloads.
use rustycad_tkernel::standard::Failure;
use std::panic::{AssertUnwindSafe, catch_unwind, panic_any, resume_unwind};

pub(super) fn native_checked<T>(result: Result<T, Failure>) -> T {
    match result {
        Ok(value) => value,
        Err(failure) => panic_any(failure),
    }
}
pub(super) fn catch_triangulation_failure<T>(
    operation: impl FnOnce() -> Result<T, Failure>,
) -> Result<T, Failure> {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(result) => result,
        Err(payload) => checked_panic_payload(payload),
    }
}
fn checked_panic_payload<T>(payload: Box<dyn std::any::Any + Send>) -> Result<T, Failure> {
    match payload.downcast::<Failure>() {
        Ok(failure) => Err(*failure),
        Err(unrelated) => resume_unwind(unrelated),
    }
}
