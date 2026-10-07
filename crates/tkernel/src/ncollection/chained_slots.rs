//! Safe `NCollection_BaseMap` bucket chains for DataMap and Map: slot indices replace node pointers.
//!
//! The native map keeps a table of `NbBuckets + 1` list heads, and each node links to the next
//! node of its bucket. Here the nodes live in one `Vec` and every link is a slot index (see
//! `slot_chains`), so the observable contracts stay native: the bucket count and its prime
//! steps, head insertion inside a bucket, traversal (bucket `0..=NbBuckets`, then each chain),
//! rehash and removal order, a null table before the first insertion or after `Clear(true)`, and
//! a retained table after `Clear(false)`. Node addresses and the per-node allocator calls are not
//! observable through the public API (plan D1 classes B5 and B6), so the map keeps only the
//! allocator identity.
use super::{
    BaseAllocator, BaseMap, HashBucketTable,
    slot_chains::{ChainSlot, ChainView, SlotChains, SlotRangeMut},
};
mod lookup;
mod resize;
mod topology;

/// Bucket heads, slots and allocator identity of one hashed map.
pub(super) struct ChainedSlots<K, V> {
    allocator: BaseAllocator,
    bucket_count: usize,
    chains: SlotChains,
    slots: Vec<ChainSlot<K, V>>,
}

impl<K, V> ChainedSlots<K, V> {
    /// Native constructors keep even a zero bucket count until the first insertion resizes.
    pub(super) fn new(bucket_count: usize, allocator: BaseAllocator) -> Self {
        Self {
            allocator,
            bucket_count,
            chains: SlotChains::default(),
            slots: Vec::new(),
        }
    }
    pub(super) fn slot(&self, index: usize) -> &ChainSlot<K, V> {
        &self.slots[index]
    }
    pub(super) fn slot_mut(&mut self, index: usize) -> &mut ChainSlot<K, V> {
        &mut self.slots[index]
    }
    /// The bucket chains, for cursors that follow native traversal order.
    pub(super) fn chains(&self) -> &SlotChains {
        &self.chains
    }
    /// The table and slots together, for shared ranges.
    pub(super) fn view(&self) -> ChainView<'_, K, V> {
        ChainView::new(&self.chains, &self.slots)
    }
    /// Exclusive traversal in native order, lending each value once.
    pub(super) fn range_mut(&mut self) -> SlotRangeMut<'_, K, V> {
        SlotRangeMut::new(self.chains.order(), &mut self.slots)
    }
    /// The allocated table, or `None` where native `Buckets()` returns null.
    pub(super) fn buckets(&self) -> Option<HashBucketTable<'_, K, V>> {
        HashBucketTable::of(self.view())
    }
    /// Native `Clear(const Handle(NCollection_BaseAllocator)&)`: a new allocator releases the
    /// table; `None` (a null handle) selects the common allocator.
    pub(super) fn clear_with_allocator(&mut self, requested: Option<BaseAllocator>) {
        let release = requested.as_ref() != Some(&self.allocator);
        self.clear(release);
        if release {
            self.allocator = requested.unwrap_or_else(BaseAllocator::common);
        }
    }
}

impl<K, V> BaseMap for ChainedSlots<K, V> {
    fn nb_buckets(&self) -> usize {
        self.bucket_count
    }
    fn size(&self) -> usize {
        self.slots.len()
    }
    fn allocator(&self) -> &BaseAllocator {
        &self.allocator
    }
}

// No API pins an entry in place: growth moves slots, so the map is Unpin whatever K and V are,
// as the boxed native nodes were.
impl<K, V> Unpin for ChainedSlots<K, V> {}

impl<K, V> Drop for ChainedSlots<K, V> {
    fn drop(&mut self) {
        // Native ~NCollection_DataMap destroys nodes in bucket/chain order, not slot order.
        self.clear(true);
    }
}
