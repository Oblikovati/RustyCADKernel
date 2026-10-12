//! Captured native filename allocation; never retarget a retained pointer to a new field buffer.
use super::AisTexturedShapeFields;
use rustycad_tkernel::{
    standard::{Failure, SharedState},
    tcollection::AsciiStringBufferIdentity,
};
use std::ffi::CString;

/// Borrowed filename allocation without an object owner or persistent field lock.
/// Example: `let file=object.texture_file(); object.set_texture_file_name(&"abc".into())?; file.read()?;`.
pub struct AisTexturedShapeFileRef {
    allocation: AsciiStringBufferIdentity,
}
impl AisTexturedShapeFileRef {
    pub(super) fn new(fields: &SharedState<AisTexturedShapeFields>) -> Self {
        let allocation = fields.read().texture_file.buffer_identity();
        Self { allocation }
    }
    /// Observe captured bytes then release the guard before callbacks; `file.read()?.as_bytes()`.
    pub fn read(&self) -> Result<CString, Failure> {
        // Weak allocation observation follows physical storage through Move/Swap, not source fields.
        self.allocation.observe()
    }
    /// Read the captured native pointer without dereferencing; `file.as_ptr()`.
    pub fn as_ptr(&self) -> *const std::ffi::c_char {
        // Unsafe dereferences retain the native validity requirements after any mutation.
        self.allocation.address() as *const std::ffi::c_char
    }
}
