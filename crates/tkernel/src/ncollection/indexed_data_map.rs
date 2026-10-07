//! Key/value bindings with dense one-based indices over the safe slots IndexedMap uses.
use super::{BaseAllocator, BaseMap, DefaultHashPolicy, HashPolicy, chained_slots::ChainedSlots};
use crate::standard::Failure;
use std::fmt::Debug;
/// Key/value bindings addressed by key or dense one-based index; `IndexedDataMap::<i32,i32>::default().add(1,42)`.
pub struct IndexedDataMap<K, V, H = DefaultHashPolicy> {
    storage: ChainedSlots<K, V>,
    hasher: H,
}
/// Shared values in index order; `map.iter()`.
pub struct IndexedDataValues<'a, V> {
    items: &'a [V],
    position: usize,
}
/// Exclusive values in index order; `map.iter_mut()`.
pub struct IndexedDataValuesMut<'a, V> {
    items: std::slice::IterMut<'a, V>,
}
/// Shared key/value pairs in index order; `map.items()`.
pub struct IndexedDataItems<'a, K, V> {
    keys: IndexedDataValues<'a, K>,
    values: IndexedDataValues<'a, V>,
}
impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Selects a hash policy and initial bucket count; `IndexedDataMap::with_hasher(policy,10)`.
    pub fn with_hasher(hasher: H, buckets: usize) -> Self {
        Self::with_allocator(hasher, buckets, BaseAllocator::common())
    }
    /// Selects storage for keys and values; `IndexedDataMap::with_allocator(policy,10,arena)`.
    pub fn with_allocator(hasher: H, buckets: usize, resource: BaseAllocator) -> Self {
        Self {
            storage: ChainedSlots::new(buckets, resource),
            hasher,
        }
    }
    /// Returns binding count; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.storage.size()
    }
    /// Returns OCCT's dense index extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.len()
    }
    /// Checks for no bindings; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the retained hash policy; `map.hasher().salt`.
    pub fn hasher(&self) -> &H {
        &self.hasher
    }
    /// Returns the map allocator; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        self.storage.allocator()
    }
    /// Returns the hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.storage.nb_buckets()
    }
    /// Grows bucket storage and rechains entries in native bucket order; `map.resize(200)`.
    pub fn resize(&mut self, size: usize) {
        self.storage.resize(size, &self.hasher);
    }
    /// Returns a key's one-based index or zero; `map.find_index(&42)`.
    pub fn find_index(&self, key: &K) -> usize {
        if self.is_empty() {
            return 0;
        }
        self.storage.find_index(key, &self.hasher).0
    }
    /// Checks membership using the injected policy; `map.contains(&42)`.
    pub fn contains(&self, key: &K) -> bool {
        self.find_index(key) != 0
    }
    /// Borrows a key by index; `map.find_key(1)?`.
    pub fn find_key(&self, index: usize) -> Result<&K, Failure> {
        self.check_index(index)?;
        Ok(self.storage.key_at(index))
    }
    /// Borrows a value by index; `map.find_from_index(1)?`.
    pub fn find_from_index(&self, index: usize) -> Result<&V, Failure> {
        self.check_index(index)?;
        Ok(self.storage.value_at(index))
    }
    /// Borrows a value exclusively by index; `*map.change_from_index(1)?=42`.
    pub fn change_from_index(&mut self, index: usize) -> Result<&mut V, Failure> {
        self.check_index(index)?;
        Ok(self.storage.value_at_mut(index))
    }
    /// Finds a value by key or reports NoSuchObject; `map.find_from_key(&42)?`.
    pub fn find_from_key(&self, key: &K) -> Result<&V, Failure> {
        self.seek(key).ok_or_else(|| missing_binding(key))
    }
    /// Finds an exclusive value by key; `*map.change_from_key(&42)?=7`.
    pub fn change_from_key(&mut self, key: &K) -> Result<&mut V, Failure> {
        self.change_seek(key).ok_or_else(|| missing_binding(key))
    }
    /// Returns an optional borrowed value; `map.seek(&42)`.
    pub fn seek(&self, key: &K) -> Option<&V> {
        let index = self.find_index(key);
        (index != 0).then(|| self.storage.value_at(index))
    }
    /// Returns an optional exclusive value; `map.change_seek(&42)`.
    pub fn change_seek(&mut self, key: &K) -> Option<&mut V> {
        let index = self.find_index(key);
        (index != 0).then(|| self.storage.value_at_mut(index))
    }
    /// Drops entries and optionally bucket storage; `map.clear(true)`.
    pub fn clear(&mut self, release: bool) {
        self.storage.clear(release);
    }
    /// Swaps entries, policies and allocators; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.hasher, &mut other.hasher);
        std::mem::swap(&mut self.storage, &mut other.storage);
    }
}
impl<K: Debug, V, H: HashPolicy<K> + Default> IndexedDataMap<K, V, H> {
    /// Selects initial buckets with a default hash policy; `IndexedDataMap::<i32,i32>::new(3)`.
    pub fn new(buckets: usize) -> Self {
        Self::with_hasher(H::default(), buckets)
    }
}
impl<K: Debug, V, H: HashPolicy<K> + Default> Default for IndexedDataMap<K, V, H> {
    fn default() -> Self {
        Self::new(1)
    }
}
impl<K: Debug, V, H: HashPolicy<K>> std::ops::Index<usize> for IndexedDataMap<K, V, H> {
    type Output = V;
    fn index(&self, index: usize) -> &V {
        self.find_from_index(index)
            .expect("IndexedDataMap index must be in 1..=len")
    }
}
impl<K: Debug, V, H: HashPolicy<K>> std::ops::IndexMut<usize> for IndexedDataMap<K, V, H> {
    fn index_mut(&mut self, index: usize) -> &mut V {
        self.change_from_index(index)
            .expect("IndexedDataMap index must be in 1..=len")
    }
}
fn missing_binding<K: Debug>(key: &K) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("IndexedDataMap key {key:?}; expected a bound key"),
        "",
    )
}
mod binding;
mod cursor;
mod iteration;
mod method_contracts;
mod operations;
mod signed_overloads;
pub use cursor::{IndexedDataMapIterator, IndexedDataMapIteratorMut};

impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    fn check_index(&self, index: usize) -> Result<(), Failure> {
        self.storage.check_index(index, "IndexedDataMap")
    }
}
