//! Payload projections borrow only their fields, preserving other outstanding iterator references.
use super::*;
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn node_pointer(&self, slot: usize) -> *mut MapLinkNode {
        self.slots[slot].unwrap()
    }
    pub fn key(&self, slot: usize) -> &K {
        // SAFETY: a live identity slot names an owned node, borrowed for self's lifetime.
        unsafe { &*N::key(self.node_pointer(slot)) }
    }
    pub fn value(&self, slot: usize) -> &V {
        // SAFETY: this shared projection never lends mutable access to the node or its links.
        unsafe { &*N::value(self.node_pointer(slot)) }
    }
    pub fn next_slot(&self, slot: usize) -> Option<usize> {
        // SAFETY: order links belong to the authoritative owned graph.
        self.slot_of(unsafe { *N::next(self.node_pointer(slot)) })
    }
    pub(super) fn slot_of(&self, node: *mut MapLinkNode) -> Option<usize> {
        self.identities.get(&node.addr()).copied()
    }
}
impl<K, V, H> OrderedTable<K, V, H> {
    pub fn value_mut(&mut self, slot: usize) -> &mut V {
        // SAFETY: an exclusive table borrow excludes competing safe payload references.
        unsafe {
            &mut *<OrderedDataMapNode<K, V> as StorageNode<K, V>>::value(self.node_pointer(slot))
        }
    }
    pub fn pair_mut(&mut self, slot: usize) -> (&K, &mut V) {
        let node = self.node_pointer(slot);
        // SAFETY: the concrete data-node key/value fields are distinct; set nodes cannot enter.
        unsafe {
            (
                &*<OrderedDataMapNode<K, V> as StorageNode<K, V>>::key(node),
                &mut *<OrderedDataMapNode<K, V> as StorageNode<K, V>>::value(node),
            )
        }
    }
}
