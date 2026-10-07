//! Shared storage of `NCollection_OrderedMap` and `NCollection_OrderedDataMap` over safe slots.
//!
//! The bucket chains and payload are the DataMap slot storage (`chained_slots`); beside it every
//! slot keeps its insertion-order links (`myOrderPrev`/`myOrderNext` as slot indices) and an
//! entry serial. Removal moves the last slot into the hole, so the one entry that moved has its
//! order neighbours, the head or tail and its serial index repointed. The serial is the native
//! node identity a detached cursor holds: it is unique in the process and never reused, as a
//! live node address is unique, so a cursor never matches an entry of another map. Class-A
//! behaviour stays native: insertion order, bucket count and prime steps, head insertion in the
//! chains, rehash in old bucket/chain order, Remove counting down and unlinking before the
//! destructor, and Clear dropping in bucket/chain order.
use super::{
    BaseAllocator, BaseMap, HashPolicy, chained_slots::ChainedSlots, slot_chains::SlotLink,
};
use std::collections::HashMap;
mod iteration;
mod links;
mod nodes;
mod placement;
pub(super) use iteration::{OrderedItems, OrderedItemsMut};
use links::OrderSlot;
pub use nodes::{OrderedNodeMut, OrderedNodeRef};

/// The ordered maps' table: policy, slot storage, order links and entry identities.
pub(super) struct OrderedTable<K, V, H> {
    pub hasher: H,
    storage: ChainedSlots<K, V>,
    order: Vec<OrderSlot>,
    serials: HashMap<u64, usize>,
    first: SlotLink,
    last: SlotLink,
}

impl<K, V, H> OrderedTable<K, V, H> {
    pub fn new(hasher: H, buckets: usize, resource: BaseAllocator) -> Self {
        Self::new_with_policy(|| hasher, buckets, resource)
    }
    /// Native constructors build the empty base before the policy (OCCT 3d097a0).
    pub fn new_with_policy(
        make_policy: impl FnOnce() -> H,
        buckets: usize,
        resource: BaseAllocator,
    ) -> Self {
        let storage = ChainedSlots::new(buckets, resource);
        Self {
            hasher: make_policy(),
            storage,
            order: Vec::new(),
            serials: HashMap::new(),
            first: None,
            last: None,
        }
    }
    pub fn len(&self) -> usize {
        self.storage.size()
    }
    pub fn bucket_count(&self) -> usize {
        self.storage.nb_buckets()
    }
    pub fn resource(&self) -> &BaseAllocator {
        self.storage.allocator()
    }
    /// The slot of the earliest and of the latest remaining entry.
    pub fn first(&self) -> SlotLink {
        self.first
    }
    pub fn last(&self) -> SlotLink {
        self.last
    }
    pub fn key(&self, slot: usize) -> &K {
        self.storage.payload().key(slot)
    }
    pub fn value(&self, slot: usize) -> &V {
        self.storage.payload().value(slot)
    }
    pub fn value_mut(&mut self, slot: usize) -> &mut V {
        self.storage.payload_mut().value_mut(slot)
    }
    pub fn pair_mut(&mut self, slot: usize) -> (&K, &mut V) {
        self.storage.payload_mut().entry_mut(slot)
    }
    /// Native `OrderNext()`: the slot inserted after `slot`, if any remains.
    pub fn next_slot(&self, slot: usize) -> SlotLink {
        self.order[slot].next
    }
    /// The never-reused identity of the entry in `slot`, the native node address a cursor holds.
    pub fn serial(&self, slot: usize) -> u64 {
        self.order[slot].serial
    }
    /// The slot of a live entry identity, or `None` once that entry was removed.
    pub fn slot_of(&self, serial: u64) -> SlotLink {
        self.serials.get(&serial).copied()
    }
    /// Native `Buckets()` view of the chains (protected order links stay private).
    pub fn buckets(&self) -> Option<crate::ncollection::HashBucketTable<'_, K, V>> {
        self.storage.buckets()
    }
    /// Native `lookup(key, node)`: an empty map skips the hasher.
    pub fn lookup(&self, key: &K) -> SlotLink
    where
        H: HashPolicy<K>,
    {
        self.storage.lookup(key, &self.hasher).map(|(_, slot)| slot)
    }
    /// Native `Remove`: count down and unlink the chains and the order list, then destroy.
    pub fn remove(&mut self, key: &K) -> bool
    where
        H: HashPolicy<K>,
    {
        let Some(slot) = self.lookup(key) else {
            return false;
        };
        drop(self.remove_slot(slot));
        true
    }
    /// Rehashes the chains; insertion order and entry identities do not change.
    pub fn resize(&mut self, requested: usize)
    where
        H: HashPolicy<K>,
    {
        self.storage.resize(requested, &self.hasher);
    }
    /// Native `Clear`: drops entries in bucket/chain order; `release` frees the table.
    pub fn clear(&mut self, release: bool) {
        self.reset_order();
        self.storage.clear(release);
    }
    /// Native `Clear(theAllocator)`: a new allocator releases the table; `None` selects the
    /// common allocator after the entries are dropped.
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        self.clear_with_borrowed_allocator(resource.as_ref());
    }
    /// Swaps every graph part but the policies, as native `exchangeMapsData` plus the order heads.
    pub fn exchange_graph(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.storage, &mut other.storage);
        std::mem::swap(&mut self.order, &mut other.order);
        std::mem::swap(&mut self.serials, &mut other.serials);
        std::mem::swap(&mut self.first, &mut other.first);
        std::mem::swap(&mut self.last, &mut other.last);
    }
    // Identities stay unique across a clear: a cursor taken before it must not match later entries.
    fn reset_order(&mut self) {
        self.order.clear();
        self.serials.clear();
        self.first = None;
        self.last = None;
    }
}

