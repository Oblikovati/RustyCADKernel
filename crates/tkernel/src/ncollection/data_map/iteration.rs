//! Safe ranges borrow the map and follow its bucket chains in native order.
use super::DataMapStorage;
use crate::ncollection::slot_chains::{ChainPosition, ChainSlot, SlotRange};
/// Shared key/value traversal in native bucket order; `map.items()`.
pub struct DataMapItems<'a, K, V> {
    entries: SlotRange<'a, K, V>,
}
impl<'a, K, V> DataMapItems<'a, K, V> {
    pub(super) fn new(source: &'a DataMapStorage<K, V>) -> Self {
        Self {
            entries: SlotRange::new(source.view()),
        }
    }
    pub(super) fn from_position(
        source: Option<&'a DataMapStorage<K, V>>,
        position: ChainPosition,
    ) -> Self {
        Self {
            entries: SlotRange::from_position(source.map(DataMapStorage::view), position),
        }
    }
    pub(super) fn current(&self) -> Option<&'a ChainSlot<K, V>> {
        self.entries.current()
    }
    pub(super) fn reset(&mut self) {
        self.entries.reset();
    }
}
impl<'a, K, V> Iterator for DataMapItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.entries
            .next()
            .map(|entry| (entry.key(), entry.value()))
    }
}
impl<K, V> Default for DataMapItems<'_, K, V> {
    fn default() -> Self {
        Self {
            entries: SlotRange::default(),
        }
    }
}
impl<K, V> Clone for DataMapItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}
impl<K, V> PartialEq for DataMapItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl<K, V> Eq for DataMapItems<'_, K, V> {}
