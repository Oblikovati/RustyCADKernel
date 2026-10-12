//! Opaque storage identity for safely checking native borrowed C-buffer lifetimes.
use super::AsciiString;
use super::allocation::AsciiBufferAllocation;
use crate::standard::{Failure, exceptions};
use std::{
    ffi::CString,
    sync::{Arc, Weak},
};

/// Project-owned allocation receipt, without ownership of the string bytes.
/// Example: `let receipt = text.buffer_identity(); assert!(receipt.matches(&text));`.
#[derive(Clone, Debug)]
pub struct AsciiStringBufferIdentity {
    allocation: Option<Weak<AsciiBufferAllocation>>,
    address: usize,
}
impl AsciiString {
    /// Capture the actual allocation, not a live-field reference; `text.buffer_identity()`.
    pub fn buffer_identity(&self) -> AsciiStringBufferIdentity {
        AsciiStringBufferIdentity {
            allocation: self.terminated.allocation.as_ref().map(Arc::downgrade),
            address: self.as_bytes_with_nul().as_ptr() as usize,
        }
    }
}
impl AsciiStringBufferIdentity {
    /// Check the captured physical buffer remains present; `receipt.matches(&text)`.
    pub fn matches(&self, text: &AsciiString) -> bool {
        let allocation_matches = match (&self.allocation, &text.terminated.allocation) {
            (Some(captured), Some(current)) => Weak::ptr_eq(captured, &Arc::downgrade(current)),
            (None, None) => true,
            _ => false,
        };
        allocation_matches && self.address == text.as_bytes_with_nul().as_ptr() as usize
    }
    /// Copy captured bytes under a temporary allocation lease; `receipt.observe()?.as_bytes()`.
    pub fn observe(&self) -> Result<CString, Failure> {
        let Some(weak) = &self.allocation else {
            return Ok(CString::default());
        };
        let allocation = weak
            .upgrade()
            .ok_or_else(|| self.released_buffer_failure())?;
        allocation
            .observe_prefix()
            .ok_or_else(|| self.released_buffer_failure())
    }
    fn released_buffer_failure(&self) -> Failure {
        exceptions::no_such_object(&format!(
            "filename buffer {:#x}; expected its captured allocation to retain a live owner",
            self.address
        ))
    }
    /// Identify native static-empty storage; `AsciiString::default().buffer_identity().is_static_empty()`.
    pub fn is_static_empty(&self) -> bool {
        // An owned buffer truncated to zero must remain distinct from the shared terminator.
        self.allocation.is_none()
    }
    /// Read the captured address without dereferencing it; `receipt.address()`.
    pub fn address(&self) -> usize {
        // Address equality alone cannot prove validity after a free/reallocation cycle.
        self.address
    }
}
