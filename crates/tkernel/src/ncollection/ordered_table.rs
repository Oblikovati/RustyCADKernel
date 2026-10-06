//! Shared hash chains and insertion-order links over stable allocator-owned nodes.
use super::hash_policy::next_map_prime;
use super::{AllocatedBox, BaseAllocator, HashPolicy};
pub(super) struct OrderedNode<K, V> {
    pub key: K,
    pub value: V,
    previous: Option<usize>,
    pub next: Option<usize>,
}
pub(super) type OrderedSlots<K, V> = Vec<Option<AllocatedBox<OrderedNode<K, V>>>>;
pub(super) struct OrderedTable<K, V, H> {
    pub slots: OrderedSlots<K, V>,
    free: Vec<usize>,
    buckets: Vec<Vec<usize>>,
    pub count: usize,
    pub bucket_count: usize,
    pub first: Option<usize>,
    pub last: Option<usize>,
    pub resource: BaseAllocator,
    pub hasher: H,
}
pub(super) struct OrderedItems<'a, K, V> {
    pub slots: &'a [Option<AllocatedBox<OrderedNode<K, V>>>],
    pub current: Option<usize>,
    // The map's head when the iterator was created: native `Reset()` restarts there.
    pub first: Option<usize>,
}
impl<K, V, H: HashPolicy<K>> OrderedTable<K, V, H> {
    pub fn new(hasher: H, buckets: usize, resource: BaseAllocator) -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            buckets: Vec::new(),
            count: 0,
            // NCollection_BaseMap keeps a requested zero; the first insertion resizes to a prime.
            bucket_count: buckets,
            first: None,
            last: None,
            resource,
            hasher,
        }
    }
    pub fn resize(&mut self, requested: usize) {
        let count = next_map_prime(requested).max(self.bucket_count);
        if count == self.bucket_count && !self.buckets.is_empty() {
            return;
        }
        let mut buckets: Vec<Vec<usize>> = (0..=count).map(|_| Vec::new()).collect();
        for &slot in self.buckets.iter().flatten() {
            let hash = self.hasher.hash(&self.slots[slot].as_ref().unwrap().key) % count + 1;
            buckets[hash].insert(0, slot);
        }
        self.buckets = buckets;
        self.bucket_count = count;
    }
    pub fn lookup(&self, key: &K) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        let hash = self.hasher.hash(key) % self.bucket_count + 1;
        self.lookup_at_hash(key, hash)
    }
    fn lookup_at_hash(&self, key: &K, hash: usize) -> Option<usize> {
        self.buckets[hash].iter().copied().find(|&slot| {
            self.hasher
                .equal(&self.slots[slot].as_ref().unwrap().key, key)
        })
    }
    fn append_allocated(&mut self, node: AllocatedBox<OrderedNode<K, V>>, hash: usize) -> usize {
        let slot = self.free.pop().unwrap_or_else(|| {
            self.slots.push(None);
            self.slots.len() - 1
        });
        self.slots[slot] = Some(node);
        if let Some(last) = self.last {
            self.slots[last].as_mut().unwrap().next = Some(slot);
        } else {
            self.first = Some(slot);
        }
        self.last = Some(slot);
        self.buckets[hash].insert(0, slot);
        self.count += 1;
        slot
    }
    pub fn remove(&mut self, key: &K) -> bool {
        // OCCT 3d097a03 hashes once before mutation: user hash callbacks can throw.
        if self.count == 0 {
            return false;
        }
        let hash = self.hasher.hash(key) % self.bucket_count + 1;
        let Some(slot) = self.lookup_at_hash(key, hash) else {
            return false;
        };
        let node = self.slots[slot].take().unwrap();
        self.count -= 1;
        self.buckets[hash].retain(|&entry| entry != slot);
        self.unlink_order(slot, &node);
        true
    }
    /// Native `Clear(theAllocator)`: release bucket storage only when the allocator changes,
    /// then adopt `resource`, or the common allocator for a null handle.
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        self.clear(resource.as_ref() != Some(&self.resource));
        self.resource = resource.unwrap_or_else(BaseAllocator::common);
    }
    pub fn items(&self) -> OrderedItems<'_, K, V> {
        OrderedItems {
            slots: &self.slots,
            current: self.first,
            first: self.first,
        }
    }
}
impl<K: Clone, V: Clone, H: HashPolicy<K>> OrderedTable<K, V, H> {
    pub fn assign(&mut self, source: &Self, replace_values: bool) {
        self.clear(false);
        if source.count == 0 {
            return;
        }
        self.resize(source.count - 1);
        for (key, value) in source.items() {
            self.place_borrowed(key, value, replace_values);
        }
    }
}
impl<K: Clone, V: Clone, H: HashPolicy<K> + Clone> Clone for OrderedTable<K, V, H> {
    fn clone(&self) -> Self {
        let mut copy = Self::new(
            self.hasher.clone(),
            self.bucket_count,
            self.resource.clone(),
        );
        copy.assign(self, false);
        copy
    }
}
/// `NCollection_BaseMap::nbBucketsFromInt`: a negative signed bucket count raises
/// Standard_OutOfRange; `buckets_from_int(-1)` is an error naming -1.
pub(super) fn buckets_from_int(buckets: i32) -> Result<usize, crate::standard::Failure> {
    usize::try_from(buckets).map_err(|_| {
        crate::standard::Failure::with_type(
            "Standard_OutOfRange",
            &format!("bucket count {buckets}; expected a non-negative count"),
            "",
        )
    })
}
mod iteration;
mod nodes;
pub use nodes::{OrderedNodeMut, OrderedNodeRef};

mod borrowed;
mod mutable;
pub(super) use mutable::OrderedItemsMut;

mod destruction;

mod placement;
