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

impl<T> Array1<'_, T> {
    /// Assigns from a live internal const donor, dispatching same-element assignment separately.
    /// Source bounds are checked before target bounds, as in `SetValue(target, Value(source))`.
    /// `array.set_value_from_index_with_assignment(0, 1, |t, s| *t = *s, |_| {})?`.
    pub fn set_value_from_index_with_assignment(
        &mut self,
        index: i32,
        source_index: i32,
        assign: impl FnOnce(&mut T, &T),
        assign_self: impl FnOnce(&mut T),
    ) -> Result<(), ArrayError> {
        self.set_value_moved_from_index_with_assignment(
            index,
            source_index,
            |t, s| assign(t, s),
            assign_self,
        )
    }

    /// Assigns from a live internal mutable donor without constructing a temporary.
    /// Self movement has its own callback; source bounds precede target bounds.
    /// `array.set_value_moved_from_index_with_assignment(0, 1, std::mem::swap, |_| {})?`.
    pub fn set_value_moved_from_index_with_assignment(
        &mut self,
        index: i32,
        source_index: i32,
        assign: impl FnOnce(&mut T, &mut T),
        assign_self: impl FnOnce(&mut T),
    ) -> Result<(), ArrayError> {
        let source = self.offset(source_index)?;
        let target = self.offset(index)?;
        self.set_value_from_offsets_with_assignment(target, source, assign, assign_self)
    }

    pub(in crate::ncollection) fn set_value_from_offsets_with_assignment(
        &mut self,
        target: usize,
        source: usize,
        assign: impl FnOnce(&mut T, &mut T),
        assign_self: impl FnOnce(&mut T),
    ) -> Result<(), ArrayError> {
        self.at(source)?;
        self.at(target)?;
        if target == source {
            assign_self(self.change_at(target)?);
            return Ok(());
        }
        let (slot, donor) = disjoint_assignment_slots(self.as_mut_slice(), target, source);
        assign_donor(Ok(slot), donor, assign)
    }
}

fn disjoint_assignment_slots<T>(
    values: &mut [T],
    target: usize,
    source: usize,
) -> (&mut T, &mut T) {
    if target < source {
        let (before, after) = values.split_at_mut(source);
        return (&mut before[target], &mut after[0]);
    }
    let (before, after) = values.split_at_mut(target);
    (&mut after[0], &mut before[source])
}
