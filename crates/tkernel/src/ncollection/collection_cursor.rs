// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! NCollection_Iterator: a More/Next cursor over a container's `[begin, end)` positions.
//!
//! Its only native owner is NCollection_Array1 (`Iterator`/`ConstIterator`), whose iterator
//! type is a position in contiguous storage. The Rust cursor therefore walks a slice (such
//! as `Array1::as_slice`) and exposes positions as offsets. `CollectionCursor` is the
//! `IsConstant = true` form; `CollectionCursorMut` the mutable default form. Both deref to
//! `CursorPositions`, which holds the shared More/Next/position protocol.
use super::ArrayError;

/// The `[current, last)` position pair both cursor forms share (native `myCur`/`myLast`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CursorPositions {
    current: usize,
    last: usize,
}

impl CursorPositions {
    /// True while the current position differs from the end; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.current != self.last
    }

    /// Moves to the next position without a bound check, like `++myCur`; `cursor.next()`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, not Iterator::next
    pub fn next(&mut self) {
        self.current += 1;
    }

    /// Current position offset (native `ValueIter`); `cursor.value_iter()`.
    pub fn value_iter(&self) -> usize {
        self.current
    }

    /// Mutable current position (native `ChangeValueIter`); `*cursor.change_value_iter() = 0`.
    pub fn change_value_iter(&mut self) -> &mut usize {
        &mut self.current
    }

    /// End position offset (native `EndIter`); `cursor.end_iter()`.
    pub fn end_iter(&self) -> usize {
        self.last
    }

    /// Mutable end position (native `ChangeEndIter`); `*cursor.change_end_iter() = 1`.
    pub fn change_end_iter(&mut self) -> &mut usize {
        &mut self.last
    }
}

/// Read-only cursor (`NCollection_Iterator<C, true>`); `CollectionCursor::new(&[1, 2])`.
///
/// ```
/// use rustycad_tkernel::ncollection::CollectionCursor;
/// let mut cursor = CollectionCursor::new(&[10, 20]);
/// let mut seen = Vec::new();
/// while cursor.more() {
///     seen.push(*cursor.value().unwrap());
///     cursor.next();
/// }
/// assert_eq!(seen, [10, 20]);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct CollectionCursor<'a, T> {
    items: &'a [T],
    positions: CursorPositions,
}

/// Mutable cursor (`NCollection_Iterator<C, false>`); `CollectionCursorMut::new(&mut items)`.
#[derive(Debug, Default)]
pub struct CollectionCursorMut<'a, T> {
    items: &'a mut [T],
    positions: CursorPositions,
}

/// Positions of a fresh cursor starting at `position` of a container of `len` items.
fn starting_at(position: usize, len: usize) -> CursorPositions {
    CursorPositions {
        current: position,
        last: len,
    }
}

/// The checked dereference: C++ dereferences without a check (undefined past the end).
fn item_index(len: usize, position: usize) -> Result<usize, ArrayError> {
    if position < len {
        return Ok(position);
    }
    Err(ArrayError::Index {
        index: position as i64,
        lower: 0,
        upper: len as i64 - 1,
    })
}

impl<'a, T> CollectionCursor<'a, T> {
    /// Cursor over `[begin, end)` of `items`; `CollectionCursor::new(array.as_slice())`.
    pub fn new(items: &'a [T]) -> Self {
        Self::with_position(items, 0)
    }

    /// Cursor starting at `position`, ending at the container end; `with_position(s, 1)`.
    pub fn with_position(items: &'a [T], position: usize) -> Self {
        let positions = starting_at(position, items.len());
        Self { items, positions }
    }

    /// Restarts on `items` (native `Init`/`Initialize`); `cursor.init(other)`.
    pub fn init(&mut self, items: &'a [T]) {
        *self = Self::new(items);
    }

    /// The current item; an index error past the storage; `cursor.value()?`.
    pub fn value(&self) -> Result<&'a T, ArrayError> {
        Ok(&self.items[item_index(self.items.len(), self.positions.current)?])
    }
}

impl<'a, T> CollectionCursorMut<'a, T> {
    /// Cursor over `[begin, end)` of mutable `items`; `CollectionCursorMut::new(slice)`.
    pub fn new(items: &'a mut [T]) -> Self {
        Self::with_position(items, 0)
    }

    /// Cursor starting at `position`, ending at the container end.
    pub fn with_position(items: &'a mut [T], position: usize) -> Self {
        let positions = starting_at(position, items.len());
        Self { items, positions }
    }

    /// Restarts on `items` (native `Init`/`Initialize`); `cursor.init(other)`.
    pub fn init(&mut self, items: &'a mut [T]) {
        *self = Self::new(items);
    }

    /// The current item, read-only; `cursor.value()?`.
    pub fn value(&self) -> Result<&T, ArrayError> {
        Ok(&self.items[item_index(self.items.len(), self.positions.current)?])
    }

    /// The current item, writable (native `ChangeValue`); `*cursor.change_value()? = 5`.
    pub fn change_value(&mut self) -> Result<&mut T, ArrayError> {
        let index = item_index(self.items.len(), self.positions.current)?;
        Ok(&mut self.items[index])
    }

    /// A read-only cursor at the same positions (native converting constructor/assignment).
    pub fn as_const(&self) -> CollectionCursor<'_, T> {
        CollectionCursor {
            items: self.items,
            positions: self.positions,
        }
    }
}

impl<T> std::ops::Deref for CollectionCursor<'_, T> {
    type Target = CursorPositions;
    fn deref(&self) -> &CursorPositions {
        &self.positions
    }
}

impl<T> std::ops::DerefMut for CollectionCursor<'_, T> {
    fn deref_mut(&mut self) -> &mut CursorPositions {
        &mut self.positions
    }
}

impl<T> std::ops::Deref for CollectionCursorMut<'_, T> {
    type Target = CursorPositions;
    fn deref(&self) -> &CursorPositions {
        &self.positions
    }
}

impl<T> std::ops::DerefMut for CollectionCursorMut<'_, T> {
    fn deref_mut(&mut self) -> &mut CursorPositions {
        &mut self.positions
    }
}

/// Native `operator==`: same storage, same end and same current position.
impl<T> PartialEq for CollectionCursor<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.items.as_ptr(), other.items.as_ptr()) && self.positions == other.positions
    }
}

impl<T> PartialEq<CollectionCursorMut<'_, T>> for CollectionCursor<'_, T> {
    fn eq(&self, other: &CollectionCursorMut<'_, T>) -> bool {
        *self == other.as_const()
    }
}

impl<T> PartialEq for CollectionCursorMut<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_const() == other.as_const()
    }
}

impl<T> PartialEq<CollectionCursor<'_, T>> for CollectionCursorMut<'_, T> {
    fn eq(&self, other: &CollectionCursor<'_, T>) -> bool {
        self.as_const() == *other
    }
}