impl<K: Clone, V: Clone, H: HashPolicy<K>> OrderedTable<K, V, H> {
    /// Native `Assign`: clear, `ReSize(Extent() - 1)`, then add every source entry in order;
    /// OrderedDataMap binds (`replace_values`), OrderedMap adds.
    pub fn assign(&mut self, source: &Self, replace_values: bool) {
        self.clear(false);
        if source.len() == 0 {
            return;
        }
        self.resize(source.len() - 1);
        for (key, value) in source.items() {
            self.place_borrowed(key, value, replace_values);
        }
    }
}

impl<K: Clone, V: Clone, H: HashPolicy<K> + Clone> OrderedTable<K, V, H> {
    /// Native copy constructor: the source modulus, allocator and policy, then `Assign`.
    pub fn clone_with(&self, replace_values: bool) -> Self {
        let mut copy = Self::new_with_policy(
            || self.hasher.clone(),
            self.bucket_count(),
            self.resource().clone(),
        );
        copy.assign(self, replace_values);
        copy
    }
}

impl<K: Clone, V: Clone, H: HashPolicy<K> + Clone> Clone for OrderedTable<K, V, H> {
    fn clone(&self) -> Self {
        self.clone_with(false)
    }
}

impl<K, V, H> Drop for OrderedTable<K, V, H> {
    fn drop(&mut self) {
        // Native ~NCollection_OrderedMap clears its nodes before its hasher member is destroyed.
        self.clear(true);
    }
}

/// Signed native bucket count validation; `buckets_from_int(-1)` names the invalid value.
pub(super) fn buckets_from_int(buckets: i32) -> Result<usize, crate::standard::Failure> {
    usize::try_from(buckets).map_err(|_| {
        crate::standard::Failure::with_type(
            "Standard_OutOfRange",
            &format!("bucket count {buckets}; expected a non-negative count"),
            "",
        )
    })
}

mod facade;
pub(super) use facade::{
    ordered_base_bridge, ordered_borrowed_clear, ordered_default_construction,
    ordered_policy_construction, ordered_policy_exchange,
};

#[cfg(test)]
mod canonical_contracts;
