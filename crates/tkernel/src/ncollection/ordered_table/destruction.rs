//! OCCT destroys ordered maps in hash-chain order, independently of insertion order.
use super::{OrderedNode, OrderedTable};

impl<K, V, H> OrderedTable<K, V, H> {
    pub(super) fn unlink_order(&mut self, slot: usize, node: &OrderedNode<K, V>) {
        if let Some(previous) = node.previous {
            self.slots[previous].as_mut().unwrap().next = node.next;
        } else {
            self.first = node.next;
        }
        if let Some(next) = node.next {
            self.slots[next].as_mut().unwrap().previous = node.previous;
        } else {
            self.last = node.previous;
        }
        self.free.push(slot);
    }

    pub fn clear(&mut self, release: bool) {
        // BaseMap::destroy visits increasing buckets and head-to-tail chains before
        // resetting extent; slots may have been reused after arbitrary removals.
        for bucket in &mut self.buckets {
            for slot in bucket.drain(..) {
                drop(self.slots[slot].take());
            }
        }
        self.slots.clear();
        self.free.clear();
        self.first = None;
        self.last = None;
        self.count = 0;
        if release {
            self.buckets.clear();
        }
    }
}

impl<K, V, H> Drop for OrderedTable<K, V, H> {
    fn drop(&mut self) {
        self.clear(true);
    }
}
