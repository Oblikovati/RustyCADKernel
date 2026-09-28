// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Stable keys with dense one-based indices and OCCT's swap-last removal.
use super::hash_policy::next_map_prime;
use super::{AllocatedBox, BaseAllocator, DefaultHashPolicy, HashPolicy};
use crate::standard::Failure;
use std::fmt::Debug;
struct IndexedNode<K> {
    key: K,
    _identity: u8,
}
/// A hash map from unique keys to dense indices; `IndexedMap::<i32>::default().add(42)`.
pub struct IndexedMap<K, H = DefaultHashPolicy> {
    nodes: Vec<AllocatedBox<IndexedNode<K>>>,
    buckets: Vec<Vec<usize>>,
    bucket_count: usize,
    resource: BaseAllocator,
    hasher: H,
}
/// Shared keys in index order; `map.iter()`.
pub struct IndexedMapIter<'a, K> {
    nodes: &'a [AllocatedBox<IndexedNode<K>>],
    position: usize,
}
/// Shared keys paired with one-based indices; `map.indexed_items()`.
pub struct IndexedMapItems<'a, K> {
    inner: IndexedMapIter<'a, K>,
}
impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Injects a hash policy and initial bucket count; `IndexedMap::with_hasher(policy,10)`.
    pub fn with_hasher(hasher: H, buckets: usize) -> Self {
        Self::with_allocator(hasher, buckets, BaseAllocator::common())
    }
    /// Selects the allocator for stable key nodes; `IndexedMap::with_allocator(policy,10,arena)`.
    pub fn with_allocator(hasher: H, buckets: usize, resource: BaseAllocator) -> Self {
        Self {
            nodes: Vec::new(),
            buckets: Vec::new(),
            bucket_count: buckets.max(1),
            resource,
            hasher,
        }
    }
    /// Returns key count; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    /// Returns OCCT's index extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.len()
    }
    /// Checks whether there are no keys; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    /// Returns the hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.bucket_count
    }
    /// Borrows the retained hash policy; `map.hasher().salt`.
    pub fn hasher(&self) -> &H {
        &self.hasher
    }
    /// Borrows the node allocator; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.resource
    }
    /// Grows hash storage without relocating keys; `map.resize(200)`.
    pub fn resize(&mut self, requested: usize) {
        let count = next_map_prime(requested).max(self.bucket_count);
        if count == self.bucket_count && !self.buckets.is_empty() {
            return;
        }
        let mut buckets: Vec<Vec<usize>> = (0..=count).map(|_| Vec::new()).collect();
        for bucket in &self.buckets {
            for &index in bucket {
                let hash = self.hasher.hash(&self.nodes[index].key) % count + 1;
                buckets[hash].insert(0, index);
            }
        }
        self.buckets = buckets;
        self.bucket_count = count;
    }
    /// Returns a key's one-based index or zero when absent; `map.find_index(&42)`.
    pub fn find_index(&self, key: &K) -> usize {
        if self.is_empty() {
            return 0;
        }
        self.buckets[self.hash_bucket(key)]
            .iter()
            .find(|&&index| self.hasher.equal(&self.nodes[index].key, key))
            .map_or(0, |index| index + 1)
    }
    /// Checks membership through the injected policy; `map.contains(&42)`.
    pub fn contains(&self, key: &K) -> bool {
        self.find_index(key) != 0
    }
    /// Borrows a key by one-based index; `map.find_key(1)?`.
    pub fn find_key(&self, index: usize) -> Result<&K, Failure> {
        self.check_index(index)?;
        Ok(&self.nodes[index - 1].key)
    }
    /// Inserts a key or returns its existing index; `map.add(42)==1`.
    pub fn add(&mut self, key: K) -> usize {
        self.place(key, false)
    }
    /// Inserts a key and borrows its stable storage; `map.added(42)`.
    pub fn added(&mut self, key: K) -> &K {
        let index = self.place(key, false);
        &self.nodes[index - 1].key
    }
    /// Reconstructs an equivalent key in place; `map.emplace(42)==1`.
    pub fn emplace(&mut self, key: K) -> usize {
        self.place(key, true)
    }
    /// Reconstructs a key and borrows its stable storage; `map.emplaced(42)`.
    pub fn emplaced(&mut self, key: K) -> &K {
        let index = self.place(key, true);
        &self.nodes[index - 1].key
    }
    /// Frees keys and optionally bucket storage; `map.clear(true)`.
    pub fn clear(&mut self, release: bool) {
        self.nodes.clear();
        if release {
            self.buckets.clear();
            self.nodes.shrink_to_fit();
        } else {
            for bucket in &mut self.buckets {
                bucket.clear();
            }
        }
    }
    /// Swaps nodes, allocator and hash policy; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    fn hash_bucket(&self, key: &K) -> usize {
        self.hasher.hash(key) % self.bucket_count + 1
    }
    fn check_index(&self, index: usize) -> Result<(), Failure> {
        if index == 0 || index > self.len() {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!("IndexedMap index {index}; expected 1..={}", self.len()),
                "",
            ));
        }
        Ok(())
    }
    fn place(&mut self, key: K, replace: bool) -> usize {
        if self.is_empty() || self.len() > self.bucket_count {
            self.resize(self.len());
        }
        let index = self.find_index(&key);
        if index != 0 {
            if replace {
                self.nodes[index - 1].key = key;
            }
            return index;
        }
        self.append_new(key)
    }
    fn append_new(&mut self, key: K) -> usize {
        let bucket = self.hash_bucket(&key);
        let index = self.len();
        self.nodes.push(
            self.resource
                .allocate_value(IndexedNode { key, _identity: 0 }),
        );
        self.buckets[bucket].insert(0, index);
        index + 1
    }
}
impl<K: Debug, H: HashPolicy<K> + Default> IndexedMap<K, H> {
    /// Selects initial buckets with the default hasher; `IndexedMap::<i32>::new(3)`.
    pub fn new(buckets: usize) -> Self {
        Self::with_hasher(H::default(), buckets)
    }
}
impl<K: Debug, H: HashPolicy<K> + Default> Default for IndexedMap<K, H> {
    fn default() -> Self {
        Self::new(1)
    }
}
impl<K: Debug, H: HashPolicy<K>> std::ops::Index<usize> for IndexedMap<K, H> {
    type Output = K;
    fn index(&self, index: usize) -> &K {
        self.find_key(index)
            .expect("IndexedMap index must be in 1..=len")
    }
}
mod iteration;
mod operations;
