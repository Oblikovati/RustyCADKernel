//! Native `appendToList`/`unlinkFromList` over slot indices, kept beside the slot storage.
use super::OrderedTable;
use crate::ncollection::slot_chains::SlotLink;
use std::sync::atomic::{AtomicU64, Ordering};

/// The next entry identity of any ordered map; identities only need uniqueness, not order.
static NEXT_SERIAL: AtomicU64 = AtomicU64::new(0);

/// The insertion-order links of one slot and the identity of its entry.
#[derive(Clone, Copy, Debug)]
pub(super) struct OrderSlot {
    pub(super) previous: SlotLink,
    pub(super) next: SlotLink,
    pub(super) serial: u64,
}

impl<K, V, H> OrderedTable<K, V, H> {
    /// Native `appendToList` for the slot the storage just appended, with a fresh identity.
    pub(super) fn append_order(&mut self, slot: usize) {
        let serial = NEXT_SERIAL.fetch_add(1, Ordering::Relaxed);
        self.order.push(OrderSlot {
            previous: self.last,
            next: None,
            serial,
        });
        self.serials.insert(serial, slot);
        match self.last {
            Some(last) => self.order[last].next = Some(slot),
            None => self.first = Some(slot),
        }
        self.last = Some(slot);
    }
    /// Native `Remove` order: unlink the order list, then the chains (the storage counts down
    /// with them); returns the pair for the caller to drop once the map is consistent.
    pub(super) fn remove_slot(&mut self, slot: usize) -> (K, V) {
        self.unlink_order(slot);
        let removed = self.storage.remove(slot);
        self.forget_slot(slot);
        removed
    }
    // Native `unlinkFromList`.
    fn unlink_order(&mut self, slot: usize) {
        let OrderSlot { previous, next, .. } = self.order[slot];
        match previous {
            Some(previous) => self.order[previous].next = next,
            None => self.first = next,
        }
        match next {
            Some(next) => self.order[next].previous = previous,
            None => self.last = previous,
        }
    }
    // The storage moved its last slot into `slot`; move that entry's order state the same way.
    fn forget_slot(&mut self, slot: usize) {
        let removed = self.order.swap_remove(slot);
        self.serials.remove(&removed.serial);
        let moved_from = self.order.len();
        if slot < moved_from {
            self.repoint_order(slot);
        }
    }
    // Every link that named the moved slot (neighbours, head, tail, identity) now names `to`.
    fn repoint_order(&mut self, to: usize) {
        let OrderSlot {
            previous,
            next,
            serial,
        } = self.order[to];
        match previous {
            Some(previous) => self.order[previous].next = Some(to),
            None => self.first = Some(to),
        }
        match next {
            Some(next) => self.order[next].previous = Some(to),
            None => self.last = Some(to),
        }
        self.serials.insert(serial, to);
    }
}
