use super::FlatItems;
impl<'a, K, V> FlatItems<'a, K, V> {
    pub(in crate::ncollection) fn more(&self) -> bool {
        self.index < self.slots.len()
    }
    pub(in crate::ncollection) fn current(&self) -> Option<(&'a K, &'a V)> {
        let slot = self.slots.get(self.index)?.as_ref()?;
        Some((&slot.key, &slot.value))
    }
    pub(in crate::ncollection) fn advance(&mut self) {
        // Native Next increments even an exhausted cursor; equality uses More().
        self.index = self.index.wrapping_add(1);
        self.seek();
    }
    pub(super) fn seek(&mut self) {
        while self.index < self.slots.len() && self.slots[self.index].is_none() {
            self.index += 1;
        }
    }
}
impl<'a, K, V> Iterator for FlatItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current()?;
        self.advance();
        Some(current)
    }
}
impl<K, V> Clone for FlatItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            slots: self.slots,
            index: self.index,
        }
    }
}
// The default-constructed native iterator has no slots and compares equal to every end.
impl<K, V> Default for FlatItems<'_, K, V> {
    fn default() -> Self {
        Self {
            slots: &[],
            index: 0,
        }
    }
}
impl<K, V> PartialEq for FlatItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        (!self.more() && !other.more())
            || (self.slots.as_ptr() == other.slots.as_ptr() && self.index == other.index)
    }
}
impl<K, V> Eq for FlatItems<'_, K, V> {}
