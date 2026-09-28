//! Standard::StackTrace and the stack capture of Standard_Failure.
//!
//! The default stack-trace length is process-wide, so every test here takes one lock
//! and restores the length to 0 before releasing it.
use rustycad_tkernel::standard::{Failure, exceptions, stack_trace};
use std::sync::{Mutex, MutexGuard, PoisonError};

static STACK_LENGTH: Mutex<()> = Mutex::new(());

/// Holds the lock while the default length is `length`, and resets it to 0 on drop.
struct StackLengthGuard {
    _lock: MutexGuard<'static, ()>,
}
impl StackLengthGuard {
    fn set(length: i32) -> Self {
        let lock = STACK_LENGTH.lock().unwrap_or_else(PoisonError::into_inner);
        Failure::set_default_stack_trace_length(length);
        Self { _lock: lock }
    }
}
impl Drop for StackLengthGuard {
    fn drop(&mut self) {
        Failure::set_default_stack_trace_length(0);
    }
}
#[test]
fn invalid_arguments_leave_the_buffer_unchanged() {
    let mut buffer = String::from("keep");
    assert!(!stack_trace(&mut buffer, 0, 10, 0));
    assert!(!stack_trace(&mut buffer, 100, 0, 0));
    assert!(!stack_trace(&mut buffer, 100, 10, -1));
    assert!(!stack_trace(&mut buffer, 4, 10, 0));
    assert!(!stack_trace(&mut buffer, 100, 10, 100_000));
    assert_eq!("keep", buffer);
}
#[test]
fn small_buffers_keep_only_what_fits() {
    let mut header_only = String::new();
    assert!(stack_trace(&mut header_only, 20, 10, 0));
    assert_eq!("\n==Backtrace==", header_only);
    let mut appended = String::from("abc");
    assert!(stack_trace(&mut appended, 20, 10, 0));
    assert_eq!("abc\n==Backtrace==", appended);
    let mut nothing = String::new();
    assert!(stack_trace(&mut nothing, 15, 10, 0));
    assert_eq!("", nothing);
}
#[test]
fn large_buffers_hold_frames_between_header_and_footer() {
    let mut trace = String::new();
    assert!(stack_trace(&mut trace, 4096, 3, 0));
    assert!(trace.starts_with("\n==Backtrace==\n"));
    assert!(trace.ends_with("\n============="));
    let frames = trace.lines().count() - 3;
    assert!((1..=3).contains(&frames), "{trace}");
    assert!(trace.len() < 4096);
}
#[test]
fn failures_capture_a_trace_only_when_the_length_is_positive() {
    let off = StackLengthGuard::set(0);
    assert_eq!(0, Failure::default_stack_trace_length());
    assert_eq!("", Failure::new("x").stack_string());
    drop(off);
    let _on = StackLengthGuard::set(4);
    assert_eq!(4, Failure::default_stack_trace_length());
    let captured = exceptions::out_of_range("x");
    assert!(captured.stack_string().starts_with("\n==Backtrace=="));
    assert!(
        captured
            .to_string()
            .starts_with("Standard_OutOfRange: x\n==Backtrace==")
    );
    let oom = exceptions::out_of_memory("m");
    assert!(
        oom.to_string()
            .starts_with("Standard_OutOfMemory\n==Backtrace==")
    );
    let explicit = Failure::with_type("Standard_Failure", "x", "");
    assert_eq!("", explicit.stack_string());
}
