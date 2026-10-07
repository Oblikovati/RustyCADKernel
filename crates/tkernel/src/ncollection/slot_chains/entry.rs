//! The payload of one slot and the native destruction order of a whole slot vector.

/// One stored entry; fields drop key then value, the native `~DataMapNode` member order.
/// DoubleMap stores `Key1` as the key and `Key2` (its node value) as the value.
pub(in crate::ncollection) struct ChainSlot<K, V> {
    key: K,
    value: V,
}

impl<K, V> ChainSlot<K, V> {
    /// Wraps constructed parts; callers construct the value before the key, as native nodes do.
    pub(in crate::ncollection) fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
    pub(in crate::ncollection) fn key(&self) -> &K {
        &self.key
    }
    pub(in crate::ncollection) fn value(&self) -> &V {
        &self.value
    }
    pub(in crate::ncollection) fn key_value_mut(&mut self) -> (&K, &mut V) {
        (&self.key, &mut self.value)
    }
    /// Key-only maps replace an equivalent stored key; the chain position does not change.
    pub(in crate::ncollection) fn replace_key(&mut self, key: K) {
        self.key = key;
    }
}

/// Drops `slots` in `order` (native bucket/chain destruction order). The map that owned them is
/// already empty, so a destructor panic cannot expose a removed entry.
pub(in crate::ncollection) fn drop_in_order<T>(slots: Vec<T>, order: Vec<usize>) {
    let mut entries: Vec<Option<T>> = slots.into_iter().map(Some).collect();
    for slot in order {
        drop(entries[slot].take());
    }
}
