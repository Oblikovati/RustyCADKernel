//! The keys and values of a slot vector, kept as two parallel vectors so that a map can lend
//! its keys or its values alone as plain slices (the indexed maps' split iterators).

/// Slot `i` holds `keys[i]` and `values[i]`. DoubleMap stores `Key1` as the key and `Key2`
/// (its native node value) as the value; key-only maps use `()` values, which allocate nothing.
pub(in crate::ncollection) struct SlotPayload<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K, V> Default for SlotPayload<K, V> {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<K, V> SlotPayload<K, V> {
    pub(in crate::ncollection) fn len(&self) -> usize {
        self.keys.len()
    }
    /// Appends a constructed pair; callers construct the value before the key, as native.
    pub(in crate::ncollection) fn push(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.values.push(value);
    }
    pub(in crate::ncollection) fn key(&self, slot: usize) -> &K {
        &self.keys[slot]
    }
    pub(in crate::ncollection) fn value(&self, slot: usize) -> &V {
        &self.values[slot]
    }
    pub(in crate::ncollection) fn key_mut(&mut self, slot: usize) -> &mut K {
        &mut self.keys[slot]
    }
    pub(in crate::ncollection) fn value_mut(&mut self, slot: usize) -> &mut V {
        &mut self.values[slot]
    }
    pub(in crate::ncollection) fn entry(&self, slot: usize) -> (&K, &V) {
        (&self.keys[slot], &self.values[slot])
    }
    pub(in crate::ncollection) fn entry_mut(&mut self, slot: usize) -> (&K, &mut V) {
        (&self.keys[slot], &mut self.values[slot])
    }
    pub(in crate::ncollection) fn pair_mut(&mut self, slot: usize) -> (&mut K, &mut V) {
        (&mut self.keys[slot], &mut self.values[slot])
    }
    pub(in crate::ncollection) fn keys(&self) -> &[K] {
        &self.keys
    }
    pub(in crate::ncollection) fn values(&self) -> &[V] {
        &self.values
    }
    /// Shared keys beside exclusive values, for ranges that lend `(&K, &mut V)`.
    pub(in crate::ncollection) fn split_mut(&mut self) -> (&[K], &mut [V]) {
        (&self.keys, &mut self.values)
    }
    /// Removes a pair; the last pair moves into `slot`, as `SlotChains::swap_remove` assumes.
    pub(in crate::ncollection) fn swap_remove(&mut self, slot: usize) -> (K, V) {
        (self.keys.swap_remove(slot), self.values.swap_remove(slot))
    }
    /// Exchanges two pairs, as when two indices trade places.
    pub(in crate::ncollection) fn swap(&mut self, first: usize, second: usize) {
        self.keys.swap(first, second);
        self.values.swap(first, second);
    }
    /// Drops every pair in `order` (native bucket/chain destruction order), key then value as
    /// the native node members. The map that owned them is already empty, so a destructor panic
    /// cannot expose a removed entry.
    pub(in crate::ncollection) fn drop_in_order(self, order: Vec<usize>) {
        let mut keys: Vec<Option<K>> = self.keys.into_iter().map(Some).collect();
        let mut values: Vec<Option<V>> = self.values.into_iter().map(Some).collect();
        for slot in order {
            drop(keys[slot].take());
            drop(values[slot].take());
        }
    }
}
