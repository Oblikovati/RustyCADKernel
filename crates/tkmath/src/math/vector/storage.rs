use super::{Failure, VectorBase, VectorItem, failure};
#[derive(Debug)]
pub(super) enum VectorStorage<'a, T> {
    Inline,
    Owned(Vec<T>),
    Borrowed(&'a mut [T]),
}
impl<T: VectorItem> VectorBase<'_, T> {
    pub(crate) fn values(&self) -> &[T] {
        match &self.storage {
            VectorStorage::Inline => &self.inline[..self.length],
            VectorStorage::Owned(values) => values,
            VectorStorage::Borrowed(values) => values,
        }
    }
    pub(super) fn values_mut(&mut self) -> &mut [T] {
        match &mut self.storage {
            VectorStorage::Inline => &mut self.inline[..self.length],
            VectorStorage::Owned(values) => values,
            VectorStorage::Borrowed(values) => values,
        }
    }
    fn owns_values(&self) -> bool {
        matches!(&self.storage,VectorStorage::Owned(values) if !values.is_empty())
    }
    /// Moves heap storage and copies inline/external storage; `Vector::move_construct(&mut source)`.
    pub fn move_construct(source: &mut Self) -> Self {
        if source.owns_values() {
            return source.take_owned();
        }
        let mut result = Self::with_size(source.length);
        result.lower = source.lower;
        result.values_mut().copy_from_slice(source.values());
        result
    }
    /// Moves compatible heap storage or copies matching lengths; `target.move_assign(&mut source)?`.
    pub fn move_assign(&mut self, source: &mut Self) -> Result<(), Failure> {
        if self.owns_values()
            && source.owns_values()
            && self.lower == source.lower
            && self.length == source.length
        {
            *self = source.take_owned();
            return Ok(());
        }
        self.initialized(source)
    }
    fn take_owned(&mut self) -> Self {
        let mut empty = Self::with_size(0);
        empty.lower = 1;
        std::mem::replace(self, empty)
    }
    /// Resizes while retaining the lower bound and existing initialized prefix; `vector.resize(50)?`.
    pub fn resize(&mut self, length: i32) -> Result<(), Failure> {
        if length < 0 {
            return Err(failure("Standard_RangeError", "NCollection_Array1::Resize"));
        }
        let length = length as usize;
        if length <= 32 {
            self.resize_inline(length);
        } else {
            self.resize_heap(length);
        }
        self.length = length;
        Ok(())
    }
    fn resize_inline(&mut self, length: usize) {
        let count = self.length.min(length);
        if let VectorStorage::Owned(values) = &self.storage {
            self.inline[..count].copy_from_slice(&values[..count]);
        }
        self.storage = VectorStorage::Inline;
    }
    fn resize_heap(&mut self, length: usize) {
        if let VectorStorage::Owned(values) = &mut self.storage {
            values.resize(length, T::ZERO);
            return;
        }
        let count = self.length.min(length);
        let mut values = vec![T::ZERO; length];
        values[..count].copy_from_slice(&self.values()[..count]);
        self.storage = VectorStorage::Owned(values);
    }
}
impl<T: VectorItem> Clone for VectorBase<'_, T> {
    fn clone(&self) -> Self {
        Self {
            lower: self.lower,
            length: self.length,
            inline: [T::ZERO; 32],
            storage: VectorStorage::Owned(self.values().to_vec()),
        }
    }
}
