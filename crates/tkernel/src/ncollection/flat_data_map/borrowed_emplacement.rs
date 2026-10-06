//! Forwarded borrowed keys are copied after hashing, including on existing-key paths.
use super::{Debug, FlatDataMap, HashPolicy, Inserted};

impl<K: Debug + Clone, V, H: HashPolicy<K>> FlatDataMap<K, V, H> {
    /// Copies a borrowed key after hashing and constructs a replacement; `map.emplace_ref(&1, || 42)`.
    pub fn emplace_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> bool {
        self.place_borrowed(key, construct, true).is_new
    }
    /// Returns the original key's item after borrowed-key forwarding; `map.emplaced_ref(&1, || 42)`.
    pub fn emplaced_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> &mut V {
        let inserted = self.place_borrowed(key, construct, true);
        self.value_at(inserted.original)
    }
    /// Copies a lookup key on every call, constructing a value only on a miss; `map.try_emplace_ref(&1, || 42)`.
    pub fn try_emplace_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> bool {
        self.place_borrowed(key, construct, false).is_new
    }
    /// Copies the key and borrows an existing or newly constructed value; `map.try_emplaced_ref(&1, || 42)`.
    pub fn try_emplaced_ref(&mut self, key: &K, construct: impl FnOnce() -> V) -> &mut V {
        let inserted = self.place_borrowed(key, construct, false);
        self.value_at(inserted.original)
    }
    fn place_borrowed(
        &mut self,
        key: &K,
        construct: impl FnOnce() -> V,
        replace: bool,
    ) -> Inserted {
        // Native 3d097a0 emplaceImpl materializes aKeyToInsert before testing occupied slots.
        let hash = self.table.reserve_then_hash(key);
        self.place_hashed(key.clone(), hash, construct, replace)
    }
}
