//! Exclusive legacy cursor with non-consuming access to the current binding.
use super::{FlatDataItems, FlatDataMap, FlatItems, FlatSlot, HashPolicy};
use crate::standard::{Failure, exceptions};
use std::fmt::Debug;

/// A mutable legacy cursor borrowing one map; `map.cursor_mut().change_value()?`.
pub struct FlatDataCursorMut<'a, K, V> {
    slots: &'a mut [Option<FlatSlot<K, V>>],
    index: usize,
}

impl<K: Debug, V, H: HashPolicy<K>> FlatDataMap<K, V, H> {
    /// Borrows a mutable cursor at the first occupied slot; `map.cursor_mut()`.
    pub fn cursor_mut(&mut self) -> FlatDataCursorMut<'_, K, V> {
        let index = self.table.items().index;
        FlatDataCursorMut {
            slots: &mut self.table.slots,
            index,
        }
    }
}

impl<K, V> FlatDataCursorMut<'_, K, V> {
    /// Borrows an immutable cursor at the same position; `cursor.as_cursor().key()?`.
    pub fn as_cursor(&self) -> FlatDataItems<'_, K, V> {
        FlatDataItems {
            inner: FlatItems {
                slots: self.slots,
                index: self.index,
            },
        }
    }

    /// Checks for a current binding without advancing; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.as_cursor().more()
    }

    /// Advances with the same slot scan as a read-only cursor; `cursor.advance()`.
    pub fn advance(&mut self) {
        let mut view = self.as_cursor();
        view.advance();
        self.index = view.inner.index;
    }

    /// Borrows the current key without changing it; `cursor.key()?`.
    pub fn key(&self) -> Result<&K, Failure> {
        self.as_cursor().key()
    }

    /// Borrows the current value without consuming it; `cursor.value()?`.
    pub fn value(&self) -> Result<&V, Failure> {
        self.as_cursor().value()
    }

    /// Borrows the current value for mutation; `*cursor.change_value()? = replacement`.
    pub fn change_value(&mut self) -> Result<&mut V, Failure> {
        self.slots
            .get_mut(self.index)
            .and_then(Option::as_mut)
            .map(|slot| &mut slot.value)
            .ok_or_else(|| {
                exceptions::out_of_range("NCollection_FlatDataMap::Iterator::ChangeValue")
            })
    }

    /// Compares native cursor position, with all exhausted cursors equal; `cursor.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.as_cursor() == other.as_cursor()
    }
}

impl<K, V> Default for FlatDataCursorMut<'_, K, V> {
    fn default() -> Self {
        Self {
            slots: &mut [],
            index: 0,
        }
    }
}
