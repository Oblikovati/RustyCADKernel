//! Explicit native copy-constructor and trivially-copyable policies share the owned block engine.
use super::{DynamicArray, DynamicArrayStorageExtension, Failure};
use crate::ncollection::dynamic_array::DynamicArrayCopyFailure;

impl<T> DynamicArrayStorageExtension<'_, T> {
    /// Copies allocated source blocks with an explicit native copy-constructor policy.
    /// `destination.storage_extension().copy_date_with(&source, |value| Ok(*value))?;`
    /// Destination must have no allocated blocks and matching stride. A complete block is published
    /// only after all of its live values are constructed; allocated empty trailing blocks are copied.
    /// Report native constructor throws as Err. The returned failure owns the unpublished prefix
    /// without dropping it until recovery; native copyDate instead leaves that allocation leaked.
    /// A Rust callback panic uses ordinary RAII cleanup and is outside the native failure policy.
    /// Allocation failure and pointer-vector growth failure are outside the recoverable error domain.
    pub fn copy_date_with(
        &mut self,
        source: &DynamicArray<T>,
        construct: impl FnMut(&T) -> Result<T, Failure>,
    ) -> Result<(), DynamicArrayCopyFailure<T>> {
        self.array.copy_date_using(source, construct)
    }
    /// Copies values bitwise without invoking Clone, selecting native's trivially-copyable policy.
    /// `destination.storage_extension().copy_date_trivial(&source)?;`
    /// Rust Copy makes bitwise duplication sound; the caller chooses this native capability policy.
    /// The same unallocated-destination and valid matching-stride requirements apply.
    pub fn copy_date_trivial(&mut self, source: &DynamicArray<T>) -> Result<(), Failure>
    where
        T: Copy,
    {
        self.array
            .copy_date_bitwise(source)
            .map_err(DynamicArrayCopyFailure::into_failure)
    }
}
