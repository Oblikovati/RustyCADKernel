//! `Standard_OutOfMemory`: a message kept in a fixed buffer instead of the base `StringRef`.
use super::Failure;
use crate::standard::{exceptions, runtime_type::c_prefix};
use std::sync::Arc;

/// Size of the C++ `myBuffer`; one byte is kept for the terminating NUL.
const OUT_OF_MEMORY_BUFFER_SIZE: usize = 1024;

/// Constructs `Standard_OutOfMemory` (a `Standard_ProgramError`). The message goes to
/// the class's own buffer, cut to its first 1023 bytes, so `what()` returns it; the
/// base message stays empty, so `Display` (C++ `operator<<`) prints only the class name
/// and any stack trace. `out_of_memory("no memory").what() == b"no memory"`.
pub fn out_of_memory(message: &str) -> Failure {
    let mut failure = Failure::with_message("Standard_OutOfMemory", "");
    failure.own_buffer = Some(truncated_message(message));
    failure
}
/// The first bytes of `message` that fit the buffer, up to its first NUL.
fn truncated_message(message: &str) -> Arc<[u8]> {
    let bytes: &[u8] = c_prefix(message).as_bytes();
    Arc::from(&bytes[..bytes.len().min(OUT_OF_MEMORY_BUFFER_SIZE - 1)])
}
impl Failure {
    /// `Standard_OutOfMemory::SetMessageString`: replaces the buffered message with the
    /// first 1023 bytes of `message`. Only an [`out_of_memory`] failure has the buffer;
    /// any other failure is refused with `Standard_TypeMismatch`.
    /// `failure.set_message_string("retry")?`.
    pub fn set_message_string(&mut self, message: &str) -> Result<(), Failure> {
        if self.own_buffer.is_none() {
            return Err(exceptions::type_mismatch(&format!(
                "SetMessageString on a {} failure; expected a Standard_OutOfMemory",
                self.exception_type
            )));
        }
        self.own_buffer = Some(truncated_message(message));
        Ok(())
    }
}
