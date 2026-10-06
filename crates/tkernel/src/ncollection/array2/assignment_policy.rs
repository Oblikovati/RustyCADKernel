//! Matrix SetValue shares the caller-owned donor assignment policy with Array1.
use super::Array2;
use crate::ncollection::{ArrayError, array1::assignment_policy::assign_mutable_donor};

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
        assign_mutable_donor(self.change_value(row, col), donor, assign)
    }
}
