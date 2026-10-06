//! Failed native construction leaks payload blocks; this explicit owner postpones Rust recovery.
use super::{DynamicArray, DynamicArrayCopyFailure};
use std::fmt;

/// Owns copied values after a failed copy constructor, before caller-selected Rust cleanup.
/// `let (published, incomplete) = error.into_parts(); drop((published, incomplete));`.
/// Native destroys its pointer-table member on constructor unwind but leaks the pointed-to blocks.
/// This owner retains Rust's descriptor storage as well; it does not claim descriptor-unwind parity.
pub struct DynamicArrayCopyConstructionFailure<T> {
    published: DynamicArray<T>,
    incomplete: DynamicArrayCopyFailure<T>,
}
impl<T> DynamicArrayCopyConstructionFailure<T> {
    pub(super) fn new(published: DynamicArray<T>, incomplete: DynamicArrayCopyFailure<T>) -> Self {
        Self {
            published,
            incomplete,
        }
    }
    /// Reads the completely published prefix without performing recovery.
    /// `assert_eq!(error.published_array().size(), 2);`.
    pub fn published_array(&self) -> &DynamicArray<T> {
        &self.published
    }
    /// Reads the constructor failure and unpublished live prefix before cleanup.
    /// `error.copy_failure().incomplete_values()`.
    pub fn copy_failure(&self) -> &DynamicArrayCopyFailure<T> {
        &self.incomplete
    }
    /// Transfers both recovery owners; dropping them destroys their values and frees blocks.
    /// `let (published, incomplete) = error.into_parts(); drop(published); drop(incomplete);`.
    /// This is explicit Rust recovery of native leaks, not native constructor unwind behavior.
    pub fn into_parts(self) -> (DynamicArray<T>, DynamicArrayCopyFailure<T>) {
        (self.published, self.incomplete)
    }
}
impl<T> fmt::Debug for DynamicArrayCopyConstructionFailure<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DynamicArrayCopyConstructionFailure")
            .field("published_size", &self.published.size())
            .field("copy_failure", &self.incomplete)
            .finish()
    }
}
impl<T> fmt::Display for DynamicArrayCopyConstructionFailure<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.incomplete, formatter)
    }
}
impl<T> std::error::Error for DynamicArrayCopyConstructionFailure<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.incomplete.failure())
    }
}
