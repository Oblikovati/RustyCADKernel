//! Matrix CopyValues checks rectangular dimensions before inherited flat lengths.
use super::Array2;
use crate::ncollection::ArrayError;

impl<T> Array2<'_, T> {
    /// Assigns equally shaped values in order without requiring copy construction.
    /// Preserves bounds and returns this receiver; failures retain partial assignments.
    /// `target.copy_values_with_assignment(&source, |slot, donor| *slot = *donor)?`.
    pub fn copy_values_with_assignment(
        &mut self,
        source: &Array2<'_, T>,
        assign: impl FnMut(&mut T, &T),
    ) -> Result<&mut Self, ArrayError> {
        self.check_rectangle(source)?;
        self.elements
            .copy_values_with_assignment(&source.elements, assign)?;
        Ok(self)
    }
}
