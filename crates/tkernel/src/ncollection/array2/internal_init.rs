//! Native matrix Init may receive one of its own cells as the const donor.
use super::Array2;
use crate::ncollection::ArrayError;

impl<T> Array2<'_, T> {
    /// Initializes the flat range from a cell, applying a distinct self-assignment callback.
    /// Donor mutations at its own position are visible to all later assignments.
    /// `array.init_from_index_with_assignment(0, 0, |slot, donor| *slot = *donor, |_| {})?`.
    pub fn init_from_index_with_assignment(
        &mut self,
        row: i32,
        col: i32,
        assign: impl FnMut(&mut T, &T),
        assign_self: impl FnOnce(&mut T),
    ) -> Result<(), ArrayError> {
        let (row, col) = self.bounded_offsets(row, col)?;
        let offset = self.offset(row, col)?;
        self.elements
            .init_from_offset_with_assignment(offset, assign, assign_self)
    }
}
