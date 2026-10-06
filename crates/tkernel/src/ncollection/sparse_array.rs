//! Sparse block allocation with stable occupied slots and ascending index traversal.
#![allow(unsafe_code)]
mod iteration;
mod storage;
use storage::TypedSparseStorage;
mod assignment;
mod borrowed;
mod cursor;
mod cursor_mut;
mod cursor_position;
mod legacy_cursor;
pub use cursor::SparseArrayCursor;
pub use cursor_mut::SparseArrayCursorMut;
pub use legacy_cursor::SparseArrayLegacyCursor;
/// Sparse indexed objects; `SparseArray::new(8).set_value(1000, 42)`.
pub struct SparseArray<T> {
    storage: TypedSparseStorage<T>,
}
impl<T> SparseArray<T> {
    /// Selects the number of slots allocated per block; `SparseArray::<i32>::new(8)`.
    pub fn new(increment: usize) -> Self {
        Self {
            storage: TypedSparseStorage::new(increment),
        }
    }
    /// Addressable slots before the block table must grow, as native
    /// `myNbBlocks * myBlockSize`; `SparseArray::<i32>::new(8).capacity() == 0`.
    pub fn capacity(&self) -> usize {
        self.storage.base.capacity()
    }
    /// Returns the number of occupied slots; `array.len() == 2`.
    pub fn len(&self) -> usize {
        self.storage.base.size()
    }
    /// Returns the native item count; `array.size() == array.len()`.
    pub fn size(&self) -> usize {
        self.storage.base.size()
    }
    /// Returns the occupied extent; `array.extent() == 2`.
    pub fn extent(&self) -> usize {
        self.storage.base.size()
    }
    /// Checks that every slot is unoccupied; `array.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.storage.base.size() == 0
    }
    /// Constructs or replaces an indexed object; `array.set_value(5, 55)`.
    pub fn set_value(&mut self, index: usize, value: T) -> &mut T {
        self.storage.insert_owned(index, value)
    }
    /// Borrows an occupied slot; `array.value(5) == Some(&55)`.
    pub fn value(&self, index: usize) -> Option<&T> {
        self.storage.value(index)
    }
    /// Borrows an occupied slot exclusively; `*array.change_value(5).unwrap() = 10`.
    pub fn change_value(&mut self, index: usize) -> Option<&mut T> {
        self.storage.value_mut(index)
    }
    /// Checks whether a slot is constructed; `array.has_value(5)`.
    pub fn has_value(&self, index: usize) -> bool {
        self.storage.base.has_value(index)
    }
    /// Destroys an occupied slot; `array.unset_value(5)`.
    pub fn unset_value(&mut self, index: usize) -> bool {
        self.storage.remove(index)
    }
    /// Frees all blocks and objects; `array.clear()`.
    pub fn clear(&mut self) {
        self.storage.clear();
    }
    /// Visits occupied indices in ascending order; `array.entries().map(|(index, _)| index)`.
    pub fn entries(&self) -> impl Iterator<Item = (usize, &T)> {
        iteration::SparseEntries::new(self)
    }
    /// Visits occupied values in index order; `array.iter().sum::<i32>()`.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries().map(|(_, value)| value)
    }
    /// Visits occupied values exclusively; `array.iter_mut().for_each(|v| *v += 1)`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries_mut().map(|(_, value)| value)
    }
    /// Visits occupied slots exclusively with their indices, as the native non-const
    /// `Iterator` (`Key()` plus `ChangeValue()`); `array.entries_mut().for_each(|(i, v)| *v += i)`.
    pub fn entries_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        iteration::SparseEntriesMut::new(self)
    }
    /// Swaps contents and block geometry; `array.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// Data-map spelling of insertion; `array.bind(5, 55)`.
    pub fn bind(&mut self, index: usize, value: T) -> &mut T {
        self.set_value(index, value)
    }
    /// Data-map spelling of occupancy; `array.is_bound(5)`.
    pub fn is_bound(&self, index: usize) -> bool {
        self.has_value(index)
    }
    /// Data-map spelling of lookup; `array.find(5)`.
    pub fn find(&self, index: usize) -> Option<&T> {
        self.value(index)
    }
    /// Data-map spelling of exclusive lookup; `array.change_find(5)`.
    pub fn change_find(&mut self, index: usize) -> Option<&mut T> {
        self.change_value(index)
    }
    /// Data-map spelling of removal; `array.unbind(5)`.
    pub fn unbind(&mut self, index: usize) -> bool {
        self.unset_value(index)
    }
}
impl<T> std::ops::Index<usize> for SparseArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.value(index)
            .unwrap_or_else(|| panic!("SparseArray index {index}; expected an occupied slot"))
    }
}
impl<T> std::ops::IndexMut<usize> for SparseArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.change_value(index)
            .unwrap_or_else(|| panic!("SparseArray index {index}; expected an occupied slot"))
    }
}
