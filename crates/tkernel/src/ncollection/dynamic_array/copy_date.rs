//! Whole-block copy publication with recoverable ownership of native's unpublished failure block.
use super::{AllocVec, BaseAllocator, DynamicArray};
use crate::standard::Failure;
use std::fmt;

#[allow(unsafe_code)]
mod copy_bits;

/// Retains an unpublished copy block until explicit recovery or ordinary Rust Drop.
/// `let initialized = error.incomplete_values().len(); drop(error);`
/// Native copyDate leaks this block on a constructor throw. This token preserves the same
/// pre-cleanup observations while keeping ownership reachable; dropping it performs cleanup.
pub struct DynamicArrayCopyFailure<T> {
    failure: Box<Failure>,
    incomplete: Option<AllocVec<T, BaseAllocator>>,
}
impl<T> DynamicArrayCopyFailure<T> {
    /// Borrows the explicit constructor or storage-domain failure; `error.failure().message()`.
    pub fn failure(&self) -> &Failure {
        &self.failure
    }
    /// Reports whether a detached allocation was retained, even with no initialized values.
    /// `assert!(error.has_incomplete_block());`
    pub fn has_incomplete_block(&self) -> bool {
        self.incomplete.is_some()
    }
    /// Borrows only fully constructed values in the unpublished block; `error.incomplete_values()`.
    /// The failed construction contributes no value and no destructor obligation.
    pub fn incomplete_values(&self) -> &[T] {
        self.incomplete.as_deref().unwrap_or(&[])
    }
    /// Cleans the unpublished block and returns its failure; `let reason = error.into_failure();`.
    /// Cleanup is explicit Rust recovery, not an assertion that native copyDate frees on throw.
    pub fn into_failure(self) -> Failure {
        *self.failure
    }
    fn domain(failure: Failure) -> Self {
        Self {
            failure: Box::new(failure),
            incomplete: None,
        }
    }
    fn construction(failure: Failure, incomplete: AllocVec<T, BaseAllocator>) -> Self {
        Self {
            failure: Box::new(failure),
            incomplete: Some(incomplete),
        }
    }
}
impl<T> fmt::Debug for DynamicArrayCopyFailure<T> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output
            .debug_struct("DynamicArrayCopyFailure")
            .field("failure", &self.failure)
            .field("has_incomplete_block", &self.has_incomplete_block())
            .field("initialized_values", &self.incomplete_values().len())
            .finish()
    }
}
impl<T> fmt::Display for DynamicArrayCopyFailure<T> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.failure, output)
    }
}
impl<T> std::error::Error for DynamicArrayCopyFailure<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.failure.as_ref())
    }
}
impl<T> DynamicArray<T> {
    pub(super) fn copy_date_using(
        &mut self,
        source: &Self,
        mut construct: impl FnMut(&T) -> Result<T, Failure>,
    ) -> Result<(), DynamicArrayCopyFailure<T>> {
        self.copy_date_blocks(source, |values, block| {
            for value in values {
                block.push(construct(value)?);
            }
            Ok(())
        })
    }
    pub(super) fn copy_date_bitwise(
        &mut self,
        source: &Self,
    ) -> Result<(), DynamicArrayCopyFailure<T>>
    where
        T: Copy,
    {
        self.copy_date_blocks(source, |values, block| {
            copy_bits::copy_bitwise_values(values, block);
            Ok(())
        })
    }
    fn copy_date_blocks(
        &mut self,
        source: &Self,
        mut copy: impl FnMut(&[T], &mut AllocVec<T, BaseAllocator>) -> Result<(), Failure>,
    ) -> Result<(), DynamicArrayCopyFailure<T>> {
        self.validate_copy_source(source)
            .map_err(DynamicArrayCopyFailure::domain)?;
        for source_block in &source.blocks {
            let count = source.used.saturating_sub(self.used).min(self.increment);
            let block = self.copy_unpublished_block(&source_block[..count], &mut copy)?;
            self.blocks.push(block);
            self.used += count;
        }
        Ok(())
    }
    fn copy_unpublished_block(
        &self,
        values: &[T],
        copy: &mut impl FnMut(&[T], &mut AllocVec<T, BaseAllocator>) -> Result<(), Failure>,
    ) -> Result<AllocVec<T, BaseAllocator>, DynamicArrayCopyFailure<T>> {
        let mut block =
            AllocVec::with_capacity_in(self.increment, self.allocator.clone().unwrap_or_default());
        if let Err(failure) = copy(values, &mut block) {
            return Err(DynamicArrayCopyFailure::construction(failure, block));
        }
        Ok(block)
    }
    fn validate_copy_source(&self, source: &Self) -> Result<(), Failure> {
        let valid_prefix = source.copy_source_prefix_valid();
        if self.blocks.is_empty()
            && self.used == 0
            && self.increment == source.increment
            && valid_prefix
        {
            return Ok(());
        }
        Err(self.copy_storage_failure(source, valid_prefix))
    }
    fn copy_source_prefix_valid(&self) -> bool {
        let uninitialized = self.blocks.iter().enumerate().any(|(index, block)| {
            let count = self
                .used
                .saturating_sub(index.saturating_mul(self.increment))
                .min(self.increment);
            count > block.len()
        });
        !uninitialized && self.used <= self.blocks.len().saturating_mul(self.increment)
    }
    fn copy_storage_failure(&self, source: &Self, valid_prefix: bool) -> Failure {
        let reason = format!(
            "destination blocks {}, size {}, stride {}; source size {}, stride {}, valid prefix {}: expected unallocated destination and matching valid source stride",
            self.blocks.len(),
            self.used,
            self.increment,
            source.used,
            source.increment,
            valid_prefix
        );
        Failure::with_type("Standard_DomainError", &reason, "")
    }
}
