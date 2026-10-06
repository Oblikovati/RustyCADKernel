//! Mixed key/item ownership overloads copy only the borrowed argument when needed.
use super::{HashPolicy, OrderedDataMap};

impl<K: Clone, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Copies an absent key and transfers the item; `map.bind_key_ref(&1, 42)`.
    pub fn bind_key_ref(&mut self, key: &K, value: V) -> bool {
        self.table.place_key_ref(key, || value, true).0
    }
    /// Copies an absent key and returns item storage; `*map.bound_key_ref(&1, 42) = 7`.
    pub fn bound_key_ref(&mut self, key: &K, value: V) -> &mut V {
        let (_, slot) = self.table.place_key_ref(key, || value, true);
        self.table.value_mut(slot)
    }
    /// Copies a key only for a new binding, retaining existing items; `map.try_bind_key_ref(&1, 42)`.
    pub fn try_bind_key_ref(&mut self, key: &K, value: V) -> bool {
        self.table.place_key_ref(key, || value, false).0
    }
    /// Returns an existing item or copies a new key; `map.try_bound_key_ref(&1, 42)`.
    pub fn try_bound_key_ref(&mut self, key: &K, value: V) -> &mut V {
        let (_, slot) = self.table.place_key_ref(key, || value, false);
        self.table.value_mut(slot)
    }
}
impl<K, V: Clone, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Transfers an absent key and copies the item; `map.bind_value_ref(1, &42)`.
    pub fn bind_value_ref(&mut self, key: K, value: &V) -> bool {
        self.table.place(key, || value.clone(), false, true).0
    }
    /// Transfers an absent key and returns copied item storage; `*map.bound_value_ref(1, &42) = 7`.
    pub fn bound_value_ref(&mut self, key: K, value: &V) -> &mut V {
        let (_, slot) = self.table.place(key, || value.clone(), false, true);
        self.table.value_mut(slot)
    }
    /// Transfers a key and copies the item only on insertion; `map.try_bind_value_ref(1, &42)`.
    pub fn try_bind_value_ref(&mut self, key: K, value: &V) -> bool {
        self.table.place(key, || value.clone(), false, false).0
    }
    /// Returns an existing item or copies a new item; `map.try_bound_value_ref(1, &42)`.
    pub fn try_bound_value_ref(&mut self, key: K, value: &V) -> &mut V {
        let (_, slot) = self.table.place(key, || value.clone(), false, false);
        self.table.value_mut(slot)
    }
}
