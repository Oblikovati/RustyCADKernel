//! CopyValues needs assignment only; it must not require element copy construction.
use super::Array1;
use crate::ncollection::ArrayError;

impl<T> Array1<'_, T> {
    /// Assigns equal-length values in order and returns this receiver without rebasing it.
    /// Callback panic retains completed assignments and mutations to the failing slot.
    /// `target.copy_values_with_assignment(&source, |slot, donor| *slot = *donor)?`.
    pub fn copy_values_with_assignment(
        &mut self,
        source: &Array1<'_, T>,
        mut assign: impl FnMut(&mut T, &T),
    ) -> Result<&mut Self, ArrayError> {
        self.copy_values_with_storage_assignment(source, |targets, donors| {
            for (slot, donor) in targets.iter_mut().zip(donors) {
                assign(slot, donor);
            }
        })
    }

    pub(super) fn copy_values_with_storage_assignment(
        &mut self,
        source: &Array1<'_, T>,
        assign: impl FnOnce(&mut [T], &[T]),
    ) -> Result<&mut Self, ArrayError> {
        if self.len() != source.len() {
            return Err(ArrayError::DimensionMismatch {
                expected: self.len(),
                actual: source.len(),
            });
        }
        // Retain clone_from_slice specialization in the existing Clone convenience policy.
        assign(self.as_mut_slice(), source.as_slice());
        Ok(self)
    }
}
