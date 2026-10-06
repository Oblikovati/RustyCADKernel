use super::{CollectionCursor, CollectionCursorMut, CursorPositions};
use crate::ncollection::Array1;

impl<T> Copy for CollectionCursor<'_, T> {}
impl<T> Clone for CollectionCursor<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Default for CollectionCursor<'_, T> {
    fn default() -> Self {
        Self {
            items: &[],
            positions: CursorPositions::default(),
            detached: true,
        }
    }
}
impl<T> Default for CollectionCursorMut<'_, T> {
    fn default() -> Self {
        Self {
            items: &mut [],
            positions: CursorPositions::default(),
            detached: true,
        }
    }
}

impl<'a, T> CollectionCursor<'a, T> {
    /// Constructs from Array1 while preserving its null empty storage; `CollectionCursor::from_array(&array)`.
    pub fn from_array(array: &'a Array1<'_, T>) -> Self {
        let mut cursor = Self::new(array.as_slice());
        cursor.detached = array.data_ptr().is_null();
        cursor
    }
    /// Restarts through OCCT's Initialize spelling; `cursor.initialize(&items)`.
    pub fn initialize(&mut self, items: &'a [T]) {
        self.init(items);
    }
    /// Copies native position and end, without cloning elements; `destination.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        *self = *source;
        self
    }
    /// Preserves native self assignment; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Assigns a shared view of a mutable cursor for its borrow lifetime; `view.assign_from_mutable(&cursor)`.
    pub fn assign_from_mutable(&mut self, source: &'a CollectionCursorMut<'_, T>) -> &mut Self {
        *self = source.as_const();
        self
    }
    /// Returns native current pointer identity; `cursor.value_iter_ptr()`.
    pub fn value_iter_ptr(&self) -> *const T {
        position_pointer(self.items.as_ptr(), self.positions.current, self.detached)
    }
    /// Returns native end pointer identity; `cursor.end_iter_ptr()`.
    pub fn end_iter_ptr(&self) -> *const T {
        position_pointer(self.items.as_ptr(), self.positions.last, self.detached)
    }
}

impl<'a, T> CollectionCursorMut<'a, T> {
    /// Constructs from Array1's exclusive storage, preserving null empty pointers; `CollectionCursorMut::from_array(&mut array)`.
    pub fn from_array(array: &'a mut Array1<'_, T>) -> Self {
        let detached = array.data_mut_ptr().is_null();
        let mut cursor = Self::new(array.as_mut_slice());
        cursor.detached = detached;
        cursor
    }
    /// Restarts through OCCT's Initialize spelling; `cursor.initialize(&mut items)`.
    pub fn initialize(&mut self, items: &'a mut [T]) {
        self.init(items);
    }
    /// Preserves writable native self assignment; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Returns current pointer identity without creating a writable alias; `cursor.value_iter_ptr()`.
    pub fn value_iter_ptr(&self) -> *const T {
        self.as_const().value_iter_ptr()
    }
    /// Returns end pointer identity without creating a writable alias; `cursor.end_iter_ptr()`.
    pub fn end_iter_ptr(&self) -> *const T {
        self.as_const().end_iter_ptr()
    }
    /// Returns writable current pointer identity; `cursor.change_value_iter_ptr()`.
    /// Dereferencing it requires preserving this cursor's exclusive storage borrow.
    pub fn change_value_iter_ptr(&mut self) -> *mut T {
        if self.detached {
            return std::ptr::null_mut();
        }
        self.items.as_mut_ptr().wrapping_add(self.positions.current)
    }
    /// Returns writable end pointer identity; `cursor.change_end_iter_ptr()`.
    /// The end pointer is normally one past storage and must not be dereferenced.
    pub fn change_end_iter_ptr(&mut self) -> *mut T {
        if self.detached {
            return std::ptr::null_mut();
        }
        self.items.as_mut_ptr().wrapping_add(self.positions.last)
    }
}

fn position_pointer<T>(origin: *const T, offset: usize, detached: bool) -> *const T {
    if detached {
        return std::ptr::null();
    }
    origin.wrapping_add(offset)
}
