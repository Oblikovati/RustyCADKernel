//! Canonical Map<Cell> and Object-first list nodes, without duplicate order/owner registries.
#![allow(unsafe_code)]
use super::{CellHash, CellIndex};
#[cfg(test)]
use crate::ncollection::owned_bucket_chains::BucketNodeRange;
use crate::ncollection::owned_bucket_chains::OwnedBucketChains;
use crate::ncollection::{BaseAllocator, BaseMap, HashPolicy, MapLinkNode, MapNode};
mod cell_node;
mod object_node;
mod retention;
use cell_node::{CellMapNode, CellRecord};
use object_node::CellObjectNode;
pub(super) struct CellMap<T> {
    storage: OwnedBucketChains<CellMapNode<T>>,
}
#[derive(Clone, Copy)]
struct CellMapHash;
impl<T> HashPolicy<CellRecord<T>> for CellMapHash {
    fn hash(&self, cell: &CellRecord<T>) -> usize {
        CellHash.hash(&cell.index)
    }
    fn equal(&self, first: &CellRecord<T>, second: &CellRecord<T>) -> bool {
        CellHash.equal(&first.index, &second.index)
    }
}
impl<T> CellMap<T> {
    pub(super) fn new(allocator: BaseAllocator) -> Self {
        Self {
            storage: OwnedBucketChains::new(1, allocator),
        }
    }
    pub(super) fn clear_with_allocator(&mut self, requested: Option<BaseAllocator>) {
        self.storage.clear_with_allocator(requested);
    }
    fn lookup_index(&self, index: &CellIndex) -> Option<(usize, *mut MapLinkNode)> {
        if self.storage.base.size() == 0 {
            return None;
        }
        let (bucket, found) = self.storage.lookup_with_bucket_by(
            index,
            |key| CellHash.hash(key),
            |cell, key| CellHash.equal(&cell.index, key),
        );
        found.map(|node| (bucket, node))
    }
    fn prepare_cell(&mut self, index: &CellIndex) -> *mut CellRecord<T> {
        let size = self.storage.base.size();
        if size == 0 || size > self.storage.base.nb_buckets() {
            self.storage.resize(size, &CellMapHash);
        }
        // Native TryEmplaced constructs the lookup Cell after map growth, even for a hit.
        let cell = CellRecord::from_index(index);
        let (bucket, existing) = self.storage.lookup_with_bucket(&cell, &CellMapHash);
        let node = existing.unwrap_or_else(|| {
            self.storage
                .allocate_node(bucket, |next| CellMapNode(MapNode::new(cell, next)))
        });
        unsafe { CellMapNode::cell_ptr(node.cast()) }
    }
}
impl<T: Clone> CellMap<T> {
    pub(super) fn add(&mut self, index: &CellIndex, target: &T) {
        let cell = self.prepare_cell(index);
        let allocator = self.storage.base.allocator();
        unsafe { CellObjectNode::prepend(cell, target, allocator) };
    }
}
#[cfg(test)]
impl<T> CellMap<T> {
    pub(super) fn len(&self) -> usize {
        self.storage.base.size()
    }
    pub(super) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(super) fn bucket_count(&self) -> usize {
        self.storage.base.nb_buckets()
    }
    pub(super) fn allocator(&self) -> &BaseAllocator {
        self.storage.base.allocator()
    }
    pub(super) fn items(&self) -> impl Iterator<Item = (&CellIndex, &CellRecord<T>)> {
        BucketNodeRange::new(&self.storage).map(|node| (&node.0.key().index, node.0.key()))
    }
}

#[cfg(test)]
mod native_tests;
