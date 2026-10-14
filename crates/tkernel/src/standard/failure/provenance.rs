// SPDX-License-Identifier: GPL-2.0-only
//! ADR0048 exposes stored provenance for lossless diagnostics without changing native exception behavior.
use super::Failure;
impl Failure {
    /// Borrow the explicit parent override, without inferring a base from the exception hierarchy.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Failure;
    /// let value = Failure::with_type("ExternalError", "", "")
    ///     .with_exception_parent("Standard_DomainError");
    /// assert_eq!(value.explicit_exception_parent(), Some("Standard_DomainError"));
    /// ```
    pub fn explicit_exception_parent(&self) -> Option<&'static str> {
        // None remains distinct from a known inferred parent used by is_kind().
        self.exception_parent
    }
    /// Borrow the optional derived message buffer; the base message remains available separately.
    ///
    /// ```
    /// use rustycad_tkernel::standard::exceptions::out_of_memory_bytes;
    /// let value = out_of_memory_bytes(&[255]);
    /// assert_eq!(value.derived_message_bytes(), Some([255].as_slice()));
    /// assert!(value.message_bytes().is_empty());
    /// ```
    pub fn derived_message_bytes(&self) -> Option<&[u8]> {
        // Some(empty) carries different provenance from an absent override, even if what() matches.
        self.own_buffer.as_deref()
    }
}
