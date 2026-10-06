//! Preserve a native Init donor inside the initialized range without aliased Rust references.
use super::Array1;
use crate::ncollection::ArrayError;

impl<T> Array1<'_, T> {
    /// Initializes from one of this array's elements, including its native self-assignment step.
    /// Earlier assignments read the original donor; later ones observe self-callback mutations.
    /// `array.init_from_index_with_assignment(0, |slot, donor| *slot = *donor, |_| {})?`.
    pub fn init_from_index_with_assignment(
        &mut self,
        index: i32,
        assign: impl FnMut(&mut T, &T),
        assign_self: impl FnOnce(&mut T),
    ) -> Result<(), ArrayError> {
        let offset = self.offset(index)?;
        self.init_from_offset_with_assignment(offset, assign, assign_self)
    }

    pub(in crate::ncollection) fn init_from_offset_with_assignment(
        &mut self,
        offset: usize,
        mut assign: impl FnMut(&mut T, &T),
        assign_self: impl FnOnce(&mut T),
    ) -> Result<(), ArrayError> {
        self.at(offset)?;
        let (before, remaining) = self.as_mut_slice().split_at_mut(offset);
        // The checked source exists; splitting keeps all target and donor borrows disjoint.
        let (donor, after) = remaining.split_first_mut().unwrap();
        assign_range(before, donor, &mut assign);
        assign_self(donor);
        assign_range(after, donor, &mut assign);
        Ok(())
    }
}

fn assign_range<T>(targets: &mut [T], donor: &T, assign: &mut impl FnMut(&mut T, &T)) {
    for target in targets {
        assign(target, donor);
    }
}
