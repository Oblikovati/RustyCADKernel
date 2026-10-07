//! Borrowed-key forwarding avoids copying keys until a missing entry is allocated.
use super::{DataMap, Debug, HashPolicy};

impl<K: Debug + Clone, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// Constructs a value using a borrowed key, copying the key only when absent; `map.emplace_ref(&1, || 2)`.
    pub fn emplace_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> bool {
        self.place_borrowed(key, construct, true).0
    }
    /// Constructs or replaces through a borrowed key and returns the item; `map.emplaced_ref(&1, || 2)`.
    pub fn emplaced_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> &mut V {
        self.place_borrowed(key, construct, true).1
    }
    /// Constructs and copies a borrowed key only if absent; `map.try_emplace_ref(&1, || 2)`.
    pub fn try_emplace_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> bool {
        self.place_borrowed(key, construct, false).0
    }
    /// Returns an existing item or constructs with a borrowed key; `map.try_emplaced_ref(&1, || 2)`.
    pub fn try_emplaced_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> &mut V {
        self.place_borrowed(key, construct, false).1
    }
    pub(super) fn place_borrowed(
        &mut self,
        key: &K,
        construct: impl FnOnce() -> V,
        replace: bool,
    ) -> (bool, &mut V) {
        if self.is_empty() || self.len() > self.bucket_count() {
            self.resize(self.len());
        }
        let (bucket, slot) = self.lookup_with_bucket(key);
        match slot {
            Some(slot) => (false, self.update_slot(slot, construct, replace)),
            None => self.insert_with(bucket, || key.clone(), construct),
        }
    }
}

impl<K: Debug + Clone, V: Clone, H: HashPolicy<K>> DataMap<K, V, H> {
    /// Binds borrowed key and value, returning the stored slot; `map.bound_ref(&1, &2)`.
    pub fn bound_ref(&mut self, key: &K, value: &V) -> &mut V {
        self.place_borrowed(key, || value.clone(), true).1
    }
    /// Copies borrowed key and value only if absent; `map.try_bind_ref(&1, &2)`.
    pub fn try_bind_ref(&mut self, key: &K, value: &V) -> bool {
        self.place_borrowed(key, || value.clone(), false).0
    }
    /// Borrows the existing item or copies both inputs on insertion; `map.try_bound_ref(&1, &2)`.
    pub fn try_bound_ref(&mut self, key: &K, value: &V) -> &mut V {
        self.place_borrowed(key, || value.clone(), false).1
    }
}
