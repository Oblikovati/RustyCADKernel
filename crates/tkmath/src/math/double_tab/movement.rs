use super::{DoubleTab, TableShape, TableStorage};
impl<'a> DoubleTab<'a> {
    /// Transfers owned storage, copying inline/external values; `DoubleTab::move_construct(&mut source)`.
    pub fn move_construct(source: &mut Self) -> Self {
        if source.is_deletable() {
            return source.take_owned();
        }
        let mut result = Self {
            shape: source.shape,
            storage: TableStorage::new(source.shape.len(), true),
        };
        result.storage.slice_mut()[..source.shape.len()].copy_from_slice(source.values());
        result
    }
    /// Transfers only equal-shaped owned tables; otherwise copies as OCCT does; `dest.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        let same_shape =
            self.shape.size == source.shape.size && self.shape.lower == source.shape.lower;
        if self.is_deletable() && source.is_deletable() && same_shape {
            *self = source.take_owned();
        } else {
            self.assign(source);
        }
    }
    fn take_owned(&mut self) -> Self {
        let shape = std::mem::replace(&mut self.shape, TableShape::new([1, 1], [0, 0]));
        let storage = std::mem::replace(&mut self.storage, TableStorage::Owned(Vec::new()));
        Self { shape, storage }
    }
}
