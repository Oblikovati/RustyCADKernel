//! Owned exception messages with OCCT's printing and shared-copy semantics.
use super::runtime_type::c_prefix;
use std::sync::Arc;
mod out_of_memory;
mod stack_capture;
pub use out_of_memory::out_of_memory;

/// A transported OCCT failure; `Failure::out_of_range("test").to_string()`.
///
/// Copies share the message and stack strings, as the C++ reference-counted `StringRef`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Failure {
    exception_type: &'static str,
    message: Arc<str>,
    stack: Arc<str>,
    /// `Standard_OutOfMemory`'s own message buffer, which its `what()` returns instead.
    own_buffer: Option<Arc<[u8]>>,
}
impl Default for Failure {
    /// `Standard_Failure()`: no message, and a stack trace when the default length is > 0.
    fn default() -> Self {
        Self::new("")
    }
}
impl Failure {
    /// Constructs Standard_Failure, capturing a stack trace when
    /// [`Failure::default_stack_trace_length`] is > 0; `Failure::new("failure").message() == "failure"`.
    pub fn new(message: &str) -> Self {
        Self::with_message("Standard_Failure", message)
    }
    /// Constructs a named exception class from a message alone, as each
    /// `DEFINE_STANDARD_EXCEPTION` class's `C1(const char* theMessage = "")` does,
    /// including its stack capture; `Failure::with_message("Standard_DomainError", "bad")`.
    pub fn with_message(exception_type: &'static str, message: &str) -> Self {
        Self {
            exception_type,
            message: c_string(message),
            stack: stack_capture::capture_stack_trace(),
            own_buffer: None,
        }
    }
    /// Constructs Standard_OutOfRange; `Failure::out_of_range("test")`.
    pub fn out_of_range(message: &str) -> Self {
        Self::with_message("Standard_OutOfRange", message)
    }
    /// The `(message, stack trace)` constructor: an explicit stack and no capture;
    /// `Failure::with_type("Standard_Failure", "bad", "trace")`.
    pub fn with_type(exception_type: &'static str, message: &str, stack: &str) -> Self {
        Self {
            exception_type,
            message: c_string(message),
            stack: c_string(stack),
            own_buffer: None,
        }
    }
    /// Returns the exception name; `Failure::default().exception_type() == "Standard_Failure"`.
    pub const fn exception_type(&self) -> &'static str {
        self.exception_type
    }
    /// Borrows the base message that `Print` writes (the `StringRef` of Standard_Failure);
    /// `Failure::new("test").message() == "test"`.
    pub fn message(&self) -> &str {
        &self.message
    }
    /// The virtual `what()` / `GetMessageString()`: the base message, or the own buffer of
    /// `Standard_OutOfMemory`, whose byte truncation may split a UTF-8 sequence;
    /// `Failure::new("test").what() == b"test"`.
    pub fn what(&self) -> &[u8] {
        self.own_buffer
            .as_deref()
            .unwrap_or(self.message.as_bytes())
    }
    /// Borrows the stack string, empty when none was captured or given;
    /// `Failure::default().stack_string()`.
    pub fn stack_string(&self) -> &str {
        &self.stack
    }
    /// Assigns the base message and stack while preserving the destination exception type
    /// and derived message buffer; `target.assign_base(&source)` returns `target`.
    pub fn assign_base(&mut self, source: &Self) -> &mut Self {
        // Standard_Failure::operator= does not assign a derived exception's fields.
        self.message = Arc::clone(&source.message);
        self.stack = Arc::clone(&source.stack);
        self
    }
    /// Replaces the message while preserving copies; `failure.set_message("updated")`.
    pub fn set_message(&mut self, message: &str) {
        self.message = c_string(message);
    }
    /// Replaces the stack while preserving copies; `failure.set_stack_string("trace")`.
    pub fn set_stack_string(&mut self, stack: &str) {
        self.stack = c_string(stack);
    }
    /// Frames captured by the message constructors, 0 (no trace) by default;
    /// `Failure::default_stack_trace_length()`.
    pub fn default_stack_trace_length() -> i32 {
        stack_capture::default_length()
    }
    /// Sets the process-wide number of frames the message constructors capture; any
    /// value <= 0 disables capture; `Failure::set_default_stack_trace_length(10)`.
    pub fn set_default_stack_trace_length(length: i32) {
        stack_capture::set_default_length(length);
    }
}
fn c_string(text: &str) -> Arc<str> {
    Arc::from(c_prefix(text))
}
/// `Print` / `operator<<`: the type, `": "` and the base message when it is not empty,
/// then the stack string.
impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.exception_type)?;
        if !self.message.is_empty() {
            write!(formatter, ": {}", self.message)?;
        }
        formatter.write_str(&self.stack)
    }
}
impl std::error::Error for Failure {}
