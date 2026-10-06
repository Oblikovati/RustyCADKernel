//! Explicit borrowed C++ operations, independent of Rust Clone's assignment default.
use super::{ArrayError, DynamicArray};
use std::fmt::Debug;

impl<T> DynamicArray<T> {
    /// Reserves the append block before copying the borrowed source; no Clone bound is needed.
    /// `array.append_ref_with_constructor(&7, |source| *source)`.
    pub fn append_ref_with_constructor(
        &mut self,
        source: &T,
        copy_construct: impl FnOnce(&T) -> T,
    ) -> &mut T {
        self.emplace_append(|| copy_construct(source))
    }
}

impl<T: Default> DynamicArray<T> {
    /// Copies new slots; existing slots copy a temporary, move-assign it, then drop its donor.
    /// `array.set_value_ref_with_assignment(0, &7, |v| *v, |to, from| *to = *from)?`.
    /// This selects OCCT's assignable replacement branch explicitly. Assignment must leave
    /// both arguments valid on unwind; Rust Clone alone does not specify C++ assignment.
    pub fn set_value_ref_with_assignment<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        source: &T,
        copy_construct: impl FnOnce(&T) -> T,
        move_assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        self.emplace_value_with_assignment(index, || copy_construct(source), move_assign)
    }

    /// Defaults the tail, move-assigns descending shifts, then copy-assigns the borrowed source.
    /// `array.insert_before_ref_with_assignment(0, &7, std::mem::swap, |a, b| *a = *b)?`.
    /// Hooks model separate C++ assignment operations without requiring Clone construction.
    pub fn insert_before_ref_with_assignment<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        source: &T,
        move_assign: impl FnMut(&mut T, &mut T),
        copy_assign: impl FnOnce(&mut T, &T),
    ) -> Result<&mut T, ArrayError> {
        self.insert_before_with_assignment(index, move_assign, |target| copy_assign(target, source))
    }

    /// Inserts after the index with explicit move-shift and borrowed copy-assignment hooks.
    /// `array.insert_after_ref_with_assignment(0, &7, std::mem::swap, |a, b| *a = *b)?`.
    /// Size commits before assignment callbacks, including callbacks that unwind.
    pub fn insert_after_ref_with_assignment<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        source: &T,
        move_assign: impl FnMut(&mut T, &mut T),
        copy_assign: impl FnOnce(&mut T, &T),
    ) -> Result<&mut T, ArrayError> {
        self.insert_after_with_assignment(index, move_assign, |target| copy_assign(target, source))
    }
}
