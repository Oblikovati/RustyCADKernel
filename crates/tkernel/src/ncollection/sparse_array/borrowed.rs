//! Native copy construction publishes occupancy only after the copy succeeds.
use super::SparseArray;
impl<T: Clone> SparseArray<T> {
    /// Copies into a new slot or assigns an existing object in place; `array.set_value_ref(5, &55)`.
    pub fn set_value_ref(&mut self, index: usize, value: &T) -> &mut T {
        self.storage.insert_borrowed(index, value)
    }
    /// Copies a value using the native map spelling; `array.bind_ref(5, &55)`.
    pub fn bind_ref(&mut self, index: usize, value: &T) -> &mut T {
        self.set_value_ref(index, value)
    }
}
