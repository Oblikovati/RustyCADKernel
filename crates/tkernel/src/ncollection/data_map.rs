//! Chained hash buckets preserving OCCT growth and head-insertion traversal order.
#![allow(unsafe_code)]
use super::{BaseAllocator, BaseMap, DefaultHashPolicy, HashPolicy, MapLinkNode};
use crate::standard::Failure;
use std::fmt::Debug;
mod storage;
use storage::DataMapStorage;
/// Allocator-owned key/value map; `DataMap::<i32,i32>::default().bind(1,42)`.
pub struct DataMap<K, V, H = DefaultHashPolicy> {
    storage: DataMapStorage<K, V>,
    hasher: H,
}
pub use iteration::DataMapItems;
impl<K: Debug, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// Injects hash/equality state and an initial bucket count; `DataMap::with_hasher(policy,10)`.
    pub fn with_hasher(hasher: H, bucket_count: usize) -> Self {
        Self::with_allocator(hasher, bucket_count, BaseAllocator::common())
    }
    /// Selects the resource for map nodes; `DataMap::with_allocator(policy,1,arena)`.
    pub fn with_allocator(hasher: H, bucket_count: usize, resource: BaseAllocator) -> Self {
        Self {
            storage: DataMapStorage::new(bucket_count, resource),
            hasher,
        }
    }
    /// Returns the injected hash policy; `map.hasher().salt`.
    pub fn hasher(&self) -> &H {
        &self.hasher
    }
    /// Returns allocator identity; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        self.storage.base.allocator()
    }
    /// Returns binding count; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.storage.base.size()
    }
    /// Returns OCCT's binding extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.storage.base.size()
    }
    /// Checks for no bindings; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.storage.base.size() == 0
    }
    /// Returns the allocated hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.storage.base.nb_buckets()
    }
    /// Grows using OCCT's exact prime-step table and rechains nodes; `map.resize(1000)`.
    pub fn resize(&mut self, size: usize) {
        self.storage.resize(size, &self.hasher);
    }
    /// Borrows the real native bucket table; `map.buckets().is_null()` before allocation.
    pub fn buckets(&self) -> *const *mut MapLinkNode {
        self.storage.base.buckets()
    }
    /// Returns native writable Buckets; `map.buckets_mut()`.
    /// Unsafe pointer writes must preserve live acyclic nodes, ownership, hashes and counts.
    pub fn buckets_mut(&mut self) -> *mut *mut MapLinkNode {
        self.storage.base.buckets_mut()
    }
    /// Checks whether a key is bound; `map.is_bound(&1)`.
    pub fn is_bound(&self, key: &K) -> bool {
        self.lookup(key).is_some()
    }
    /// Borrows a bound value or reports Standard_NoSuchObject; `map.find(&1)?`.
    pub fn find(&self, key: &K) -> Result<&V, Failure> {
        let (_, node) = self.lookup(key).ok_or_else(|| missing_key(key))?;
        Ok(unsafe { self.storage.node(node).value() })
    }
    /// Borrows a bound value exclusively; `*map.change_find(&1)?=42`.
    pub fn change_find(&mut self, key: &K) -> Result<&mut V, Failure> {
        let (_, node) = self.lookup(key).ok_or_else(|| missing_key(key))?;
        Ok(unsafe { self.storage.node_mut(node).change_value() })
    }
    /// Removes a binding, returning whether it existed; `map.unbind(&1)`.
    pub fn unbind(&mut self, key: &K) -> bool {
        let Some((bucket, node)) = self.lookup(key) else {
            return false;
        };
        self.storage.remove(bucket, node);
        true
    }
    /// Frees bindings and optionally bucket storage; `map.clear(true)`.
    pub fn clear(&mut self, release: bool) {
        self.storage.clear(release);
    }
    /// Returns a shared key/value range; `for (key,value) in map.items() {}`.
    pub fn items(&self) -> DataMapItems<'_, K, V> {
        DataMapItems::new(&self.storage)
    }
    /// Borrows values while keeping keys immutable; `map.items_mut().for_each(|(_,v)|*v+=1)`.
    pub fn items_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        iteration::DataMapItemsMut::new(&mut self.storage)
    }
    /// Iterates shared values; `map.values().min()`.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.items().map(|(_, value)| value)
    }
    /// Iterates exclusive values; `map.values_mut().for_each(|value|*value+=1)`.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.items_mut().map(|(_, value)| value)
    }
    fn lookup(&self, key: &K) -> Option<(usize, *mut MapLinkNode)> {
        self.storage.lookup(key, &self.hasher)
    }
    fn lookup_with_bucket(&self, key: &K) -> (usize, Option<*mut MapLinkNode>) {
        self.storage.lookup_with_bucket(key, &self.hasher)
    }
    fn place(&mut self, key: K, construct: impl FnOnce() -> V, replace: bool) -> (bool, &mut V) {
        if self.is_empty() || self.storage.base.size() > self.storage.base.nb_buckets() {
            self.resize(self.storage.base.size());
        }
        let (bucket, index) = self.lookup_with_bucket(&key);
        if let Some(index) = index {
            let value = unsafe { self.storage.node_mut(index).change_value() };
            if replace {
                *value = construct();
            }
            return (false, value);
        }
        self.insert_with(bucket, || key, construct)
    }
    fn insert_with(
        &mut self,
        bucket: usize,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> (bool, &mut V) {
        // Native placement new allocates before constructing the value base, then its key.
        let node = self.storage.insert(bucket, construct_key, construct_value);
        (true, unsafe { self.storage.node_mut(node).change_value() })
    }
}
impl<K: Debug, V, H: HashPolicy<K> + Default> Default for DataMap<K, V, H> {
    fn default() -> Self {
        Self::with_hasher(H::default(), 1)
    }
}
fn missing_key<K: Debug>(key: &K) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("DataMap key {key:?}; expected a bound key"),
        "",
    )
}
mod assignment;
mod iteration;
mod operations;

mod cursor;
mod queries;
pub use cursor::{DataMapIterator, DataMapIteratorMut};

mod borrowed_operations;
mod signed_sizes;

mod iterator_source;
