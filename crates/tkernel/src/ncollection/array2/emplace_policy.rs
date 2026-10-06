//! Share Array1's native temporary and assignment lifetime for a matrix cell.
use super::Array2;
use crate::ncollection::{ArrayError, array1::emplace_policy::assign_emplaced};

impl<T> Array2<'_, T> {
    /// Constructs, assigns and destroys a temporary while retaining the target object.
    /// Row/column bounds precede construction; inherited storage checks follow it.
    /// `array.emplace_value_with_assignment(0, 0, || 42, |slot, donor| *slot = *donor)?`.
    pub fn emplace_value_with_assignment(
        &mut self,
        row: i32,
        col: i32,
        construct: impl FnOnce() -> T,
        assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        let (row_offset, col_offset) = self.bounded_offsets(row, col)?;
        let offset = self.offset(row_offset, col_offset)?;
        assign_emplaced(construct, || self.elements.change_at(offset), assign)
    }
}

impl<T> Array2<'_, T> {
    /// Constructs a distinct temporary from a live internal argument, then assigns it.
    /// Source validation precedes target validation; constructor mutations survive panic.
    /// `array.emplace_value_from_index_with_assignment((0, 0), (0, 1), |s| *s, |t, s| *t = *s)?`.
    pub fn emplace_value_from_index_with_assignment(
        &mut self,
        target: (i32, i32),
        source: (i32, i32),
        construct: impl FnOnce(&T) -> T,
        assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        let (row, col) = self.bounded_offsets(source.0, source.1)?;
        let source = self.offset(row, col)?;
        self.elements.at(source)?;
        let (row, col) = self.bounded_offsets(target.0, target.1)?;
        let target = self.offset(row, col)?;
        let temporary = construct(self.elements.at(source)?);
        assign_emplaced(|| temporary, || self.elements.change_at(target), assign)
    }

    /// Constructs a distinct temporary from a live internal argument, then assigns it.
    /// Source validation precedes target validation; constructor mutations survive panic.
    /// `array.emplace_value_moved_from_index_with_assignment((0, 0), (0, 1), |s| *s, |t, s| *t = *s)?`.
    pub fn emplace_value_moved_from_index_with_assignment(
        &mut self,
        target: (i32, i32),
        source: (i32, i32),
        construct: impl FnOnce(&mut T) -> T,
        assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        let (row, col) = self.bounded_offsets(source.0, source.1)?;
        let source = self.offset(row, col)?;
        self.elements.at(source)?;
        let (row, col) = self.bounded_offsets(target.0, target.1)?;
        let target = self.offset(row, col)?;
        let temporary = construct(self.elements.change_at(source)?);
        assign_emplaced(|| temporary, || self.elements.change_at(target), assign)
    }
}
