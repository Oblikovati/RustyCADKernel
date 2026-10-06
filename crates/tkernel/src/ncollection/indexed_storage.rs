#![allow(unsafe_code)]
//! Canonical native tables are the sole owning graph; no separate owner or order registry.
use crate::ncollection::{BaseAllocator, BaseMap, MapLinkNode, node_owner::NativeNodeOwner};
use std::marker::PhantomData;
mod node_contract;
mod topology;
pub(super) use node_contract::IndexedNode;
use topology::IndexedTopology;
pub use topology::{IndexedMapObservation, IndexedMapObserver};
mod assignment;
mod queries;
mod removal;
mod resize;
pub(super) struct IndexedStorage<N: IndexedNode> {
    topology: IndexedTopology,
    ownership: PhantomData<Box<N>>,
}
// SAFETY: only this storage owns its reachable nodes; mutation needs &mut and observer
// copies no payload. Moving ownership does not move K; links are non-owning prefixes.
unsafe impl<N: IndexedNode> Send for IndexedStorage<N>
where
    N::Key: Send,
    N::Value: Send,
{
}
unsafe impl<N: IndexedNode> Sync for IndexedStorage<N>
where
    N::Key: Sync,
    N::Value: Sync,
{
}
impl<N: IndexedNode> Unpin for IndexedStorage<N> {}
impl<N: IndexedNode> std::panic::RefUnwindSafe for IndexedStorage<N>
where
    N::Key: std::panic::RefUnwindSafe,
    N::Value: std::panic::RefUnwindSafe,
{
}
impl<N: IndexedNode> std::panic::UnwindSafe for IndexedStorage<N>
where
    N::Key: std::panic::UnwindSafe,
    N::Value: std::panic::UnwindSafe,
{
}
impl<N: IndexedNode> IndexedStorage<N> {
    pub(super) fn new(count: usize, allocator: BaseAllocator) -> Self {
        Self {
            topology: IndexedTopology::new(crate::ncollection::BaseIndexedMapStorage::new(
                count,
                Some(allocator),
            )),
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
    pub(super) fn key(&self, index: usize) -> &N::Key {
        // SAFETY: callers validate indices; only shared keys escape the owner borrow.
        unsafe { &*N::key_ptr(self.node(index).cast()) }
    }
    pub(super) fn key_mut(&mut self, index: usize) -> &mut N::Key {
        // SAFETY: exclusive map access excludes other payload borrows.
        unsafe { &mut *N::key_ptr(self.node(index).cast()) }
    }
    pub(super) fn observer(&self) -> IndexedMapObserver {
        IndexedMapObserver(self.topology.observer())
    }
    pub(super) fn exchange(&mut self, other: &mut Self) {
        self.topology.exchange(&mut other.topology);
    }
    pub(super) fn append(
        &mut self,
        bucket: usize,
        construct: impl FnOnce(i32, *mut MapLinkNode) -> N,
    ) -> usize {
        let index = self.len() + 1;
        let next = self.head(bucket);
        let allocated = self
            .allocator()
            .allocate_value_with(|| construct(index as i32, next));
        let (pointer, _allocator) = NativeNodeOwner::from_allocated(allocated).into_parts();
        let node = pointer.cast();
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
            *N::index_ptr(a.cast()) = second as i32;
            *N::index_ptr(b.cast()) = first as i32;
        }
        self.topology
            .edit(|base| base.index_slots_mut().swap(first - 1, second - 1));
    }
}
impl<N: IndexedNode> Drop for IndexedStorage<N> {
    fn drop(&mut self) {
        self.clear(true);
    }
}
