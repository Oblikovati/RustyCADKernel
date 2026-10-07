//! Head insertion, removal and destruction keep the native chain and destructor order.
use super::{ChainSlot, ChainedSlots, SlotLink};
use crate::ncollection::map_payload::construct_value_then_key;

impl<K, V> ChainedSlots<K, V> {
    /// Constructs the value, then the key, and prepends the entry to `bucket`; returns its slot.
    /// A constructor panic leaves the map unchanged. The caller prepared a table.
    pub(in crate::ncollection) fn insert(
        &mut self,
        bucket: usize,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> usize {
        let (value, key) = construct_value_then_key(construct_value, construct_key);
        let index = self.slots.len();
        let head = self.head_mut(bucket);
        let next = head.replace(index);
        self.slots.push(ChainSlot {
            next,
            bucket,
            key,
            value,
        });
        index
    }
    /// Unlinks `target` from `bucket` and returns its entry for the caller to drop afterwards,
    /// so a panicking destructor finds the map already consistent (native `UnBind` order).
    pub(in crate::ncollection) fn remove(
        &mut self,
        bucket: usize,
        target: usize,
    ) -> ChainSlot<K, V> {
        let next = self.slots[target].next;
        self.set_link(bucket, target, next);
        let removed = self.slots.swap_remove(target);
        let moved_from = self.slots.len();
        if target < moved_from {
            // The last slot moved into `target`; repoint the one link that named it.
            let moved_bucket = self.slots[target].bucket;
            self.set_link(moved_bucket, moved_from, Some(target));
        }
        removed
    }
    /// Native `destroy`: empties the map in bucket/chain order; `release` frees the table.
    pub(in crate::ncollection) fn clear(&mut self, release: bool) {
        let order = self.chain_order();
        let mut entries: Vec<Option<ChainSlot<K, V>>> = std::mem::take(&mut self.slots)
            .into_iter()
            .map(Some)
            .collect();
        self.reset_heads(release);
        // The map is already empty, so a destructor panic cannot expose a removed entry.
        for slot in order {
            drop(entries[slot].take());
        }
    }
    /// Every slot in native traversal order: bucket `0..=NbBuckets`, then each chain.
    pub(in crate::ncollection) fn chain_order(&self) -> Vec<usize> {
        let mut order = Vec::with_capacity(self.slots.len());
        for &head in self.table() {
            let mut link = head;
            while let Some(slot) = link {
                order.push(slot);
                link = self.slots[slot].next;
            }
        }
        order
    }
    fn reset_heads(&mut self, release: bool) {
        match (release, self.heads.as_deref_mut()) {
            (true, _) => self.heads = None,
            (false, Some(heads)) => heads.fill(None),
            (false, None) => {}
        }
    }
    fn head_mut(&mut self, bucket: usize) -> &mut SlotLink {
        let heads = self.heads.as_deref_mut().unwrap_or(&mut []);
        &mut heads[bucket]
    }
    // Replaces the link that names `target` in `bucket` (its head or a predecessor's next).
    fn set_link(&mut self, bucket: usize, target: usize, value: SlotLink) {
        let mut previous: SlotLink = None;
        let mut link = self.table()[bucket];
        while let Some(slot) = link.filter(|&slot| slot != target) {
            previous = Some(slot);
            link = self.slots[slot].next;
        }
        match previous {
            Some(slot) => self.slots[slot].next = value,
            None => *self.head_mut(bucket) = value,
        }
    }
}
