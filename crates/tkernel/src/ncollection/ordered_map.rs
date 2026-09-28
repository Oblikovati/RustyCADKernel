// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Hash keys threaded in insertion order, with stable node identity.
use super::ordered_table::{OrderedItems, OrderedTable};
use super::{BaseAllocator, DefaultHashPolicy, HashPolicy};
use crate::standard::Failure;
/// An insertion-ordered set; `OrderedMap::<i32>::default().add(42)`.
pub struct OrderedMap<K, H = DefaultHashPolicy> {
    table: OrderedTable<K, (), H>,
}
/// Shared key range and legacy cursor; `map.iter().key()?`.
pub struct OrderedMapIter<'a, K> {
    inner: OrderedItems<'a, K, ()>,
}
impl<K, H: HashPolicy<K>> OrderedMap<K, H> {
    /// Injects hash/equality policy and initial buckets; `OrderedMap::with_hasher(policy,10)`.
    pub fn with_hasher(hasher: H, buckets: usize) -> Self {
        Self::with_allocator(hasher, buckets, BaseAllocator::common())
    }
    /// Selects the node allocator; `OrderedMap::with_allocator(policy,10,arena)`.
    pub fn with_allocator(hasher: H, buckets: usize, resource: BaseAllocator) -> Self {
        Self {
            table: OrderedTable::new(hasher, buckets, resource),
        }
    }
    /// Counts unique keys; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.table.count
    }
    /// Returns OCCT's key extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.len()
    }
    /// Checks for no keys; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.table.bucket_count
    }
    /// Borrows the retained hash policy; `map.hasher().salt`.
    pub fn hasher(&self) -> &H {
        &self.table.hasher
    }
    /// Borrows the node allocator; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.table.resource
    }
    /// Rehashes while retaining insertion order and node addresses; `map.resize(200)`.
    pub fn resize(&mut self, size: usize) {
        self.table.resize(size);
    }
    /// Tests membership with the retained policy; `map.contains(&42)`.
    pub fn contains(&self, key: &K) -> bool {
        self.table.lookup(key).is_some()
    }
    /// Borrows an equivalent stored key if present; `map.contained(&42)`.
    pub fn contained(&self, key: &K) -> Option<&K> {
        Some(&self.table.slots[self.table.lookup(key)?].as_ref()?.key)
    }
    /// Appends a new key without changing duplicate order; `map.add(42)`.
    pub fn add(&mut self, key: K) -> bool {
        self.table.place(key, || (), false, false).0
    }
    /// Borrows a stored key, appending only when absent; `map.added(42)`.
    pub fn added(&mut self, key: K) -> &K {
        let (_, slot) = self.table.place(key, || (), false, false);
        &self.table.slots[slot].as_ref().unwrap().key
    }
    /// Replaces an equivalent key without changing its position; `map.emplace(42)`.
    pub fn emplace(&mut self, key: K) -> bool {
        self.table.place(key, || (), true, false).0
    }
    /// Reconstructs and returns stable key storage; `map.emplaced(42)`.
    pub fn emplaced(&mut self, key: K) -> &K {
        let (_, slot) = self.table.place(key, || (), true, false);
        &self.table.slots[slot].as_ref().unwrap().key
    }
    /// Adds without replacing an equivalent key; `map.try_emplace(42)`.
    pub fn try_emplace(&mut self, key: K) -> bool {
        self.add(key)
    }
    /// Returns an existing key or appends it; `map.try_emplaced(42)`.
    pub fn try_emplaced(&mut self, key: K) -> &K {
        self.added(key)
    }
    /// Unlinks a key while preserving all other positions; `map.remove(&42)`.
    pub fn remove(&mut self, key: &K) -> bool {
        self.table.remove(key)
    }
    /// Frees keys, optionally freeing bucket storage; `map.clear(false)`.
    pub fn clear(&mut self, release: bool) {
        self.table.clear(release);
    }
    /// Borrows the earliest remaining key; `map.first()?`.
    pub fn first(&self) -> Result<&K, Failure> {
        self.edge(self.table.first)
    }
    /// Borrows the latest remaining key; `map.last()?`.
    pub fn last(&self) -> Result<&K, Failure> {
        self.edge(self.table.last)
    }
    /// Swaps storage, order, policies and allocators; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// Borrows keys in insertion order; `map.iter()`.
    pub fn iter(&self) -> OrderedMapIter<'_, K> {
        OrderedMapIter {
            inner: self.table.items(),
        }
    }
    /// Returns an exhausted iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> OrderedMapIter<'_, K> {
        OrderedMapIter::default()
    }
    fn edge(&self, slot: Option<usize>) -> Result<&K, Failure> {
        slot.map(|slot| &self.table.slots[slot].as_ref().unwrap().key)
            .ok_or_else(empty_ordered)
    }
}
impl<K, H: HashPolicy<K> + Default> OrderedMap<K, H> {
    /// Selects initial buckets with default hashing; `OrderedMap::<i32>::new(10)`.
    pub fn new(buckets: usize) -> Self {
        Self::with_hasher(H::default(), buckets)
    }
}
impl<K, H: HashPolicy<K> + Clone> OrderedMap<K, H> {
    /// Moves nodes, leaving source allocator and bucket selection intact; `OrderedMap::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        let empty = Self::with_allocator(
            source.hasher().clone(),
            source.bucket_count(),
            source.allocator().clone(),
        );
        std::mem::replace(source, empty)
    }
    /// Replaces storage with a moved source; `map.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = Self::move_from(source);
    }
}
impl<K, H: HashPolicy<K> + Default> Default for OrderedMap<K, H> {
    fn default() -> Self {
        Self::new(1)
    }
}
impl<K: Clone, H: HashPolicy<K> + Clone> OrderedMap<K, H> {
    /// Copies order retaining destination hash policy and allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        self.table.assign(&source.table, false);
    }
}
impl<K: Clone, H: HashPolicy<K> + Clone> Clone for OrderedMap<K, H> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
        }
    }
}
fn empty_ordered() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "OrderedMap extent 0; expected at least one key",
        "",
    )
}
impl<'a, K> OrderedMapIter<'a, K> {
    /// Checks whether the cursor has a current key; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.inner.current.is_some()
    }
    /// Borrows the current key without advancing; `cursor.key()?`.
    pub fn key(&self) -> Result<&'a K, Failure> {
        self.inner
            .current_pair()
            .map(|(key, ())| key)
            .ok_or_else(empty_ordered)
    }
    /// Borrows the current value, an alias for key; `cursor.value()?`.
    pub fn value(&self) -> Result<&'a K, Failure> {
        self.key()
    }
    /// Advances the legacy cursor; `cursor.advance()`.
    pub fn advance(&mut self) {
        self.inner.advance();
    }
    /// Reinitializes a cursor on another borrowed map; `cursor.initialize(&map)`.
    pub fn initialize<H: HashPolicy<K>>(&mut self, map: &'a OrderedMap<K, H>) {
        self.inner = map.table.items();
    }
}
impl<'a, K> Iterator for OrderedMapIter<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, ())| key)
    }
}
impl<K> Clone for OrderedMapIter<'_, K> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<K> Default for OrderedMapIter<'_, K> {
    fn default() -> Self {
        Self {
            inner: OrderedItems::default(),
        }
    }
}
impl<K> PartialEq for OrderedMapIter<'_, K> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<K> Eq for OrderedMapIter<'_, K> {}
impl<'a, K, H: HashPolicy<K>> IntoIterator for &'a OrderedMap<K, H> {
    type Item = &'a K;
    type IntoIter = OrderedMapIter<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
