//! Native node links are authoritative; identity slots are never traversed as a second graph.
use super::*;
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub(super) fn append_allocated(&mut self, node: *mut MapLinkNode, hash: usize) -> usize {
        let slot = self.register_node(node);
        // SAFETY: the new allocation and old tail are distinct owned nodes; only order fields change.
        unsafe {
            if let Some(last) = self.last {
                *N::previous(node) = self.node_pointer(last);
                *N::next(self.node_pointer(last)) = node;
            } else {
                self.first = Some(slot);
            }
        }
        self.last = Some(slot);
        self.base.bucket_slots_mut()[hash] = node;
        self.count = self.base.increment();
        slot
    }
    fn register_node(&mut self, node: *mut MapLinkNode) -> usize {
        let slot = self.free.pop().unwrap_or_else(|| {
            self.slots.push(None);
            self.slots.len() - 1
        });
        self.slots[slot] = Some(node);
        self.identities.insert(node.addr(), slot);
        slot
    }
    pub(super) fn remove_slot(&mut self, slot: usize, hash: usize) {
        let node = self.node_pointer(slot);
        self.count = self.base.decrement();
        self.unlink_bucket(node, hash);
        self.unlink_order(node);
        self.slots[slot] = None;
        self.identities.remove(&node.addr());
        self.free.push(slot);
        // SAFETY: all graph links were detached before releasing this exact native allocation.
        unsafe {
            crate::ncollection::node_release::release_typed_node(node.cast::<N>(), &self.resource)
        };
    }
    fn unlink_bucket(&mut self, target: *mut MapLinkNode, hash: usize) {
        let mut link = std::ptr::from_mut(&mut self.base.bucket_slots_mut()[hash]);
        // SAFETY: lookup selected this target from this live chain; writes touch only link fields.
        unsafe {
            while *link != target {
                link = std::ptr::from_mut((*(*link)).next_mut());
            }
            *link = (*target).next();
        }
    }
    fn unlink_order(&mut self, node: *mut MapLinkNode) {
        // SAFETY: both neighboring nodes remain owned; payload references do not overlap links.
        unsafe {
            let (previous, next) = (*N::previous(node), *N::next(node));
            if previous.is_null() {
                self.first = self.slot_of(next);
            } else {
                *N::next(previous) = next;
            }
            if next.is_null() {
                self.last = self.slot_of(previous);
            } else {
                *N::previous(next) = previous;
            }
        }
    }
}
