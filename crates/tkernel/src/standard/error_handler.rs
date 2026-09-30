//! `Standard_ErrorHandler`: try-block registration per thread, as built with
//! `OCC_CONVERT_SIGNALS` (the Linux build).
//!
//! C++ jumps with `longjmp` from `Abort` to the `setjmp` label of the innermost handler.
//! Rust has no jump: [`ErrorHandler::abort`] stores the failure in that handler and
//! returns it as `Err`, which the caller propagates with `?` up to the handler's
//! [`ErrorHandler::catch_signals`] block; that block then raises the stored failure,
//! as the `OCC_CATCH_SIGNALS` label does.
use super::Failure;
use std::{convert::Infallible, marker::PhantomData, rc::Rc};
mod callback;
mod handler_stack;
pub use callback::ErrorHandlerCallback;
use handler_stack::HandlerId;

/// Classes of the C++ `SignalException` variant, the only ones `Abort` accepts.
pub const SIGNAL_EXCEPTION_TYPES: [&str; 23] = [
    "OSD_SIGBUS",
    "OSD_SIGHUP",
    "OSD_SIGILL",
    "OSD_SIGINT",
    "OSD_SIGKILL",
    "OSD_SIGQUIT",
    "OSD_SIGSEGV",
    "OSD_SIGSYS",
    "OSD_Exception_ACCESS_VIOLATION",
    "OSD_Exception_ARRAY_BOUNDS_EXCEEDED",
    "OSD_Exception_ILLEGAL_INSTRUCTION",
    "OSD_Exception_IN_PAGE_ERROR",
    "OSD_Exception_INT_OVERFLOW",
    "OSD_Exception_INVALID_DISPOSITION",
    "OSD_Exception_NONCONTINUABLE_EXCEPTION",
    "OSD_Exception_PRIV_INSTRUCTION",
    "OSD_Exception_STACK_OVERFLOW",
    "OSD_Exception_STATUS_NO_MEMORY",
    "Standard_DivideByZero",
    "Standard_NumericError",
    "Standard_Overflow",
    "Standard_ProgramError",
    "Standard_Underflow",
];

/// One active try block of the current thread; `let handler = ErrorHandler::new();`.
pub struct ErrorHandler {
    id: HandlerId,
    // The handler stack is per thread, so a handler must stay on its thread.
    thread_bound: PhantomData<Rc<()>>,
}
impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}
impl ErrorHandler {
    /// Links a new handler on top of this thread's stack, as the constructor;
    /// `ErrorHandler::new()`.
    pub fn new() -> Self {
        Self {
            id: handler_stack::push_handler(),
            thread_bound: PhantomData,
        }
    }
    /// `Destroy()`: unlinks this handler (and any handler above it) and runs its
    /// registered callbacks. Calling it on a handler that is no longer linked unlinks
    /// every handler of the thread, as in C++; `handler.destroy()`.
    pub fn destroy(&self) {
        handler_stack::unlink(self.id);
    }
    /// `IsInTryBlock()`: true while any handler of this thread is linked;
    /// `ErrorHandler::is_in_try_block()`.
    pub fn is_in_try_block() -> bool {
        handler_stack::top_handler().is_some()
    }
    /// `Error()`: the failure `Abort` stored in this handler, `None` for the empty
    /// variant state; `handler.error()`.
    pub fn error(&self) -> Option<Failure> {
        handler_stack::with_frame(self.id, |frame| frame.caught.clone()).flatten()
    }
    /// `Raise()`: returns the stored failure as `Err`. Without one, C++ prints
    /// "*** Abort *** an exception handler was called, but no exception object is set."
    /// to stderr and exits the process with status 1; so does this; `handler.raise()?`.
    pub fn raise(&self) -> Result<Infallible, Failure> {
        let Some(caught) = self.error() else {
            eprintln!(
                "*** Abort *** an exception handler was called, but no exception object is set."
            );
            std::process::exit(1);
        };
        Err(caught)
    }
    /// `Abort(theError)`: stores `error` in the innermost handler and returns it as `Err`
    /// for `?` to carry to that handler's block. `error` must be one of
    /// [`SIGNAL_EXCEPTION_TYPES`] (C++ rejects other classes at compile time; Rust returns
    /// `Standard_TypeMismatch`). Without an active handler, C++ prints the two
    /// "*** Abort ***" lines to stderr and exits with status 1; so does this.
    /// `ErrorHandler::abort(exceptions::divide_by_zero("x"))?`.
    pub fn abort(error: Failure) -> Result<Infallible, Failure> {
        check_signal_type(&error)?;
        let Some(active) = handler_stack::top_handler() else {
            eprintln!("*** Abort *** an exception was raised, but no catch was found.");
            eprintln!(
                "\t... The exception is: {}",
                String::from_utf8_lossy(error.what())
            );
            std::process::exit(1);
        };
        handler_stack::with_frame(active, |frame| frame.caught = Some(error.clone()));
        Err(error)
    }
    /// The `OCC_CATCH_SIGNALS` block (`Standard_ErrorHandler h; if (setjmp(h.Label()))
    /// h.Raise();` then the body): runs `body` under a new handler; when an `abort` stored
    /// a failure in it, the handler is destroyed and that failure is returned, whatever
    /// `body` returned, as the jump always lands on the label. Otherwise the handler is
    /// destroyed and `body`'s result is returned.
    /// `ErrorHandler::catch_signals(|| compute())?`.
    pub fn catch_signals<T>(body: impl FnOnce() -> Result<T, Failure>) -> Result<T, Failure> {
        let handler: ErrorHandler = Self::new();
        let outcome: Result<T, Failure> = body();
        let caught: Option<Failure> = handler.error();
        drop(handler);
        caught.map_or(outcome, Err)
    }
}
/// `~Standard_ErrorHandler()` calls `Destroy()`.
impl Drop for ErrorHandler {
    fn drop(&mut self) {
        self.destroy();
        handler_stack::remove_frame(self.id);
    }
}
/// Refuses a class outside the C++ `SignalException` variant.
fn check_signal_type(error: &Failure) -> Result<(), Failure> {
    if SIGNAL_EXCEPTION_TYPES.contains(&error.exception_type()) {
        return Ok(());
    }
    Err(super::exceptions::type_mismatch(&format!(
        "Abort with {}; expected one of the Standard_ErrorHandler::SignalException classes",
        error.exception_type()
    )))
}
