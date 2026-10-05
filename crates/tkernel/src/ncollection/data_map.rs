//! Chained hash buckets preserving OCCT growth and head-insertion traversal order.
use super::hash_policy::next_map_prime;
use super::{AllocatedBox, BaseAllocator, DefaultHashPolicy, HashPolicy};
use crate::standard::Failure;
use std::fmt::Debug;
struct Entry<K, V> {
    key: K,
    value: V,
}
type Bucket<K, V> = Vec<AllocatedBox<Entry<K, V>>>;
/// Allocator-owned key/value map; `DataMap::<i32,i32>::default().bind(1,42)`.
pub struct DataMap<K, V, H = DefaultHashPolicy> {
    resource: BaseAllocator,
    hasher: H,
    buckets: Vec<Bucket<K, V>>,
    bucket_count: usize,
    count: usize,
}
/// Shared key/value traversal in bucket order; `map.items()`.
pub struct DataMapItems<'a, K, V> {
    buckets: &'a [Bucket<K, V>],
    bucket: usize,
    index: usize,
}
impl<K: Debug, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// Injects hash/equality state and an initial bucket count; `DataMap::with_hasher(policy,10)`.
    pub fn with_hasher(hasher: H, bucket_count: usize) -> Self {
        Self::with_allocator(hasher, bucket_count, BaseAllocator::common())
    }
    /// Selects the resource for map nodes; `DataMap::with_allocator(policy,1,arena)`.
    pub fn with_allocator(hasher: H, bucket_count: usize, resource: BaseAllocator) -> Self {
        Self {
            resource,
            hasher,
            buckets: Vec::new(),
            // The native constructors keep a zero bucket count until the first insertion resizes.
            bucket_count,
            count: 0,
        }
    }
    /// Returns the injected hash policy; `map.hasher().salt`.
    pub fn hasher(&self) -> &H {
        &self.hasher
    }
    /// Returns allocator identity; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.resource
    }
    /// Returns binding count; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.count
    }
    /// Returns OCCT's binding extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.count
    }
    /// Checks for no bindings; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// Returns the allocated hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.bucket_count
    }
    /// Grows using OCCT's exact prime-step table and rechains nodes; `map.resize(1000)`.
    pub fn resize(&mut self, size: usize) {
        let new_count = next_map_prime(size).max(self.bucket_count);
        if new_count == self.bucket_count && !self.buckets.is_empty() {
            return;
        }
        let mut buckets: Vec<Bucket<K, V>> = (0..=new_count).map(|_| Vec::new()).collect();
        for bucket in self.buckets.drain(..) {
            for node in bucket {
                let hash = self.hasher.hash(&node.key) % new_count + 1;
                buckets[hash].insert(0, node);
            }
        }
        self.buckets = buckets;
        self.bucket_count = new_count;
    }
    /// Checks whether a key is bound; `map.is_bound(&1)`.
    pub fn is_bound(&self, key: &K) -> bool {
        self.lookup(key).is_some()
    }
    /// Borrows a bound value or reports Standard_NoSuchObject; `map.find(&1)?`.
    pub fn find(&self, key: &K) -> Result<&V, Failure> {
        let (bucket, index) = self.lookup(key).ok_or_else(|| missing_key(key))?;
        Ok(&self.buckets[bucket][index].value)
    }
    /// Borrows a bound value exclusively; `*map.change_find(&1)?=42`.
    pub fn change_find(&mut self, key: &K) -> Result<&mut V, Failure> {
        let (bucket, index) = self.lookup(key).ok_or_else(|| missing_key(key))?;
        Ok(&mut self.buckets[bucket][index].value)
    }
    /// Removes a binding, returning whether it existed; `map.unbind(&1)`.
    pub fn unbind(&mut self, key: &K) -> bool {
        let Some((bucket, index)) = self.lookup(key) else {
            return false;
        };
        self.buckets[bucket].remove(index);
        self.count -= 1;
        true
    }
    /// Frees bindings and optionally bucket storage; `map.clear(true)`.
    pub fn clear(&mut self, release: bool) {
        if release {
            self.buckets.clear();
        } else {
            for bucket in &mut self.buckets {
                bucket.clear();
            }
        }
        self.count = 0;
    }
    /// Returns a shared key/value range; `for (key,value) in map.items() {}`.
    pub fn items(&self) -> DataMapItems<'_, K, V> {
        let mut iter = DataMapItems {
            buckets: &self.buckets,
            bucket: 0,
            index: 0,
        };
        iter.seek();
        iter
    }
    /// Borrows values while keeping keys immutable; `map.items_mut().for_each(|(_,v)|*v+=1)`.
    pub fn items_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.buckets
            .iter_mut()
            .flat_map(|bucket| bucket.iter_mut())
            .map(|node| {
                let entry = &mut **node;
                (&entry.key, &mut entry.value)
            })
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
        if self.buckets.is_empty() {
            return None;
        }
        let (bucket, index) = self.lookup_with_bucket(key);
        index.map(|index| (bucket, index))
    }
    fn lookup_with_bucket(&self, key: &K) -> (usize, Option<usize>) {
        let bucket = self.hasher.hash(key) % self.bucket_count + 1;
        let index = self.buckets[bucket]
            .iter()
            .position(|node| self.hasher.equal(&node.key, key));
        (bucket, index)
    }
    fn place(&mut self, key: K, construct: impl FnOnce() -> V, replace: bool) -> (bool, &mut V) {
        if self.is_empty() || self.count > self.bucket_count {
            self.resize(self.count);
        }
        if let Some((bucket, index)) = self.lookup(&key) {
            let value = &mut self.buckets[bucket][index].value;
            if replace {
                *value = construct();
            }
            return (false, value);
        }
        let bucket = self.hasher.hash(&key) % self.bucket_count + 1;
        let node = self.resource.allocate_value(Entry {
            key,
            value: construct(),
        });
        self.buckets[bucket].insert(0, node);
        self.count += 1;
        (true, &mut self.buckets[bucket][0].value)
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
mod keys;
mod queries;
pub use cursor::{DataMapIterator, DataMapIteratorMut};
