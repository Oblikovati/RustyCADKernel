//! Canonical native tables are the sole owning graph; no separate owner or order registry.
use crate::ncollection::{BaseAllocator, BaseMap, IndexedMapNode, MapLinkNode};
use std::marker::PhantomData;
mod topology;
use topology::IndexedTopology;
pub use topology::{IndexedMapObservation, IndexedMapObserver};
mod removal;
mod resize;
pub(super) struct IndexedStorage<K> {
    topology: IndexedTopology,
    ownership: PhantomData<Box<K>>,
}
// SAFETY: only this storage owns its reachable nodes; mutation needs &mut and observer
// copies no payload. Moving ownership does not move K; links are non-owning prefixes.
unsafe impl<K: Send> Send for IndexedStorage<K> {}
unsafe impl<K: Sync> Sync for IndexedStorage<K> {}
impl<K> Unpin for IndexedStorage<K> {}
impl<K: std::panic::RefUnwindSafe> std::panic::RefUnwindSafe for IndexedStorage<K> {}
impl<K: std::panic::UnwindSafe> std::panic::UnwindSafe for IndexedStorage<K> {}
impl<K> IndexedStorage<K> {
    pub(super) fn new(count: usize, allocator: BaseAllocator) -> Self {
        Self {
            topology: IndexedTopology::new(count, allocator),
            ownership: PhantomData,
        }
    }
    pub(super) fn len(&self) -> usize {
        self.topology.read().size()
    }
    pub(super) fn bucket_count(&self) -> usize {
        self.topology.read().nb_buckets()
    }
    pub(super) fn allocator(&self) -> &BaseAllocator {
        self.topology.read().allocator()
    }
    pub(super) fn indices(&self) -> &[*mut MapLinkNode] {
        &self.topology.read().index_slots()[..self.len()]
    }
    pub(super) fn node(&self, index: usize) -> *mut MapLinkNode {
        self.topology.read().index_slots()[index - 1]
    }
    pub(super) fn head(&self, bucket: usize) -> *mut MapLinkNode {
        self.topology.read().base().bucket_slots()[bucket]
    }
    pub(super) fn key(&self, index: usize) -> &K {
        // SAFETY: callers validate indices; only shared keys escape the owner borrow.
        unsafe { (&*self.node(index).cast::<IndexedMapNode<K>>()).value() }
    }
    pub(super) fn key_mut(&mut self, index: usize) -> &mut K {
        // SAFETY: exclusive map access excludes other payload borrows.
        unsafe { (&mut *self.node(index).cast::<IndexedMapNode<K>>()).key1() }
    }
    pub(super) fn observer(&self) -> IndexedMapObserver {
        self.topology.observer()
    }
    pub(super) fn exchange(&mut self, other: &mut Self) {
        self.topology.exchange(&mut other.topology);
    }
    pub(super) fn append(&mut self, bucket: usize, construct: impl FnOnce() -> K) -> usize {
        let index = self.len() + 1;
        let owner = IndexedMapNode::allocate_with(
            self.allocator(),
            index as i32,
            self.head(bucket),
            construct,
        );
        let (node, _allocator) = IndexedMapNode::into_raw(owner);
        self.topology.edit(|base| {
            base.base_mut().bucket_slots_mut()[bucket] = node;
            base.index_slots_mut()[index - 1] = node;
            base.base_mut().increment();
        });
        index
    }
    pub(super) fn swap(&mut self, first: usize, second: usize) {
        let (a, b) = (self.node(first), self.node(second));
        // SAFETY: distinct live nodes are exclusively owned, and only indices change.
        unsafe {
            *(*a.cast::<IndexedMapNode<K>>()).index() = second as i32;
            *(*b.cast::<IndexedMapNode<K>>()).index() = first as i32;
        }
        self.topology
            .edit(|base| base.index_slots_mut().swap(first - 1, second - 1));
    }
}
impl<K> Drop for IndexedStorage<K> {
    fn drop(&mut self) {
        self.clear(true);
    }
}
