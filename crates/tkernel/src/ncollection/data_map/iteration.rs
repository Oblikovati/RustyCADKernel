//! Safe ranges borrow the sole graph owner while following native links.
use super::{position::DataMapPosition, storage::DataMapStorage};
use crate::ncollection::DataMapNode;
use std::{marker::PhantomData, panic::RefUnwindSafe};
/// Shared key/value traversal in native bucket order; `map.items()`.
pub struct DataMapItems<'a, K, V> {
    pub(super) source: Option<&'a DataMapStorage<K, V>>,
    pub(super) position: DataMapPosition,
}
impl<'a, K, V> DataMapItems<'a, K, V> {
    pub(super) fn new(source: &'a DataMapStorage<K, V>) -> Self {
        Self {
            source: Some(source),
            position: DataMapPosition::new(&source.base),
        }
    }
    pub(super) fn current(&self) -> Option<&'a DataMapNode<K, V>> {
        if self.position.node.is_null() {
            return None;
        }
        Some(unsafe { self.source?.node(self.position.node) })
    }
    pub(super) fn reset(&mut self) {
        self.position = self.source.map_or_else(DataMapPosition::default, |source| {
            DataMapPosition::new(&source.base)
        });
    }
}
// SAFETY: each shared range borrows a graph that cannot mutate; references require K,V:Sync.
unsafe impl<K: Sync, V: Sync> Send for DataMapItems<'_, K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for DataMapItems<'_, K, V> {}
impl<'a, K, V> Iterator for DataMapItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.current()?;
        self.position.advance(&self.source?.base);
        Some((entry.key(), entry.value()))
    }
}
impl<K, V> Default for DataMapItems<'_, K, V> {
    fn default() -> Self {
        Self {
            source: None,
            position: DataMapPosition::default(),
        }
    }
}
impl<K, V> Clone for DataMapItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            position: self.position,
        }
    }
}
impl<K, V> PartialEq for DataMapItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.position.node == other.position.node
    }
}
impl<K, V> Eq for DataMapItems<'_, K, V> {}

pub(super) struct DataMapItemsMut<'a, K, V> {
    source: &'a DataMapStorage<K, V>,
    position: DataMapPosition,
    exclusive: PhantomData<&'a mut (K, V)>,
}
impl<'a, K, V> DataMapItemsMut<'a, K, V> {
    pub(super) fn new(source: &'a mut DataMapStorage<K, V>) -> Self {
        Self {
            position: DataMapPosition::new(&source.base),
            source,
            exclusive: PhantomData,
        }
    }
}
// SAFETY: an exclusive owner borrow freezes topology; next yields each disjoint value once.
unsafe impl<K: Send, V: Send> Send for DataMapItemsMut<'_, K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for DataMapItemsMut<'_, K, V> {}
impl<K: RefUnwindSafe, V: RefUnwindSafe> RefUnwindSafe for DataMapItemsMut<'_, K, V> {}
impl<K, V> Unpin for DataMapItemsMut<'_, K, V> {}
impl<'a, K, V> Iterator for DataMapItemsMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.position.node;
        if node.is_null() {
            return None;
        }
        // Read the current link before lending its value, never reborrow a previously yielded node.
        self.position.advance(&self.source.base);
        Some(unsafe { (&mut *node.cast::<DataMapNode<K, V>>()).key_value_mut() })
    }
}
