//! Public copy construction and assignment select native element-copy policies explicitly.
use super::{DynamicArray, DynamicArrayCopyConstructionFailure, DynamicArrayCopyFailure};
use crate::standard::Failure;
impl<T> DynamicArray<T> {
    /// Copies using an explicit fallible native copy constructor, without requiring Clone.
    /// `let copied = DynamicArray::from_copied_with(&source, |value| Ok(*value))?;`.
    /// Whole blocks publish only after their constructors succeed. On Err the returned owner keeps
    /// both published values and the incomplete block alive until explicit Rust recovery or Drop.
    pub fn from_copied_with(
        source: &Self,
        construct: impl FnMut(&T) -> Result<T, Failure>,
    ) -> Result<Self, DynamicArrayCopyConstructionFailure<T>> {
        Self::construct_copy_policy(source, |target, source| {
            target.copy_date_using(source, construct)
        })
    }
    /// Clears the receiver, selects native allocator ownership, then copy-constructs source blocks.
    /// `target.assign_with_constructor(&source, true, |value| Ok(*value))?;`.
    /// Err leaves complete blocks published and returns ownership of the incomplete block. Report
    /// native constructor throws as Err; Rust callback panics use RAII and are a separate domain.
    pub fn assign_with_constructor(
        &mut self,
        source: &Self,
        own_allocator: bool,
        construct: impl FnMut(&T) -> Result<T, Failure>,
    ) -> Result<&mut Self, DynamicArrayCopyFailure<T>> {
        self.assign_copy_policy(source, own_allocator, |target, source| {
            target.copy_date_using(source, construct)
        })
    }
    /// Uses copy operator='s source-allocator adoption with an explicit element constructor.
    /// `target.assign_operator_with_constructor(&source, |value| Ok(*value))?;`.
    pub fn assign_operator_with_constructor(
        &mut self,
        source: &Self,
        construct: impl FnMut(&T) -> Result<T, Failure>,
    ) -> Result<&mut Self, DynamicArrayCopyFailure<T>> {
        self.assign_with_constructor(source, false, construct)
    }
    fn construct_copy_policy(
        source: &Self,
        copy: impl FnOnce(&mut Self, &Self) -> Result<(), DynamicArrayCopyFailure<T>>,
    ) -> Result<Self, DynamicArrayCopyConstructionFailure<T>> {
        let mut target = Self::with_rounded_increment(source.increment, source.allocator.clone());
        if let Err(incomplete) = copy(&mut target, source) {
            return Err(DynamicArrayCopyConstructionFailure::new(target, incomplete));
        }
        Ok(target)
    }
    fn assign_copy_policy(
        &mut self,
        source: &Self,
        own_allocator: bool,
        copy: impl FnOnce(&mut Self, &Self) -> Result<(), DynamicArrayCopyFailure<T>>,
    ) -> Result<&mut Self, DynamicArrayCopyFailure<T>> {
        self.clear(true);
        self.increment = source.increment;
        if !own_allocator {
            self.allocator = source.allocator.clone();
        }
        copy(self, source)?;
        Ok(self)
    }
}
impl<T: Copy> DynamicArray<T> {
    /// Selects native trivially-copyable construction, bypassing observable Clone implementations.
    /// `let copied = DynamicArray::from_copied_trivial(&source)?;`.
    pub fn from_copied_trivial(
        source: &Self,
    ) -> Result<Self, DynamicArrayCopyConstructionFailure<T>> {
        Self::construct_copy_policy(source, Self::copy_date_bitwise)
    }
    /// Selects native trivially-copyable assignment after clearing and allocator selection.
    /// `target.assign_trivial(&source, false)?;` selects copy operator='s allocator policy.
    pub fn assign_trivial(
        &mut self,
        source: &Self,
        own_allocator: bool,
    ) -> Result<&mut Self, DynamicArrayCopyFailure<T>> {
        self.assign_copy_policy(source, own_allocator, Self::copy_date_bitwise)
    }
}
