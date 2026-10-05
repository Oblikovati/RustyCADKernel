//! Owned exception messages with OCCT's printing and shared-copy semantics.
use std::sync::Arc;
mod byte_messages;
mod text;
use text::FailureText;
mod out_of_memory;
mod stack_capture;
pub use out_of_memory::{out_of_memory, out_of_memory_bytes};

/// A transported OCCT failure; `Failure::out_of_range("test").to_string()`.
///
/// Copies share the message and stack strings, as the C++ reference-counted `StringRef`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Failure {
    exception_type: &'static str,
    message: FailureText,
    stack: FailureText,
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
        Self::with_message_bytes(exception_type, message.as_bytes())
    }
    /// Constructs Standard_OutOfRange; `Failure::out_of_range("test")`.
    pub fn out_of_range(message: &str) -> Self {
        Self::with_message("Standard_OutOfRange", message)
    }
    /// The `(message, stack trace)` constructor: an explicit stack and no capture;
    /// `Failure::with_type("Standard_Failure", "bad", "trace")`.
    pub fn with_type(exception_type: &'static str, message: &str, stack: &str) -> Self {
        Self::with_type_bytes(exception_type, message.as_bytes(), stack.as_bytes())
    }
    /// Returns the exception name; `Failure::default().exception_type() == "Standard_Failure"`.
    pub const fn exception_type(&self) -> &'static str {
        self.exception_type
    }
    /// Borrows a UTF-8 presentation of the base message (replacement characters for invalid bytes);
    /// `Failure::new("test").message() == "test"`.
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
    /// The virtual `what()` / `GetMessageString()`: the base message, or the own buffer of
    /// `Standard_OutOfMemory`, whose byte truncation may split a UTF-8 sequence;
    /// `Failure::new("test").what() == b"test"`.
    pub fn what(&self) -> &[u8] {
        self.own_buffer
            .as_deref()
            .unwrap_or(self.message.as_bytes())
    }
    /// Borrows a UTF-8 presentation of the stack, replacing invalid bytes;
    /// `Failure::default().stack_string()`.
    pub fn stack_string(&self) -> &str {
        self.stack.as_str()
    }
    /// Copies only the base exception state, as C++ base copy construction does;
    /// `Failure::copy_base(&derived).exception_type() == "Standard_Failure"`.
    pub fn copy_base(source: &Self) -> Self {
        Self {
            exception_type: "Standard_Failure",
            message: source.message.clone(),
            stack: source.stack.clone(),
            own_buffer: None,
        }
    }
    /// Assigns the base message and stack while preserving the destination exception type
    /// and derived message buffer; `target.assign_base(&source)` returns `target`.
    pub fn assign_base(&mut self, source: &Self) -> &mut Self {
        // Standard_Failure::operator= does not assign a derived exception's fields.
        self.message = source.message.clone();
        self.stack = source.stack.clone();
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
fn c_string(text: &str) -> FailureText {
    FailureText::from_bytes(text.as_bytes())
}
/// UTF-8 presentation of the type, message, and stack. Native byte-preserving
/// output uses [`Failure::print_to`] when the message or stack is not UTF-8.
impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.exception_type)?;
        if !self.message.as_bytes().is_empty() {
            write!(formatter, ": {}", self.message.as_str())?;
        }
        formatter.write_str(self.stack.as_str())
    }
}
impl std::error::Error for Failure {}
