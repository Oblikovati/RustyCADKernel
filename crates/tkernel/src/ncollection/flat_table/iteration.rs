use super::FlatItems;
impl<K, V> FlatItems<'_, K, V> {
    pub(super) fn seek(&mut self) {
        while self.index < self.slots.len() && self.slots[self.index].is_none() {
            self.index += 1;
        }
    }
}
impl<'a, K, V> Iterator for FlatItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.slots.get(self.index)?.as_ref()?;
        self.index += 1;
        self.seek();
        Some((&slot.key, &slot.value))
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
        (self.index == self.slots.len() && other.index == other.slots.len())
            || (self.slots.as_ptr() == other.slots.as_ptr() && self.index == other.index)
    }
}
impl<K, V> Eq for FlatItems<'_, K, V> {}
