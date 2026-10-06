//! Native internal references retain their physical source slot while insertion shifts its value.
use super::{ArrayError, DynamicArray};
use std::fmt::Debug;

impl<T: Default> DynamicArray<T> {
    /// Inserts before an item, copying the internal source slot only after descending shifts.
    /// `array.insert_before_from_index_with_assignment(0, 1, std::mem::swap, |a,b| *a = *b, |_| {})?`.
    /// The source value may have changed during shifting. Equal final source/target indices invoke
    /// self_assign with one reference, preserving native self-assignment without Rust aliasing.
    pub fn insert_before_from_index_with_assignment<
        I: TryInto<usize> + Copy + Debug,
        S: TryInto<usize> + Copy + Debug,
    >(
        &mut self,
        index: I,
        source_index: S,
        shift_assign: impl FnMut(&mut T, &mut T),
        copy_assign: impl FnOnce(&mut T, &T),
        self_assign: impl FnOnce(&mut T),
    ) -> Result<&mut T, ArrayError> {
        let (target, source) = self.internal_insertion_indices(index, source_index, false)?;
        Ok(self.assign_internal_insertion(
            target,
            source,
            shift_assign,
            |a, b| copy_assign(a, b),
            self_assign,
        ))
    }
    /// Inserts after an item and copy-assigns from the original physical source slot after shifting.
    /// `array.insert_after_from_index_with_assignment(0, 1, std::mem::swap, |a,b| *a = *b, |_| {})?`.
    /// self_assign supplies the item's native copy-self-assignment behavior when the slots coincide.
    pub fn insert_after_from_index_with_assignment<
        I: TryInto<usize> + Copy + Debug,
        S: TryInto<usize> + Copy + Debug,
    >(
        &mut self,
        index: I,
        source_index: S,
        shift_assign: impl FnMut(&mut T, &mut T),
        copy_assign: impl FnOnce(&mut T, &T),
        self_assign: impl FnOnce(&mut T),
    ) -> Result<&mut T, ArrayError> {
        let (target, source) = self.internal_insertion_indices(index, source_index, true)?;
        Ok(self.assign_internal_insertion(
            target,
            source,
            shift_assign,
            |a, b| copy_assign(a, b),
            self_assign,
        ))
    }
    /// Inserts before an item, then move-assigns from the internal slot as modified by shifting.
    /// `array.insert_before_moved_from_index_with_assignment(0, 1, std::mem::swap, std::mem::swap, |_| {})?`.
    /// self_assign supplies native move-self-assignment; Rust cannot form two mutable aliases.
    pub fn insert_before_moved_from_index_with_assignment<
        I: TryInto<usize> + Copy + Debug,
        S: TryInto<usize> + Copy + Debug,
    >(
        &mut self,
        index: I,
        source_index: S,
        shift_assign: impl FnMut(&mut T, &mut T),
        move_assign: impl FnOnce(&mut T, &mut T),
        self_assign: impl FnOnce(&mut T),
    ) -> Result<&mut T, ArrayError> {
        let (target, source) = self.internal_insertion_indices(index, source_index, false)?;
        Ok(self.assign_internal_insertion(target, source, shift_assign, move_assign, self_assign))
    }
    /// Inserts after an item, keeping the internal source position fixed through descending shifts.
    /// `array.insert_after_moved_from_index_with_assignment(0, 1, std::mem::swap, std::mem::swap, |_| {})?`.
    /// All callbacks leave their arguments live on unwind; committed size and mutations persist.
    pub fn insert_after_moved_from_index_with_assignment<
        I: TryInto<usize> + Copy + Debug,
        S: TryInto<usize> + Copy + Debug,
    >(
        &mut self,
        index: I,
        source_index: S,
        shift_assign: impl FnMut(&mut T, &mut T),
        move_assign: impl FnOnce(&mut T, &mut T),
        self_assign: impl FnOnce(&mut T),
    ) -> Result<&mut T, ArrayError> {
        let (target, source) = self.internal_insertion_indices(index, source_index, true)?;
        Ok(self.assign_internal_insertion(target, source, shift_assign, move_assign, self_assign))
    }
    fn internal_insertion_indices<
        I: TryInto<usize> + Copy + Debug,
        S: TryInto<usize> + Copy + Debug,
    >(
        &self,
        index: I,
        source_index: S,
        after: bool,
    ) -> Result<(usize, usize), ArrayError> {
        let source = self.logical_source_index(source_index)?;
        let target = self.insertion_position(index, after)?;
        Ok((target, source))
    }
    fn assign_internal_insertion(
        &mut self,
        target: usize,
        source: usize,
        shift_assign: impl FnMut(&mut T, &mut T),
        final_assign: impl FnOnce(&mut T, &mut T),
        self_assign: impl FnOnce(&mut T),
    ) -> &mut T {
        self.stage_insertion(target, shift_assign);
        if target == source {
            self_assign(&mut self.blocks[target / self.increment][target % self.increment]);
        } else {
            let (target, source) = self.distinct_value_slots(target, source);
            final_assign(target, source);
        }
        &mut self.blocks[target / self.increment][target % self.increment]
    }
}
