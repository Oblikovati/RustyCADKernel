//! Native SetValue(T&&) borrows its donor; assignment never owns its lifetime.
use super::Array1;
use crate::ncollection::ArrayError;

impl<T> Array1<'_, T> {
    /// Assigns into the existing object while retaining the caller's donor lifetime.
    /// Bounds failure skips assignment; panic preserves callback mutations to both objects.
    /// `array.set_value_with_assignment(0, &mut donor, std::mem::swap)?`.
    pub fn set_value_with_assignment(
        &mut self,
        index: i32,
        donor: &mut T,
        assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<(), ArrayError> {
        assign_mutable_donor(self.change_value(index), donor, assign)
    }
}

pub(in crate::ncollection) fn assign_mutable_donor<T>(
    target: Result<&mut T, ArrayError>,
    donor: &mut T,
    assign: impl FnOnce(&mut T, &mut T),
) -> Result<(), ArrayError> {
    // The C++ rvalue reference does not imply destroying or replacing the donor.
    assign(target?, donor);
    Ok(())
}
