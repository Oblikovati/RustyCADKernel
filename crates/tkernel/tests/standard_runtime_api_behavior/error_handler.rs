//! Standard_ErrorHandler linking, callbacks, Abort and Raise.
use rustycad_tkernel::standard::{ErrorHandler, ErrorHandlerCallback, Failure, exceptions};
use std::{cell::RefCell, process::Command, rc::Rc};

/// A named fake that records every DestroyCallback call.
struct CallbackLog {
    calls: Rc<RefCell<Vec<&'static str>>>,
}
impl CallbackLog {
    fn new() -> Self {
        Self {
            calls: Rc::default(),
        }
    }
    fn callback(&self, name: &'static str) -> ErrorHandlerCallback {
        let calls = self.calls.clone();
        ErrorHandlerCallback::new(move || calls.borrow_mut().push(name))
    }
    fn calls(&self) -> Vec<&'static str> {
        self.calls.borrow().clone()
    }
}
#[test]
fn handlers_nest_and_unlink_in_order() {
    assert!(!ErrorHandler::is_in_try_block());
    let outer = ErrorHandler::new();
    let inner = ErrorHandler::new();
    assert!(ErrorHandler::is_in_try_block());
    drop(inner);
    assert!(ErrorHandler::is_in_try_block());
    drop(outer);
    assert!(!ErrorHandler::is_in_try_block());
}
#[test]
fn destroying_an_unlinked_handler_unlinks_the_whole_stack() {
    let outer = ErrorHandler::new();
    let inner = ErrorHandler::new();
    inner.destroy();
    assert!(ErrorHandler::is_in_try_block());
    inner.destroy();
    assert!(!ErrorHandler::is_in_try_block());
    drop(outer);
    assert!(!ErrorHandler::is_in_try_block());
}
#[test]
fn callbacks_run_newest_first_when_their_handler_unlinks() {
    let log = CallbackLog::new();
    let (first, second) = (log.callback("first"), log.callback("second"));
    first.register_callback();
    assert!(!first.is_registered());
    let outer = ErrorHandler::new();
    let inner = ErrorHandler::new();
    first.register_callback();
    second.register_callback();
    outer.destroy();
    assert_eq!(vec!["second", "first"], log.calls());
    assert!(!ErrorHandler::is_in_try_block());
    drop(inner);
    assert_eq!(2, log.calls().len());
}
#[test]
fn unregistered_callbacks_are_not_called() {
    let log = CallbackLog::new();
    let callback = log.callback("gone");
    let handler = ErrorHandler::new();
    callback.register_callback();
    callback.unregister_callback();
    drop(handler);
    assert!(log.calls().is_empty());
    callback.destroy_callback();
    assert_eq!(vec!["gone"], log.calls());
}
#[test]
fn a_callback_left_registered_stays_bound_to_its_dead_handler() {
    let log = CallbackLog::new();
    let callback = log.callback("stale");
    drop_handler_with(&callback);
    assert_eq!(vec!["stale"], log.calls());
    assert!(callback.is_registered());
    let next = ErrorHandler::new();
    callback.register_callback();
    drop(next);
    assert_eq!(1, log.calls().len());
    callback.unregister_callback();
    assert!(!callback.is_registered());
}
fn drop_handler_with(callback: &ErrorHandlerCallback) {
    let handler = ErrorHandler::new();
    callback.register_callback();
    drop(handler);
}
#[test]
fn abort_lands_on_the_innermost_catch_block() {
    let result: Result<i32, Failure> = ErrorHandler::catch_signals(|| {
        assert!(ErrorHandler::is_in_try_block());
        ErrorHandler::abort(exceptions::divide_by_zero("zero"))?;
        Ok(1)
    });
    let failure = result.unwrap_err();
    assert_eq!("Standard_DivideByZero", failure.exception_type());
    assert_eq!("zero", failure.message());
    assert!(!ErrorHandler::is_in_try_block());
}
#[test]
fn a_swallowed_abort_still_lands_on_the_label() {
    let result: Result<i32, Failure> = ErrorHandler::catch_signals(|| {
        let _ignored = ErrorHandler::abort(Failure::with_message("OSD_SIGSEGV", "segv"));
        Ok(5)
    });
    assert_eq!("OSD_SIGSEGV", result.unwrap_err().exception_type());
    assert_eq!(Ok(3), ErrorHandler::catch_signals(|| Ok(3)));
}
#[test]
fn error_and_raise_report_the_stored_failure() {
    let handler = ErrorHandler::new();
    assert!(handler.error().is_none());
    let aborted = ErrorHandler::abort(exceptions::overflow("big")).unwrap_err();
    assert_eq!(Some(aborted.clone()), handler.error());
    assert_eq!(aborted, handler.raise().unwrap_err());
}
#[test]
fn abort_refuses_classes_outside_the_signal_variant() {
    let handler = ErrorHandler::new();
    let refused = ErrorHandler::abort(exceptions::out_of_range("x")).unwrap_err();
    assert_eq!("Standard_TypeMismatch", refused.exception_type());
    assert!(refused.message().contains("Standard_OutOfRange"));
    assert!(handler.error().is_none());
}
/// Runs one test of this binary in a child process that sees `CHILD_ENV`.
fn run_child(test: &str) -> std::process::Output {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env(CHILD_ENV, "1")
        .output()
        .unwrap()
}
const CHILD_ENV: &str = "RUSTYCAD_ERROR_HANDLER_CHILD";
#[test]
fn abort_without_handler_exits_with_status_1() {
    if std::env::var_os(CHILD_ENV).is_some() {
        let _ = ErrorHandler::abort(exceptions::divide_by_zero("lost"));
        return;
    }
    let output = run_child("error_handler::abort_without_handler_exits_with_status_1");
    assert_eq!(Some(1), output.status.code());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("*** Abort *** an exception was raised, but no catch was found.\n"));
    assert!(stderr.contains("\t... The exception is: lost\n"));
}
#[test]
fn raise_without_error_exits_with_status_1() {
    if std::env::var_os(CHILD_ENV).is_some() {
        let _ = ErrorHandler::new().raise();
        return;
    }
    let output = run_child("error_handler::raise_without_error_exits_with_status_1");
    assert_eq!(Some(1), output.status.code());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(
        "*** Abort *** an exception handler was called, but no exception object is set.\n"
    ));
}
