use super::{Array1, Storage};

impl<T: Default> Array1<'_, T> {
    pub(in crate::ncollection) fn resize_no_data(&mut self, length: usize) {
        if length == self.len() {
            return;
        }
        let mut values = Vec::new();
        values.resize_with(length, T::default);
        self.storage = Storage::Owned(values);
    }
}
