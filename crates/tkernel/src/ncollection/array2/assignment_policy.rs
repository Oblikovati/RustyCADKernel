//! Matrix SetValue shares the caller-owned donor assignment policy with Array1.
use super::Array2;
use crate::ncollection::{ArrayError, array1::assignment_policy::assign_donor};

impl<T> Array2<'_, T> {
    /// Assigns into the existing cell without consuming or destroying the mutable donor.
    /// Bounds failure skips assignment; panic preserves callback mutations to both objects.
    /// `array.set_value_with_assignment(0, 0, &mut donor, std::mem::swap)?`.
    pub fn set_value_with_assignment(
        &mut self,
        row: i32,
        col: i32,
        donor: &mut T,
        assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<(), ArrayError> {
        assign_donor(self.change_value(row, col), donor, assign)
    }
}

impl<T> Array2<'_, T> {
    /// Assigns a borrowed donor to an existing cell without requiring copy construction.
    /// `array.set_value_ref_with_assignment(0, 0, &42, |slot, donor| *slot = *donor)?`.
    pub fn set_value_ref_with_assignment(
        &mut self,
        row: i32,
        col: i32,
        donor: &T,
        assign: impl FnOnce(&mut T, &T),
    ) -> Result<(), ArrayError> {
        assign_donor(self.change_value(row, col), donor, assign)
    }

    /// Assigns the inherited flat range, including independently changed Array1 storage.
    /// `array.init_ref_with_assignment(&42, |slot, donor| *slot = *donor)`.
    pub fn init_ref_with_assignment(&mut self, donor: &T, assign: impl FnMut(&mut T, &T)) {
        self.elements.init_ref_with_assignment(donor, assign);
    }
}

impl<T> Array2<'_, T> {
    /// Assigns from an internal const donor with a separate same-cell callback.
    /// Source row/column validation precedes target validation.
    /// `array.set_value_from_index_with_assignment((0, 0), (0, 1), |t, s| *t = *s, |_| {})?`.
    pub fn set_value_from_index_with_assignment(
        &mut self,
        target: (i32, i32),
        source: (i32, i32),
        assign: impl FnOnce(&mut T, &T),
        assign_self: impl FnOnce(&mut T),
    ) -> Result<(), ArrayError> {
        self.set_value_moved_from_index_with_assignment(
            target,
            source,
            |t, s| assign(t, s),
            assign_self,
        )
    }

    /// Assigns directly from a mutable internal donor while retaining both cell lifetimes.
    /// Self movement dispatches separately; source indices precede target indices.
    /// `array.set_value_moved_from_index_with_assignment((0, 0), (0, 1), std::mem::swap, |_| {})?`.
    pub fn set_value_moved_from_index_with_assignment(
        &mut self,
        target: (i32, i32),
        source: (i32, i32),
        assign: impl FnOnce(&mut T, &mut T),
        assign_self: impl FnOnce(&mut T),
    ) -> Result<(), ArrayError> {
        let (source_row, source_col) = self.bounded_offsets(source.0, source.1)?;
        let (target_row, target_col) = self.bounded_offsets(target.0, target.1)?;
        let source = self.offset(source_row, source_col)?;
        let target = self.offset(target_row, target_col)?;
        self.elements
            .set_value_from_offsets_with_assignment(target, source, assign, assign_self)
    }
}
