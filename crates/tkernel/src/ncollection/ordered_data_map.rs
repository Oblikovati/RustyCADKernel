//! Key/value hash nodes threaded in insertion order with stable entry storage.
use super::ordered_table::{
    OrderedItems, OrderedItemsMut, OrderedNodeMut, OrderedNodeRef, OrderedTable, buckets_from_int,
};
use super::{BaseAllocator, DefaultHashPolicy, HashPolicy};
use crate::standard::Failure;
use std::fmt::Debug;
/// Insertion-ordered key/value bindings; `OrderedDataMap::<i32,i32>::default().bind(1,42)`.
pub struct OrderedDataMap<K, V, H = DefaultHashPolicy> {
    table: OrderedTable<K, V, H>,
}
/// Shared key/value pairs in insertion order; `map.items()`.
pub struct OrderedDataItems<'a, K, V> {
    inner: OrderedItems<'a, K, V>,
}
/// Exclusive values paired with immutable keys; `map.items_mut()`.
pub struct OrderedDataItemsMut<'a, K, V> {
    inner: OrderedItemsMut<'a, K, V>,
}
/// Shared values in insertion order; `map.iter()`.
pub struct OrderedDataValues<'a, K, V> {
    inner: OrderedItems<'a, K, V>,
}
/// Exclusive values in insertion order; `map.iter_mut()`.
pub struct OrderedDataValuesMut<'a, K, V> {
    inner: OrderedItemsMut<'a, K, V>,
}
/// A detached legacy cursor with explicit map reborrowing; `map.cursor().value(&map)?`.
#[derive(Clone, Default)]
pub struct OrderedDataCursor {
    slot: Option<usize>,
    // Identity tokens are never dereferenced: native equality compares nodes, not dense slots.
    owner: *const (),
    node: *const (),
}
/// Native `IsEqual`: two cursors of the same map are equal on the same node, and every
/// exhausted cursor equals every other; `map.cursor() == map.cursor()`.
impl PartialEq for OrderedDataCursor {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}
impl Eq for OrderedDataCursor {}
impl<K, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Injects hash/equality policy and initial buckets; `OrderedDataMap::with_hasher(policy,10)`.
    pub fn with_hasher(hasher: H, buckets: usize) -> Self {
        Self::with_allocator(hasher, buckets, BaseAllocator::common())
    }
    /// Selects allocation for stable key/value nodes; `OrderedDataMap::with_allocator(policy,10,arena)`.
    pub fn with_allocator(hasher: H, buckets: usize, resource: BaseAllocator) -> Self {
        Self {
            table: OrderedTable::new(hasher, buckets, resource),
        }
    }
    /// The signed-count constructors; a negative count raises Standard_OutOfRange;
    /// `OrderedDataMap::with_int_buckets(policy, 10, BaseAllocator::common())?`.
    pub fn with_int_buckets(
        hasher: H,
        buckets: i32,
        resource: BaseAllocator,
    ) -> Result<Self, Failure> {
        Ok(Self::with_allocator(
            hasher,
            buckets_from_int(buckets)?,
            resource,
        ))
    }
    /// `ReSize(int)`: a negative size raises Standard_OutOfRange; `map.resize_int(200)?`.
    pub fn resize_int(&mut self, size: i32) -> Result<(), Failure> {
        self.resize(buckets_from_int(size)?);
        Ok(())
    }
    /// Drops bindings and adopts `resource` (the common allocator for `None`), releasing
    /// bucket storage only when the allocator changes; `map.clear_with_allocator(None)`.
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        self.table.clear_with_allocator(resource);
    }
    /// Counts bindings; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.table.count
    }
    /// Returns OCCT's binding extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.len()
    }
    /// Checks for no bindings; `map.is_empty()`.
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
    /// Rehashes without changing order or node addresses; `map.resize(200)`.
    pub fn resize(&mut self, size: usize) {
        self.table.resize(size);
    }
    /// Tests key membership; `map.is_bound(&42)`.
    pub fn is_bound(&self, key: &K) -> bool {
        self.table.lookup(key).is_some()
    }
    /// Optionally borrows a value; `map.seek(&42)`.
    pub fn seek(&self, key: &K) -> Option<&V> {
        self.contained(key).map(|(_, value)| value)
    }
    /// Optionally borrows a value exclusively; `map.change_seek(&42)`.
    pub fn change_seek(&mut self, key: &K) -> Option<&mut V> {
        self.contained_mut(key).map(|(_, value)| value)
    }
    /// Borrows the stored key/value pair; `map.contained(&42)`.
    pub fn contained(&self, key: &K) -> Option<(&K, &V)> {
        let slot = self.table.lookup(key)?;
        Some((self.table.key(slot), self.table.value(slot)))
    }
    /// Borrows the key immutably and value exclusively; `map.contained_mut(&42)`.
    pub fn contained_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        let slot = self.table.lookup(key)?;
        Some(self.table.pair_mut(slot))
    }
    /// Unlinks a pair while retaining all other positions; `map.unbind(&42)`.
    pub fn unbind(&mut self, key: &K) -> bool {
        self.table.remove(key)
    }
    /// Drops nodes and optionally bucket storage; `map.clear(false)`.
    pub fn clear(&mut self, release: bool) {
        self.table.clear(release);
    }
    /// Swaps order, storage, policies and allocators; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
}
impl<K: Debug, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Finds a value or reports NoSuchObject; `map.find(&42)?`.
    pub fn find(&self, key: &K) -> Result<&V, Failure> {
        self.seek(key).ok_or_else(|| missing_ordered_key(key))
    }
    /// Finds a mutable value or reports NoSuchObject; `*map.change_find(&42)?=7`.
    pub fn change_find(&mut self, key: &K) -> Result<&mut V, Failure> {
        self.change_seek(key)
            .ok_or_else(|| missing_ordered_key(key))
    }
}
impl<K, V, H: HashPolicy<K> + Default> OrderedDataMap<K, V, H> {
    /// Selects initial buckets with default hashing; `OrderedDataMap::<i32,i32>::new(10)`.
    pub fn new(buckets: usize) -> Self {
        Self::with_hasher(H::default(), buckets)
    }
}
impl<K, V, H: HashPolicy<K> + Clone> OrderedDataMap<K, V, H> {
    /// Moves nodes, leaving source allocator and bucket selection intact; `OrderedDataMap::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        let empty = Self::with_allocator(
            source.hasher().clone(),
            source.bucket_count(),
            source.allocator().clone(),
        );
        std::mem::replace(source, empty)
    }
}
impl<K, V, H: HashPolicy<K> + Default> Default for OrderedDataMap<K, V, H> {
    fn default() -> Self {
        Self::new(1)
    }
}
impl<K, V: Clone, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Copies a value to output only when found; `map.find_copy(&42,&mut value)`.
    pub fn find_copy(&self, key: &K, output: &mut V) -> bool {
        let Some(value) = self.seek(key) else {
            return false;
        };
        output.clone_from(value);
        true
    }
}
impl<K: Clone, V: Clone, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Copies order retaining destination policy/allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.table.clear(false);
        if source.table.count == 0 {
            return self;
        }
        self.table.resize(source.table.count - 1);
        for (key, value) in source.table.items() {
            self.table.place_borrowed(key, value, true);
        }
        self
    }
}
impl<K: Clone, V: Clone, H: HashPolicy<K> + Clone> Clone for OrderedDataMap<K, V, H> {
    fn clone(&self) -> Self {
        let mut copy = Self {
            table: OrderedTable::new(
                self.table.hasher.clone(),
                self.table.bucket_count,
                self.table.resource.clone(),
            ),
        };
        copy.assign(self);
        copy
    }
}
impl<K: Debug, V, H: HashPolicy<K>> std::ops::Index<&K> for OrderedDataMap<K, V, H> {
    type Output = V;
    fn index(&self, key: &K) -> &V {
        self.find(key)
            .expect("OrderedDataMap index must be a bound key")
    }
}
impl<K: Debug, V, H: HashPolicy<K>> std::ops::IndexMut<&K> for OrderedDataMap<K, V, H> {
    fn index_mut(&mut self, key: &K) -> &mut V {
        self.change_find(key)
            .expect("OrderedDataMap index must be a bound key")
    }
}
fn missing_ordered_key<K: Debug>(key: &K) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("OrderedDataMap key {key:?}; expected a bound key"),
        "",
    )
}
fn missing_ordered_node(slot: Option<usize>) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("OrderedDataMap slot {slot:?}; expected a live node"),
        "",
    )
}
mod binding;
mod cursor;
mod edges;
mod iteration;
mod method_contracts;

mod mixed_binding;
