//! Returning assignments and borrowed bindings of the ordered map.
use super::{HashPolicy, OrderedDataMap};

impl<K, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Swaps both maps and returns this destination; `map.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) -> &mut Self {
        std::mem::swap(self, source);
        self
    }

    /// Models copy/move self-assignment without aliased references; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}

impl<K: Clone, V: Clone, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Copies a binding, replacing only its item on a hit; `map.bind_ref(&1, &42)`.
    pub fn bind_ref(&mut self, key: &K, value: &V) -> bool {
        self.table.place_borrowed(key, value, true).0
    }

    /// Copies a binding and borrows the stored item; `*map.bound_ref(&1, &42) = 7`.
    pub fn bound_ref(&mut self, key: &K, value: &V) -> &mut V {
        let (_, slot) = self.table.place_borrowed(key, value, true);
        self.table.value_mut(slot)
    }

    /// Copies key and item only for an absent binding; `map.try_bind_ref(&1, &42)`.
    pub fn try_bind_ref(&mut self, key: &K, value: &V) -> bool {
        self.table.place_borrowed(key, value, false).0
    }

    /// Returns the existing item or copies a new binding; `map.try_bound_ref(&1, &42)`.
    pub fn try_bound_ref(&mut self, key: &K, value: &V) -> &mut V {
        let (_, slot) = self.table.place_borrowed(key, value, false);
        self.table.value_mut(slot)
    }
}
