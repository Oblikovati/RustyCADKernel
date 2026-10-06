//! Stable allocator-owned hash keys with OCCT's insertion and traversal behavior.
#![allow(unsafe_code)]
use super::{
    BaseAllocator, BaseMap, DefaultHashPolicy, HashPolicy, MapLinkNode, MapNode,
    owned_bucket_chains::{BucketNodeRange, OwnedBucketChains},
};
use std::fmt::Debug;
/// A set of hash keys; `Map::<i32>::default().add(42)`.
pub struct Map<K, H = DefaultHashPolicy> {
    storage: OwnedBucketChains<MapNode<K>>,
    hasher: H,
}
/// Shared keys in OCCT bucket order; `map.iter()`.
pub struct MapIter<'a, K> {
    entries: BucketNodeRange<'a, MapNode<K>>,
}
impl<K: Debug, H: HashPolicy<K>> Map<K, H> {
    /// Creates a map with injected equality/hash state; `Map::with_hasher(policy,10)`.
    pub fn with_hasher(hasher: H, buckets: usize) -> Self {
        Self::with_allocator(hasher, buckets, BaseAllocator::common())
    }
    /// Selects storage for stable key nodes; `Map::with_allocator(policy,10,arena)`.
    pub fn with_allocator(hasher: H, buckets: usize, resource: BaseAllocator) -> Self {
        Self {
            storage: OwnedBucketChains::new(buckets, resource),
            hasher,
        }
    }
    /// Returns the retained hash policy; `map.hasher().salt`.
    pub fn hasher(&self) -> &H {
        &self.hasher
    }
    /// Returns the node allocator; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        self.storage.base.allocator()
    }
    /// Counts unique keys; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.storage.base.size()
    }
    /// Returns OCCT's key extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.len()
    }
    /// Checks for no keys; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.storage.base.size() == 0
    }
    /// Returns the hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.storage.base.nb_buckets()
    }
    /// Grows buckets without relocating keys; `map.resize(200)`.
    pub fn resize(&mut self, size: usize) {
        self.storage.resize(size, &self.hasher);
    }
    /// Tests key membership using the injected equality; `map.contains(&42)`.
    pub fn contains(&self, key: &K) -> bool {
        self.storage.lookup(key, &self.hasher).is_some()
    }
    /// Returns the stored key when found; `map.contained(&42)==Some(&42)`.
    pub fn contained(&self, key: &K) -> Option<&K> {
        self.storage
            .lookup(key, &self.hasher)
            .map(|(_, node)| unsafe { self.storage.node(node).key() })
    }
    /// Adds a key, preserving the existing value on duplicates; `map.add(42)`.
    pub fn add(&mut self, key: K) -> bool {
        self.place_key(key, false).0
    }
    /// Returns the stored key, adding when absent; `map.added(42)`.
    pub fn added(&mut self, key: K) -> &K {
        self.place_key(key, false).1
    }
    /// Replaces an equivalent stored key in place; `map.emplace(42)`.
    pub fn emplace(&mut self, key: K) -> bool {
        self.place_key(key, true).0
    }
    /// Reconstructs and returns a stable stored key; `map.emplaced(42)`.
    pub fn emplaced(&mut self, key: K) -> &K {
        self.place_key(key, true).1
    }
    /// Keeps an existing equivalent key; `map.try_emplace(42)`.
    pub fn try_emplace(&mut self, key: K) -> bool {
        self.add(key)
    }
    /// Returns an existing equivalent key or inserts it; `map.try_emplaced(42)`.
    pub fn try_emplaced(&mut self, key: K) -> &K {
        self.added(key)
    }
    /// Removes an equivalent key, reporting whether found; `map.remove(&42)`.
    pub fn remove(&mut self, key: &K) -> bool {
        let Some((bucket, node)) = self.storage.lookup(key, &self.hasher) else {
            return false;
        };
        self.storage.remove(bucket, node);
        true
    }
    /// Frees keys and optionally buckets; `map.clear(true)`.
    pub fn clear(&mut self, release: bool) {
        self.storage.clear(release);
    }
    /// Swaps nodes, allocator and hash policy; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// Borrows keys in bucket traversal order; `for key in map.iter() {}`.
    pub fn iter(&self) -> MapIter<'_, K> {
        MapIter {
            entries: BucketNodeRange::new(&self.storage),
        }
    }
    /// Returns an exhausted iterator for OCCT iterator comparisons; `map.iter()!=map.end()`.
    pub fn end(&self) -> MapIter<'_, K> {
        MapIter {
            entries: BucketNodeRange::default(),
        }
    }
}
impl<K: Debug, H: HashPolicy<K> + Default> Map<K, H> {
    /// Selects an initial bucket count with the default hasher; `Map::<i32>::new(101)`.
    pub fn new(buckets: usize) -> Self {
        Self::with_hasher(H::default(), buckets)
    }
}
impl<K: Debug, H: HashPolicy<K> + Default> Default for Map<K, H> {
    fn default() -> Self {
        Self::new(1)
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> Map<K, H> {
    /// Copies keys while retaining destination allocator and hasher; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        self.assign_returning(source);
    }
}
impl<K: Debug + Clone, H: HashPolicy<K> + Clone> Clone for Map<K, H> {
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(
            self.hasher.clone(),
            self.bucket_count(),
            self.allocator().clone(),
        );
        copy.assign(self);
        copy
    }
}
impl<'a, K: Debug, H: HashPolicy<K>> IntoIterator for &'a Map<K, H> {
    type Item = &'a K;
    type IntoIter = MapIter<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, K> Iterator for MapIter<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next().map(MapNode::key)
    }
}
impl<K> Clone for MapIter<'_, K> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}
impl<K> PartialEq for MapIter<'_, K> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl<K> Eq for MapIter<'_, K> {}

mod borrowed;
mod cursor;
pub use cursor::MapIterator;

mod legacy_algebra;
mod method_contracts;

mod storage_methods;
