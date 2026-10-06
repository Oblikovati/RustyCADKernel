//! One binary owns its global setting; restore the previous value even if an assertion panics.
use rustycad_tkernel::standard::Failure;
use std::sync::{Mutex, PoisonError};
static STACK_SETTING_LOCK: Mutex<()> = Mutex::new(());
struct StackRestore(i32);
impl StackRestore {
    fn set(frames: i32) -> Self {
        let previous = Failure::default_stack_trace_length();
        Failure::set_default_stack_trace_length(frames);
        Self(previous)
    }
}
impl Drop for StackRestore {
    fn drop(&mut self) {
        Failure::set_default_stack_trace_length(self.0);
    }
}
pub(super) fn with_length(frames: i32, check: impl FnOnce()) {
    let _lock = STACK_SETTING_LOCK
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let _restore = StackRestore::set(frames);
    check();
    assert_eq!(Failure::default_stack_trace_length(), frames);
}
#[test]
fn iterator_stack_setting_restores_after_unwind() {
    let _lock = STACK_SETTING_LOCK
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let previous = Failure::default_stack_trace_length();
    let failure = std::panic::catch_unwind(|| {
        let _restore = StackRestore::set(-37);
        panic!("injected test panic: expected stack setting restoration");
    });
    assert!(failure.is_err());
    assert_eq!(Failure::default_stack_trace_length(), previous);
}
