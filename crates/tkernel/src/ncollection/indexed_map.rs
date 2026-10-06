//! Stable keys with dense one-based indices and OCCT's swap-last removal.
#![allow(unsafe_code)]
use super::indexed_storage as storage;
use super::{BaseAllocator, DefaultHashPolicy, HashPolicy, IndexedMapNode, MapLinkNode};
use crate::standard::Failure;
use std::fmt::Debug;
use storage::IndexedStorage;
pub use storage::{IndexedMapObservation, IndexedMapObserver};
/// A hash map from unique keys to dense indices; `IndexedMap::<i32>::default().add(42)`.
pub struct IndexedMap<K, H = DefaultHashPolicy> {
    storage: IndexedStorage<IndexedMapNode<K>>,
    hasher: H,
}
/// Shared keys in index order; `map.iter()`.
pub struct IndexedMapIter<'a, K> {
    nodes: &'a [*mut MapLinkNode],
    ownership: std::marker::PhantomData<&'a K>,
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
        // Native constructors retain zero buckets until the first insertion resizes.
        Self {
            storage: IndexedStorage::new(buckets, resource),
            hasher,
        }
    }
    /// Returns key count; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.storage.len()
    }
    /// Returns OCCT's index extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.len()
    }
    /// Checks whether there are no keys; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.storage.bucket_count()
    }
    /// Borrows the retained hash policy; `map.hasher().salt`.
    pub fn hasher(&self) -> &H {
        &self.hasher
    }
    /// Borrows the node allocator; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        self.storage.allocator()
    }
    /// Grows hash storage without relocating keys; `map.resize(200)`.
    pub fn resize(&mut self, requested: usize) {
        self.storage.resize(requested, &self.hasher);
    }
    /// Returns a key's one-based index or zero when absent; `map.find_index(&42)`.
    pub fn find_index(&self, key: &K) -> usize {
        if self.is_empty() {
            return 0;
        }
        self.lookup_with_bucket(key).0
    }
    pub(super) fn lookup_with_bucket(&self, key: &K) -> (usize, usize) {
        self.storage.lookup(key, &self.hasher)
    }
    /// Checks membership through the injected policy; `map.contains(&42)`.
    pub fn contains(&self, key: &K) -> bool {
        self.find_index(key) != 0
    }
    /// Borrows a key by one-based index; `map.find_key(1)?`.
    pub fn find_key(&self, index: usize) -> Result<&K, Failure> {
        self.check_index(index)?;
        Ok(self.storage.key(index))
    }
    /// Inserts a key or returns its existing index; `map.add(42)==1`.
    pub fn add(&mut self, key: K) -> usize {
        self.place(key, false)
    }
    /// Inserts a key and borrows its stable storage; `map.added(42)`.
    pub fn added(&mut self, key: K) -> &K {
        let index = self.place(key, false);
        self.storage.key(index)
    }
    /// Assigns an equivalent key in place (OCCT 3d097a emplaceImpl); `map.emplace(42)==1`.
    pub fn emplace(&mut self, key: K) -> usize {
        self.place(key, true)
    }
    /// Assigns a key and borrows its stable storage (OCCT 3d097a emplaceImpl); `map.emplaced(42)`.
    pub fn emplaced(&mut self, key: K) -> &K {
        let index = self.place(key, true);
        self.storage.key(index)
    }
    /// Frees keys and optionally bucket storage; `map.clear(true)`.
    pub fn clear(&mut self, release: bool) {
        self.storage.clear(release);
    }
    /// Swaps nodes, allocator and hash policy; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.hasher, &mut other.hasher);
        self.storage.exchange(&mut other.storage);
    }
    fn hash_bucket(&self, key: &K) -> usize {
        self.hasher.hash(key) % self.bucket_count() + 1
    }
    fn check_index(&self, index: usize) -> Result<(), Failure> {
        self.storage.check_index(index, "IndexedMap")
    }
    fn place(&mut self, key: K, replace: bool) -> usize {
        self.prepare_add();
        self.place_prepared(key, replace)
    }
    fn prepare_add(&mut self) {
        if self.is_empty() || self.len() > self.bucket_count() {
            self.resize(self.len());
        }
    }
    pub(super) fn append_new_at_bucket(&mut self, key: K, bucket: usize) -> usize {
        self.append_new_at_bucket_with(bucket, || key)
    }
    fn append_new_at_bucket_with(&mut self, bucket: usize, construct: impl FnOnce() -> K) -> usize {
        self.storage.append(bucket, |index, next| {
            IndexedMapNode::construct(index, next, construct)
        })
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Adds a borrowed key without cloning when an equivalent key is already present.
    ///
    /// Native `Add(const Key&)` hashes and searches the borrowed key before copying it on a
    /// miss. Rust follows that behavior for value-semantic keys; allocator-failure effects differ.
    ///
    /// # Example
    /// ```
    /// use rustycad_tkernel::ncollection::IndexedMap;
    /// let mut map = IndexedMap::<i32>::default();
    /// assert_eq!(map.add_ref(&42), 1);
    /// assert_eq!(map.add_ref(&42), 1);
    /// ```
    pub fn add_ref(&mut self, key: &K) -> usize {
        self.prepare_add();
        let (index, bucket) = self.lookup_with_bucket(key);
        if index != 0 {
            return index;
        }
        self.append_new_at_bucket_with(bucket, || key.clone())
    }
    /// Borrows the stored key, copying the argument only on a miss; `map.added_ref(&42)`.
    pub fn added_ref(&mut self, key: &K) -> &K {
        let index = self.add_ref(key);
        self.storage.key(index)
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
mod cursor;
mod iteration;
mod method_contracts;
mod operations;
mod signed_overloads;
pub use cursor::IndexedMapIterator;

impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Observes actual native metadata, including key Drop callbacks; `map.observer().snapshot().size`.
    pub fn observer(&self) -> IndexedMapObserver {
        self.storage.observer()
    }
    fn place_prepared(&mut self, key: K, replace: bool) -> usize {
        let (index, bucket) = self.lookup_with_bucket(&key);
        if index != 0 {
            if replace {
                *self.storage.key_mut(index) = key;
            }
            return index;
        }
        self.append_new_at_bucket(key, bucket)
    }
    /// Constructs after native resize, then assigns an equivalent key; `map.emplace_with(|| 7)`.
    pub fn emplace_with(&mut self, construct: impl FnOnce() -> K) -> usize {
        self.prepare_add();
        self.place_prepared(construct(), true)
    }
    /// Constructs after native resize and keeps an equivalent key; `map.try_emplace_with(|| 7)`.
    pub fn try_emplace_with(&mut self, construct: impl FnOnce() -> K) -> usize {
        self.prepare_add();
        self.place_prepared(construct(), false)
    }
}

impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Borrows the actual native node for extension code; `map.native_node(1)?.index_value()`.
    /// The shared map borrow prevents structural edits or key mutation during inspection.
    pub fn native_node(&self, index: usize) -> Result<&IndexedMapNode<K>, Failure> {
        self.check_index(index)?;
        // SAFETY: the checked slot retains a live node for the complete shared owner borrow.
        Ok(unsafe { &*self.storage.node(index).cast::<IndexedMapNode<K>>() })
    }
}
