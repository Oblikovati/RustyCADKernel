//! Mutable donors expose C++ move construction and assignment separately from Rust value moves.
use super::{ArrayError, DynamicArray};
use std::fmt::Debug;

impl<T> DynamicArray<T> {
    /// Allocates before move-constructing from a donor that remains observable after this call.
    /// `array.append_moved_with_constructor(&mut value, std::mem::take)`.
    /// The constructor must leave its donor valid on unwind; no Clone or Default bound is imposed.
    pub fn append_moved_with_constructor(
        &mut self,
        donor: &mut T,
        move_construct: impl FnOnce(&mut T) -> T,
    ) -> &mut T {
        self.emplace_append(|| move_construct(donor))
    }
}
impl<T: Default> DynamicArray<T> {
    /// Move-constructs new slots; existing slots move-construct then assign from a temporary.
    /// `array.set_value_moved_with_assignment(0, &mut value, std::mem::take, std::mem::swap)?`.
    /// This explicitly selects native's assignable replacement branch. Donors stay valid on unwind.
    pub fn set_value_moved_with_assignment<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        donor: &mut T,
        move_construct: impl FnOnce(&mut T) -> T,
        move_assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        self.emplace_value_with_assignment(index, || move_construct(donor), move_assign)
    }
    /// Defaults the tail, assigns descending shifts, then move-assigns from the external donor.
    /// `array.insert_before_moved_with_assignment(0, &mut value, std::mem::swap, std::mem::swap)?`.
    /// Both callbacks represent the item's native move assignment and must leave slots valid.
    pub fn insert_before_moved_with_assignment<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        donor: &mut T,
        shift_assign: impl FnMut(&mut T, &mut T),
        final_assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        self.insert_before_with_assignment(index, shift_assign, |target| {
            final_assign(target, donor)
        })
    }
    /// Inserts after the index, preserving observable donor mutation even when assignment panics.
    /// `array.insert_after_moved_with_assignment(0, &mut value, std::mem::swap, std::mem::swap)?`.
    /// Callbacks select native move assignment explicitly; no constructor runs for the donor.
    pub fn insert_after_moved_with_assignment<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        donor: &mut T,
        shift_assign: impl FnMut(&mut T, &mut T),
        final_assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        self.insert_after_with_assignment(index, shift_assign, |target| final_assign(target, donor))
    }
}
