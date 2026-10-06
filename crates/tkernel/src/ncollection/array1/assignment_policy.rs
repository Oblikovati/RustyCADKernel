//! Native assignment borrows its donor; callbacks preserve independent type operations.
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
        assign_donor(self.change_value(index), donor, assign)
    }
}

pub(in crate::ncollection) fn assign_donor<T, Donor>(
    target: Result<&mut T, ArrayError>,
    donor: Donor,
    assign: impl FnOnce(&mut T, Donor),
) -> Result<(), ArrayError> {
    // The C++ rvalue reference does not imply destroying or replacing the donor.
    assign(target?, donor);
    Ok(())
}

impl<T> Array1<'_, T> {
    /// Assigns a borrowed value without requiring its type to support copy construction.
    /// The donor remains borrowed across success or callback panic.
    /// `array.set_value_ref_with_assignment(0, &42, |slot, donor| *slot = *donor)?`.
    pub fn set_value_ref_with_assignment(
        &mut self,
        index: i32,
        donor: &T,
        assign: impl FnOnce(&mut T, &T),
    ) -> Result<(), ArrayError> {
        assign_donor(self.change_value(index), donor, assign)
    }

    /// Assigns one borrowed value to every slot in storage order without constructing copies.
    /// A callback panic retains earlier assignments and any mutation to the failing slot.
    /// `array.init_ref_with_assignment(&42, |slot, donor| *slot = *donor)`.
    pub fn init_ref_with_assignment(&mut self, donor: &T, mut assign: impl FnMut(&mut T, &T)) {
        for slot in self.as_mut_slice() {
            assign(slot, donor);
        }
    }
}
