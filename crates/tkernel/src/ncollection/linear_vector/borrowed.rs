//! Borrowed copy construction and copy assignment retain native publication order.
use super::{LengthRollback, LinearVector};

impl<T> LinearVector<T> {
    pub(super) fn resize_with_constructor(&mut self, size: usize, construct: impl FnMut() -> T) {
        if size < self.len() {
            self.elements.truncate(size);
            return;
        }
        self.reserve(size);
        let previous_len = self.len();
        let mut rollback = LengthRollback::new(&mut self.elements, previous_len);
        rollback.elements.resize_with(size, construct);
        rollback.commit();
    }
}
impl<T: Clone> LinearVector<T> {
    /// Copies after reserving the next slot, as native Append(const T&).
    /// `vector.append_ref(&value)` leaves grown capacity visible if cloning panics.
    pub fn append_ref(&mut self, value: &T) -> &mut T {
        self.reserve_next_item();
        self.push_reserved_item(value.clone())
    }
    fn set_value_ref_with(
        &mut self,
        index: usize,
        value: &T,
        construct: impl FnMut() -> T,
    ) -> &mut T {
        if index >= self.len() {
            self.resize_with_constructor(index + 1, construct);
        }
        self.elements[index].clone_from(value);
        &mut self.elements[index]
    }
}
impl<T: Clone + Default> LinearVector<T> {
    /// Assigns a borrowed value after default extension has published its size.
    /// `vector.set_value_ref(5, &value)` uses clone_from for the native copy-assignment branch.
    /// A clone_from panic retains the new size and any changes made to its target slot.
    pub fn set_value_ref(&mut self, index: usize, value: &T) -> &mut T {
        self.set_value_ref_with(index, value, T::default)
    }
}

#[cfg(test)]
mod tests;
