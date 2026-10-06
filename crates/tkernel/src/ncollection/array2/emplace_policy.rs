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
