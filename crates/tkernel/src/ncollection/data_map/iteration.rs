//! Safe ranges borrow the sole graph owner while following native links.
use super::storage::DataMapStorage;
use crate::ncollection::{
    DataMapNode,
    owned_bucket_chains::{BucketChainPosition, BucketNodeRange},
};
use std::{marker::PhantomData, panic::RefUnwindSafe};
/// Shared key/value traversal in native bucket order; `map.items()`.
pub struct DataMapItems<'a, K, V> {
    nodes: BucketNodeRange<'a, DataMapNode<K, V>>,
}
impl<'a, K, V> DataMapItems<'a, K, V> {
    pub(super) fn new(source: &'a DataMapStorage<K, V>) -> Self {
        Self {
            nodes: BucketNodeRange::new(source),
        }
    }
    pub(super) fn from_position(
        source: Option<&'a DataMapStorage<K, V>>,
        position: BucketChainPosition,
    ) -> Self {
        Self {
            nodes: BucketNodeRange::from_position(source, position),
        }
    }
    pub(super) fn current(&self) -> Option<&'a DataMapNode<K, V>> {
        self.nodes.current()
    }
    pub(super) fn reset(&mut self) {
        self.nodes.reset();
    }
}
impl<'a, K, V> Iterator for DataMapItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.nodes.next().map(|node| (node.key(), node.value()))
    }
}
impl<K, V> Default for DataMapItems<'_, K, V> {
    fn default() -> Self {
        Self {
            nodes: BucketNodeRange::default(),
        }
    }
}
impl<K, V> Clone for DataMapItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
        }
    }
}
impl<K, V> PartialEq for DataMapItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}
impl<K, V> Eq for DataMapItems<'_, K, V> {}

pub(super) struct DataMapItemsMut<'a, K, V> {
    source: &'a DataMapStorage<K, V>,
    position: BucketChainPosition,
    exclusive: PhantomData<&'a mut (K, V)>,
}
impl<'a, K, V> DataMapItemsMut<'a, K, V> {
    pub(super) fn new(source: &'a mut DataMapStorage<K, V>) -> Self {
        Self {
            position: BucketChainPosition::new(&source.base),
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
