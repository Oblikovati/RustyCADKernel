//! OCCT 3d097a0 commits the appended default before any shifting assignment.
use super::{ArrayError, DynamicArray, checked_index};
use std::fmt::Debug;

impl<T: Default> DynamicArray<T> {
    /// Inserts using Rust value moves after native default staging; `array.insert_before(1, 42)?`.
    /// Use `insert_before_with_assignment` for observable custom C++ assignment operations.
    pub fn insert_before<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        value: T,
    ) -> Result<&mut T, ArrayError> {
        self.insert_before_with_assignment(index, std::mem::swap, |slot| *slot = value)
    }
    /// Inserts after an existing item; `array.insert_after(1, 42)?`.
    /// Default construction precedes Rust value moves; custom assignments use the hook adapter.
    pub fn insert_after<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        value: T,
    ) -> Result<&mut T, ArrayError> {
        self.insert_after_with_assignment(index, std::mem::swap, |slot| *slot = value)
    }
    /// Runs native descending shift assignments then the supplied final assignment.
    /// `array.insert_before_with_assignment(1, std::mem::swap, |slot| *slot = 42)?`.
    /// A panic preserves the committed size and completed assignments, matching OCCT.
    pub fn insert_before_with_assignment<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        shift_assignment: impl FnMut(&mut T, &mut T),
        final_assignment: impl FnOnce(&mut T),
    ) -> Result<&mut T, ArrayError> {
        let index = self.insertion_position(index, false)?;
        Ok(self.assign_insertion(index, shift_assignment, final_assignment))
    }
    /// Assigns shifts from high to low, then assigns the new value after the selected item.
    /// `array.insert_after_with_assignment(0, std::mem::swap, |slot| *slot = 42)?`.
    /// Each callback receives distinct initialized slots; no raw pointers are required.
    pub fn insert_after_with_assignment<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        shift_assignment: impl FnMut(&mut T, &mut T),
        final_assignment: impl FnOnce(&mut T),
    ) -> Result<&mut T, ArrayError> {
        let index = self.insertion_position(index, true)?;
        Ok(self.assign_insertion(index, shift_assignment, final_assignment))
    }
    pub(super) fn insertion_position<I: TryInto<usize> + Copy + Debug>(
        &self,
        index: I,
        after: bool,
    ) -> Result<usize, ArrayError> {
        let index = checked_index(index)?;
        if index >= self.used {
            return Err(self.index_error(index));
        }
        Ok(index + usize::from(after))
    }
    fn assign_insertion(
        &mut self,
        index: usize,
        shift_assignment: impl FnMut(&mut T, &mut T),
        final_assignment: impl FnOnce(&mut T),
    ) -> &mut T {
        self.stage_insertion(index, shift_assignment);
        let target = &mut self.blocks[index / self.increment][index % self.increment];
        final_assignment(target);
        target
    }
    pub(super) fn stage_insertion(
        &mut self,
        index: usize,
        mut shift_assignment: impl FnMut(&mut T, &mut T),
    ) {
        self.appended();
        for destination in (index + 1..self.used).rev() {
            let (target, source) = self.distinct_value_slots(destination, destination - 1);
            shift_assignment(target, source);
        }
    }
}
impl<T: Clone + Default> DynamicArray<T> {
    /// Copies into the staged insertion slot using `Clone::clone_from`.
    /// `array.insert_before_ref(1, &value)?`; ordinary shifts use Rust moves.
    pub fn insert_before_ref<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        value: &T,
    ) -> Result<&mut T, ArrayError> {
        self.insert_before_ref_with_assignment(index, value, std::mem::swap, T::clone_from)
    }
    /// Copies after the selected item once native default and shift staging have completed.
    /// `array.insert_after_ref(1, &value)?`; custom move assignment uses the hook adapter.
    pub fn insert_after_ref<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        value: &T,
    ) -> Result<&mut T, ArrayError> {
        self.insert_after_ref_with_assignment(index, value, std::mem::swap, T::clone_from)
    }
}
