//! Native EmplaceValue assigns a constructed temporary into the existing object.
use super::Array1;
use crate::ncollection::ArrayError;

impl<T> Array1<'_, T> {
    /// Constructs, assigns and destroys a temporary while retaining the target object.
    /// Bounds are checked before construction; assignment panic keeps target mutations.
    /// `array.emplace_value_with_assignment(0, || 42, |slot, donor| *slot = *donor)?`.
    pub fn emplace_value_with_assignment(
        &mut self,
        index: i32,
        construct: impl FnOnce() -> T,
        assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        let offset = self.offset(index)?;
        assign_emplaced(construct, || self.change_at(offset), assign)
    }
}

pub(in crate::ncollection) fn assign_emplaced<'a, T>(
    construct: impl FnOnce() -> T,
    target: impl FnOnce() -> Result<&'a mut T, ArrayError>,
    assign: impl FnOnce(&mut T, &mut T),
) -> Result<&'a mut T, ArrayError> {
    // C++17 evaluates the assignment's RHS before its LHS. Array2's inherited
    // ChangeAt can reject an independently shortened buffer after construction.
    let mut temporary = construct();
    let slot = target()?;
    assign(slot, &mut temporary);
    drop(temporary);
    Ok(slot)
}

impl<T> Array1<'_, T> {
    /// Constructs a distinct temporary from a live internal argument, then assigns it.
    /// Source validation precedes target validation; constructor mutations survive panic.
    /// `array.emplace_value_from_index_with_assignment(0, 1, |s| *s, |t, s| *t = *s)?`.
    pub fn emplace_value_from_index_with_assignment(
        &mut self,
        target: i32,
        source: i32,
        construct: impl FnOnce(&T) -> T,
        assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        let source = self.offset(source)?;
        let target = self.offset(target)?;
        let temporary = construct(self.at(source)?);
        assign_emplaced(|| temporary, || self.change_at(target), assign)
    }

    /// Constructs a distinct temporary from a live internal argument, then assigns it.
    /// Source validation precedes target validation; constructor mutations survive panic.
    /// `array.emplace_value_moved_from_index_with_assignment(0, 1, |s| *s, |t, s| *t = *s)?`.
    pub fn emplace_value_moved_from_index_with_assignment(
        &mut self,
        target: i32,
        source: i32,
        construct: impl FnOnce(&mut T) -> T,
        assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        let source = self.offset(source)?;
        let target = self.offset(target)?;
        let temporary = construct(self.change_at(source)?);
        assign_emplaced(|| temporary, || self.change_at(target), assign)
    }
}
