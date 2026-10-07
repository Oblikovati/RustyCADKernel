//! Safe `NCollection_BaseDoubleMap` storage: two bucket tables of slot-index chains over one slot
//! vector. Each slot holds `Key1` as its key and `Key2` as its value (the native node's
//! `TListNode<Key2>` base), so construction is `Key2` then `Key1` and destruction `Key1` then
//! `Key2`, as native. The second table and its `Next2()` links are protected native members
//! (plan D1 class B6); only the first table is public, through `Buckets()`.
use crate::ncollection::{
    BaseAllocator, HashBucketTable,
    base_map::planned_bucket_count,
    map_payload::construct_value_then_key,
    slot_chains::{ChainView, SlotChains, SlotLink, SlotPayload},
};

/// Shared modulus, the two chain tables, the slots and the allocator identity of a DoubleMap.
pub(super) struct DoubleStorage<A, B> {
    allocator: BaseAllocator,
    bucket_count: usize,
    first: SlotChains,
    second: SlotChains,
    payload: SlotPayload<A, B>,
}

impl<A, B> DoubleStorage<A, B> {
    pub(super) fn new(bucket_count: usize, allocator: BaseAllocator) -> Self {
        Self {
            allocator,
            bucket_count,
            first: SlotChains::default(),
            second: SlotChains::default(),
            payload: SlotPayload::default(),
        }
    }
    pub(super) fn len(&self) -> usize {
        self.payload.len()
    }
    pub(super) fn bucket_count(&self) -> usize {
        self.bucket_count
    }
    pub(super) fn allocator(&self) -> &BaseAllocator {
        &self.allocator
    }
    pub(super) fn payload(&self) -> &SlotPayload<A, B> {
        &self.payload
    }
    /// The first table with the slots: native iteration order and the public `Buckets()`.
    pub(super) fn view(&self) -> ChainView<'_, A, B> {
        ChainView::new(&self.first, &self.payload)
    }
    pub(super) fn buckets(&self) -> Option<HashBucketTable<'_, A, B>> {
        HashBucketTable::of(self.view())
    }
    /// Native `Resizable()`: an empty map, or more pairs than buckets.
    pub(super) fn resizable(&self) -> bool {
        self.len() == 0 || self.len() > self.bucket_count
    }
    /// The first `Key1` chain match in `bucket`.
    pub(super) fn find1(&self, bucket: usize, accept: impl Fn(&A) -> bool) -> SlotLink {
        self.first
            .find(bucket, |slot| accept(self.payload.key(slot)))
    }
    /// The first `Key2` chain match in `bucket`, following `Next2()`.
    pub(super) fn find2(&self, bucket: usize, accept: impl Fn(&B) -> bool) -> SlotLink {
        self.second
            .find(bucket, |slot| accept(self.payload.value(slot)))
    }
    /// Native `AreBound` second walk: the second table's head of `bucket`, then `Next()` links
    /// of the first chains (upstream quirk, reproduced exactly).
    pub(super) fn find2_through_first(
        &self,
        bucket: usize,
        accept: impl Fn(&B) -> bool,
    ) -> SlotLink {
        let mut link = self.second.head(bucket);
        while let Some(slot) = link {
            if accept(self.payload.value(slot)) {
                return Some(slot);
            }
            link = self.first.next(slot);
        }
        None
    }
}

impl<A, B> DoubleStorage<A, B> {
    /// Native `ReSize`: both tables grow to the prime step for `extent`; `route` hashes `Key1`
    /// then `Key2` of every pair in first-chain order before any link changes.
    pub(super) fn resize(
        &mut self,
        extent: usize,
        route: impl Fn(&A, &B, usize) -> (usize, usize),
    ) {
        let allocated = self.first.is_allocated();
        let Some(count) = planned_bucket_count(extent, self.bucket_count, allocated) else {
            return;
        };
        let (mut firsts, mut seconds) = (Vec::new(), Vec::new());
        for slot in self.first.order() {
            let (one, two) = route(self.payload.key(slot), self.payload.value(slot), count);
            firsts.push((slot, one));
            seconds.push((slot, two));
        }
        self.first.rebuild(count, &firsts);
        self.second.rebuild(count, &seconds);
        self.bucket_count = count;
    }
    /// Constructs `Key2`, then `Key1`, and prepends the pair to both buckets. A constructor
    /// panic leaves the map unchanged. The caller prepared the tables.
    pub(super) fn insert(
        &mut self,
        (first, second): (usize, usize),
        key1: impl FnOnce() -> A,
        key2: impl FnOnce() -> B,
    ) {
        let (value, key) = construct_value_then_key(key2, key1);
        self.payload.push(key, value);
        self.first.push_front(first);
        self.second.push_front(second);
    }
    /// Unlinks a pair from both chains and returns it for the caller to drop afterwards, so the
    /// map is consistent when the destructor runs (native `UnBind` order).
    pub(super) fn remove(&mut self, slot: usize) -> (A, B) {
        self.first.unlink(slot);
        self.second.unlink(slot);
        self.first.swap_remove(slot);
        self.second.swap_remove(slot);
        self.payload.swap_remove(slot)
    }
    /// Native `destroy`: drops pairs in first-table bucket/chain order; `release` frees both.
    pub(super) fn clear(&mut self, release: bool) {
        let order = self.first.order();
        let payload = std::mem::take(&mut self.payload);
        self.first.reset(release);
        self.second.reset(release);
        payload.drop_in_order(order);
    }
    /// Native `Clear(theAllocator)`: always releases both tables, even for the same allocator;
    /// `None` (a null handle) selects the common allocator.
    pub(super) fn clear_with_allocator(&mut self, requested: Option<BaseAllocator>) {
        self.clear(true);
        self.allocator = requested.unwrap_or_else(BaseAllocator::common);
    }
    /// Native `exchangeDoubleMapsData`: tables, slots, counts and allocator identity.
    pub(super) fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
}

// No API pins a pair in place: growth moves slots, so the map is Unpin whatever A and B are,
// as the boxed native nodes were.
impl<A, B> Unpin for DoubleStorage<A, B> {}

impl<A, B> Drop for DoubleStorage<A, B> {
    fn drop(&mut self) {
        self.clear(true);
    }
}
