//! Native forwarding-reference templates also borrow keys until absent-node construction.
use super::{HashPolicy, OrderedDataMap};
impl<K: Clone, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Defers value construction and copies only an absent key; `map.emplace_key_ref(&key, || value)`.
    pub fn emplace_key_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> bool {
        self.table.place_key_ref(key, construct, true).0
    }
    /// Returns replaced or new value storage without consuming the key; `map.emplaced_key_ref(&key, || value)`.
    pub fn emplaced_key_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> &mut V {
        let (_, slot) = self.table.place_key_ref(key, construct, true);
        self.table.value_mut(slot)
    }
    /// Constructs neither key nor value on a hit; `map.try_emplace_key_ref(&key, || value)`.
    pub fn try_emplace_key_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> bool {
        self.table.place_key_ref(key, construct, false).0
    }
    /// Returns existing storage or defers value then key construction; `map.try_emplaced_key_ref(&key, || value)`.
    pub fn try_emplaced_key_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> &mut V {
        let (_, slot) = self.table.place_key_ref(key, construct, false);
        self.table.value_mut(slot)
    }
}
