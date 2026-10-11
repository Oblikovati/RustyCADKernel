//! Native exceptions cross nonfallible virtual hooks as typed Failure panic payloads.
use super::Failure;
use std::panic::{AssertUnwindSafe, catch_unwind, panic_any, resume_unwind};
/// Transport a checked error across a void hook; `raise_native_failure(result)` preserves success values.
pub fn raise_native_failure<T>(result: Result<T, Failure>) -> T {
    match result {
        Ok(value) => value,
        Err(failure) => panic_any(failure),
    }
}
/// Recover only typed native failures; `catch_native_failure(|| Ok(42))?` resumes unrelated panics.
pub fn catch_native_failure<T>(
    operation: impl FnOnce() -> Result<T, Failure>,
) -> Result<T, Failure> {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(result) => result,
        Err(payload) => failure_transport_payload(payload),
    }
}
fn failure_transport_payload<T>(payload: Box<dyn std::any::Any + Send>) -> Result<T, Failure> {
    match payload.downcast::<Failure>() {
        Ok(failure) => Err(*failure),
        Err(unrelated) => resume_unwind(unrelated),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_success_and_typed_failure_preserve_payload_identity() {
        assert_eq!(
            catch_native_failure(|| Ok(raise_native_failure(Ok(42)))).unwrap(),
            42
        );
        let failed: Result<(), Failure> = catch_native_failure(|| {
            raise_native_failure(Err(Failure::new("hook rejected value=7; expected 0")))
        });
        assert!(failed.unwrap_err().to_string().contains("value=7"));
    }
    #[test]
    fn result_failures_pass_through_without_becoming_panics() {
        let result: Result<(), Failure> =
            catch_native_failure(|| Err(Failure::new("value=2; expected 1")));
        assert!(result.unwrap_err().to_string().contains("value=2"));
    }
    #[test]
    fn non_failure_panics_resume_to_the_original_caller() {
        let result = catch_unwind(|| catch_native_failure::<()>(|| panic_any(73_u32)));
        assert_eq!(*result.unwrap_err().downcast::<u32>().unwrap(), 73);
    }
}
