//! Native variadic key construction runs after capacity preparation, including hits.
use super::super::flat_table::Inserted;
use super::{FlatMap, HashPolicy};

impl<K, H: HashPolicy<K>> FlatMap<K, H> {
    /// Reserves before constructing a replacement key; `map.emplace_with(|| make_key())`.
    pub fn emplace_with(&mut self, construct: impl FnOnce() -> K) -> bool {
        self.place_constructed_key(construct, true).is_new
    }
    /// Returns OCCT's final displaced slot after key construction; `map.emplaced_with(|| make_key())`.
    pub fn emplaced_with(&mut self, construct: impl FnOnce() -> K) -> &K {
        let inserted = self.place_constructed_key(construct, true);
        &self.table.slots[inserted.last].as_ref().unwrap().key
    }
    /// Constructs a temporary even on hits, preserving existing keys; `map.try_emplace_with(|| make_key())`.
    pub fn try_emplace_with(&mut self, construct: impl FnOnce() -> K) -> bool {
        self.place_constructed_key(construct, false).is_new
    }
    /// Preserves existing keys and the native final-slot return; `map.try_emplaced_with(|| make_key())`.
    pub fn try_emplaced_with(&mut self, construct: impl FnOnce() -> K) -> &K {
        let inserted = self.place_constructed_key(construct, false);
        &self.table.slots[inserted.last].as_ref().unwrap().key
    }
    fn place_constructed_key(&mut self, construct: impl FnOnce() -> K, replace: bool) -> Inserted {
        // OCCT 3d097a0 ensureCapacity precedes aTempKey construction, even if it throws.
        self.table.prepare();
        let key: K = construct();
        let hash: usize = self.table.hasher.hash(&key);
        self.table.insert_prepared(key, (), hash, replace, false)
    }
}
