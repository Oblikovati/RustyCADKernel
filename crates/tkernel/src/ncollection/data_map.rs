//! Chained hash buckets preserving OCCT growth and head-insertion traversal order.
use super::{
    BaseAllocator, BaseMap, DefaultHashPolicy, HashBucketTable, HashPolicy,
    chained_slots::ChainedSlots,
};
use crate::standard::Failure;
use std::fmt::Debug;
type DataMapStorage<K, V> = ChainedSlots<K, V>;
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
        self.storage.allocator()
    }
    /// Returns binding count; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.storage.size()
    }
    /// Returns OCCT's binding extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.storage.size()
    }
    /// Checks for no bindings; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.storage.size() == 0
    }
    /// Returns the allocated hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.storage.nb_buckets()
    }
    /// Grows using OCCT's exact prime-step table and rechains nodes; `map.resize(1000)`.
    pub fn resize(&mut self, size: usize) {
        self.storage.resize(size, &self.hasher);
    }
    /// Native `Buckets()`: the allocated table, or `None` where native returns null.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// assert!(map.buckets().is_none());
    /// map.bind(1, 10);
    /// assert_eq!(map.buckets().map(|table| table.len()), Some(102));
    /// ```
    pub fn buckets(&self) -> Option<HashBucketTable<'_, K, V>> {
        self.storage.buckets()
    }
    /// Checks whether a key is bound; `map.is_bound(&1)`.
    pub fn is_bound(&self, key: &K) -> bool {
        self.lookup(key).is_some()
    }
    /// Borrows a bound value or reports Standard_NoSuchObject; `map.find(&1)?`.
    pub fn find(&self, key: &K) -> Result<&V, Failure> {
        let (_, slot) = self.lookup(key).ok_or_else(|| missing_key(key))?;
        Ok(self.storage.slot(slot).value())
    }
    /// Borrows a bound value exclusively; `*map.change_find(&1)?=42`.
    pub fn change_find(&mut self, key: &K) -> Result<&mut V, Failure> {
        let (_, slot) = self.lookup(key).ok_or_else(|| missing_key(key))?;
        Ok(self.storage.slot_mut(slot).key_value_mut().1)
    }
    /// Removes a binding, returning whether it existed; `map.unbind(&1)`.
    pub fn unbind(&mut self, key: &K) -> bool {
        let Some((_, slot)) = self.lookup(key) else {
            return false;
        };
        // Native UnBind unlinks and counts down before the node destructor runs.
        drop(self.storage.remove(slot));
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
        self.storage.range_mut()
    }
    /// Iterates shared values; `map.values().min()`.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.items().map(|(_, value)| value)
    }
    /// Iterates exclusive values; `map.values_mut().for_each(|value|*value+=1)`.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.items_mut().map(|(_, value)| value)
    }
    fn lookup(&self, key: &K) -> Option<(usize, usize)> {
        self.storage.lookup(key, &self.hasher)
    }
    fn lookup_with_bucket(&self, key: &K) -> (usize, Option<usize>) {
        self.storage.lookup_with_bucket(key, &self.hasher)
    }
    fn place(&mut self, key: K, construct: impl FnOnce() -> V, replace: bool) -> (bool, &mut V) {
        if self.is_empty() || self.storage.size() > self.storage.nb_buckets() {
            self.resize(self.storage.size());
        }
        let (bucket, slot) = self.lookup_with_bucket(&key);
        match slot {
            Some(slot) => (false, self.update_slot(slot, construct, replace)),
            None => self.insert_with(bucket, || key, construct),
        }
    }
    // Native emplaceImpl on a hit: assign a newly constructed value unless it is a Try variant.
    fn update_slot(&mut self, slot: usize, construct: impl FnOnce() -> V, replace: bool) -> &mut V {
        let value = self.storage.slot_mut(slot).key_value_mut().1;
        if replace {
            *value = construct();
        }
        value
    }
    fn insert_with(
        &mut self,
        bucket: usize,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> (bool, &mut V) {
        // Native placement new constructs the value base, then its key.
        let slot = self.storage.insert(bucket, construct_key, construct_value);
        (true, self.storage.slot_mut(slot).key_value_mut().1)
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
