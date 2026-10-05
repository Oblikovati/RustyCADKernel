//! Byte-oriented counterparts of native exception message and stream APIs.
use super::{Failure, stack_capture, text::FailureText};
use std::io::{Result, Write};

impl Failure {
    /// Constructs a base failure from a bounded C string; `Failure::from_message_bytes(&[255]).what() == [255]`.
    pub fn from_message_bytes(message: &[u8]) -> Self {
        Self::with_message_bytes("Standard_Failure", message)
    }

    /// Constructs a named exception without interpreting message encoding;
    /// `Failure::with_message_bytes("Standard_DomainError", &[255])`.
    pub fn with_message_bytes(exception_type: &'static str, message: &[u8]) -> Self {
        Self {
            exception_type,
            message: FailureText::from_bytes(message),
            stack: stack_capture::capture_stack_trace(),
            own_buffer: None,
        }
    }

    /// Uses explicit bounded C strings without capturing a stack;
    /// `Failure::with_type_bytes("Standard_Failure", &[255], &[254])`.
    pub fn with_type_bytes(exception_type: &'static str, message: &[u8], stack: &[u8]) -> Self {
        Self {
            exception_type,
            message: FailureText::from_bytes(message),
            stack: FailureText::from_bytes(stack),
            own_buffer: None,
        }
    }

    /// Borrows the base message bytes used by native Print; `failure.message_bytes()`.
    pub fn message_bytes(&self) -> &[u8] {
        self.message.as_bytes()
    }

    /// Borrows the stack without UTF-8 conversion; `failure.stack_bytes()`.
    pub fn stack_bytes(&self) -> &[u8] {
        self.stack.as_bytes()
    }

    /// Prints native bytes to a Rust writer; `failure.print_to(&mut Vec::new())?`.
    pub fn print_to<W: Write + ?Sized>(&self, stream: &mut W) -> Result<()> {
        stream.write_all(self.exception_type.as_bytes())?;
        if !self.message_bytes().is_empty() {
            stream.write_all(b": ")?;
            stream.write_all(self.message_bytes())?;
        }
        stream.write_all(self.stack_bytes())
    }

    /// Prints bytes and returns the same stream for chaining; `failure.insert_into(&mut buffer)?`.
    pub fn insert_into<'stream, W: Write + ?Sized>(
        &self,
        stream: &'stream mut W,
    ) -> Result<&'stream mut W> {
        self.print_to(stream)?;
        Ok(stream)
    }
}
