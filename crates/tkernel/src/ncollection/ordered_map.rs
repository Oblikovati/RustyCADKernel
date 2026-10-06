//! Hash keys threaded in insertion order, with stable node identity.
use super::ordered_table::{OrderedItems, OrderedNodeRef, OrderedTable, buckets_from_int};
use super::{BaseAllocator, DefaultHashPolicy, HashPolicy};
use crate::standard::Failure;
/// An insertion-ordered set; `OrderedMap::<i32>::default().add(42)`.
pub struct OrderedMap<K, H = DefaultHashPolicy> {
    table: OrderedTable<K, (), H, super::OrderedMapNode<K>>,
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
    /// The signed-count constructors; a negative count raises Standard_OutOfRange;
    /// `OrderedMap::with_int_buckets(policy, 10, BaseAllocator::common())?`.
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
    /// Drops keys and adopts `resource` (the common allocator for `None`), releasing bucket
    /// storage only when the allocator changes; `map.clear_with_allocator(Some(arena))`.
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        self.table.clear_with_allocator(resource);
    }
    /// The head of the insertion order as a node view, `None` when empty; `map.first_node()`.
    pub fn first_node(&self) -> Option<OrderedNodeRef<'_, K, ()>> {
        self.table.first_node()
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
        Some(self.table.key(self.table.lookup(key)?))
    }
    /// Appends a new key without changing duplicate order; `map.add(42)`.
    pub fn add(&mut self, key: K) -> bool {
        self.table.place(key, || (), false, false).0
    }
    /// Borrows a stored key, appending only when absent; `map.added(42)`.
    pub fn added(&mut self, key: K) -> &K {
        let (_, slot) = self.table.place(key, || (), false, false);
        self.table.key(slot)
    }
    /// Replaces an equivalent key without changing its position; `map.emplace(42)`.
    pub fn emplace(&mut self, key: K) -> bool {
        self.table.place(key, || (), true, false).0
    }
    /// Reconstructs and returns stable key storage; `map.emplaced(42)`.
    pub fn emplaced(&mut self, key: K) -> &K {
        let (_, slot) = self.table.place(key, || (), true, false);
        self.table.key(slot)
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
    /// Swaps both maps and returns the destination, matching move assignment; `map.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) -> &mut Self {
        self.exchange(source);
        self
    }
    /// Preserves copy/move self-assignment safely; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
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
        slot.map(|slot| self.table.key(slot))
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
}
impl<K, H: HashPolicy<K> + Default> Default for OrderedMap<K, H> {
    fn default() -> Self {
        Self::new(1)
    }
}
impl<K: Clone, H: HashPolicy<K>> OrderedMap<K, H> {
    /// Copies order retaining destination hash policy and allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.table.assign(&source.table, false);
        self
    }
    /// Copies a key only when absent, preserving duplicate identity; `map.add_ref(&42)`.
    pub fn add_ref(&mut self, key: &K) -> bool {
        self.table.place_borrowed(key, &(), false).0
    }
    /// Returns the stored key, copying only on a miss; `map.added_ref(&42)`.
    pub fn added_ref(&mut self, key: &K) -> &K {
        let (_, slot) = self.table.place_borrowed(key, &(), false);
        self.table.key(slot)
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
    /// Restarts at the map's first key, or stays exhausted for a default cursor; `cursor.reset()`.
    pub fn reset(&mut self) {
        self.inner.restart();
    }
    /// Reinitializes a cursor on another borrowed map; `cursor.initialize(&map)`.
    pub fn initialize<H: HashPolicy<K>>(&mut self, map: &'a OrderedMap<K, H>) {
        self.inner = map.table.items();
    }
    /// Compares native current-node identity; `cursor.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }
    /// Advances and returns this iterator, as prefix increment does; `cursor.advance_returning()`.
    pub fn advance_returning(&mut self) -> &mut Self {
        self.advance();
        self
    }
    /// Returns the current state and then advances, as postfix increment does; `cursor.post_advance()`.
    pub fn post_advance(&mut self) -> Self {
        let previous = self.clone();
        self.advance();
        previous
    }
    /// Copies iterator state and returns the destination; `cursor.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        self.clone_from(other);
        self
    }
    /// Models iterator self-assignment without aliasing references; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
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
