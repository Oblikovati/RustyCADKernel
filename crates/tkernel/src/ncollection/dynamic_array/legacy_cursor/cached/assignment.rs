//! Assignment retains snapshots; legacy initialization deliberately captures a new owner and end.
use super::{DynamicArray, DynamicArrayCachedIterator, Failure};
impl<T, const M: bool> DynamicArrayCachedIterator<T, M> {
    /// Copies owner, index, end and actual cached pointers without inspecting either allocation.
    /// `cursor.assign_from(&other);` requires no Clone bound on the payload.
    pub fn assign_from(&mut self, other: &Self) -> &mut Self {
        *self = *other;
        self
    }
    /// Represents native self assignment without creating overlapping Rust references.
    /// `cursor.assign_self();` preserves every saved field and returns this cursor.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}
impl<T> DynamicArrayCachedIterator<T, false> {
    /// Copies a mutable iterator's saved fields into this read-only iterator without reloading.
    /// `constant.assign_from_mutable(&mutable);` does not grant mutable pointer access.
    pub fn assign_from_mutable(
        &mut self,
        other: &DynamicArrayCachedIterator<T, true>,
    ) -> &mut Self {
        *self = other.as_const_cursor();
        self
    }
    /// Recaptures the supplied array at index zero with its current end, as Init/Initialize.
    /// `cursor.initialize_with(&array)?;` may replace a different original owner.
    /// Invalid live-slot layouts are checked Rust errors that preserve the previous cursor.
    pub fn initialize_with(&mut self, owner: &DynamicArray<T>) -> Result<(), Failure> {
        *self = Self::from_shared(owner, 0)?;
        Ok(())
    }
}
impl<T> DynamicArrayCachedIterator<T, true> {
    /// Recaptures writable provenance and the current end, as mutable legacy Init/Initialize.
    /// `cursor.initialize_with(&mut array)?;` accepts a different original owner.
    /// Native const-cast access is restricted to an exclusive Rust owner borrow.
    pub fn initialize_with(&mut self, owner: &mut DynamicArray<T>) -> Result<(), Failure> {
        *self = Self::from_mutable(owner, 0)?;
        Ok(())
    }
}
