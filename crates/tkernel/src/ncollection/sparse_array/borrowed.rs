//! Native copy construction publishes occupancy only after the copy succeeds.
use super::SparseArray;

impl<T> SparseArray<T> {
    pub(super) fn ensure_sparse_block(&mut self, number: usize) {
        self.reserve_block(number);
        let increment = self.increment;
        self.blocks
            .entry(number)
            .or_insert_with(|| std::iter::repeat_with(|| None).take(increment).collect());
    }
}

impl<T: Clone> SparseArray<T> {
    /// Copies into a new slot or assigns an existing object in place; `array.set_value_ref(5, &55)`.
    pub fn set_value_ref(&mut self, index: usize, value: &T) -> &mut T {
        let number = index / self.increment;
        self.ensure_sparse_block(number);
        let slot = &mut self.blocks.get_mut(&number).unwrap()[index % self.increment];
        if let Some(previous) = slot {
            previous.clone_from(value);
        } else {
            *slot = Some(value.clone());
            self.count += 1;
        }
        slot.as_mut().unwrap()
    }

    /// Copies a value using the native map spelling; `array.bind_ref(5, &55)`.
    pub fn bind_ref(&mut self, index: usize, value: &T) -> &mut T {
        self.set_value_ref(index, value)
    }
}
