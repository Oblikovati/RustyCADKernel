//! The historical base iterator retains its within-block offset at exhaustion.
use super::SparseArray;

/// Read cursor corresponding to `NCollection_SparseArrayBase::Iterator`; `array.legacy_cursor()`.
pub struct SparseArrayLegacyCursor<'a, T> {
    array: Option<&'a SparseArray<T>>,
    index: usize,
    more: bool,
}

impl<T> Default for SparseArrayLegacyCursor<'_, T> {
    fn default() -> Self {
        Self {
            array: None,
            index: 0,
            more: false,
        }
    }
}

impl<T> SparseArray<T> {
    /// Starts the historical base iterator; `array.legacy_cursor().index()`.
    pub fn legacy_cursor(&self) -> SparseArrayLegacyCursor<'_, T> {
        let mut cursor = SparseArrayLegacyCursor::default();
        cursor.initialize(self);
        cursor
    }
}

impl<'a, T> SparseArrayLegacyCursor<'a, T> {
    /// Binds the native protected initialization operation; `cursor.initialize(&array)`.
    pub fn initialize(&mut self, array: &'a SparseArray<T>) {
        self.array = Some(array);
        self.restart();
    }

    /// Restarts on the same array; `cursor.restart()`.
    pub fn restart(&mut self) {
        let Some(array) = self.array else {
            return;
        };
        self.more = false;
        self.index = array.capacity();
        if !array.blocks.is_empty() {
            self.seek_legacy_key(0);
        }
    }

    /// Reports whether a live object is selected; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.more
    }

    /// Returns the historical block-relative index; `cursor.index()`.
    /// Detached native Index dereferences null; Rust returns zero for its default cursor.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Advances across occupied slots, preserving the exhausted offset; `cursor.next()`.
    pub fn next(&mut self) {
        if self.more {
            self.seek_legacy_key(self.index + 1);
        }
    }

    /// Borrows the selected slot (native protected value); `cursor.value()`.
    pub fn value(&self) -> Option<&'a T> {
        if !self.more {
            return None;
        }
        self.array.and_then(|array| array.value(self.index))
    }

    fn seek_legacy_key(&mut self, start: usize) {
        let Some(array) = self.array else {
            return;
        };
        let next = array.entries().find(|(index, _)| *index >= start);
        self.more = next.is_some();
        self.index = next.map_or(array.capacity() + array.increment, |(index, _)| index);
    }
}
