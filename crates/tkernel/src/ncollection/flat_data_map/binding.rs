use super::{Debug, FlatDataMap, HashPolicy, Inserted};
impl<K: Debug, V, H: HashPolicy<K>> FlatDataMap<K, V, H> {
    /// Inserts or replaces a value, reporting a new key; `map.bind(1,42)`.
    pub fn bind(&mut self, key: K, value: V) -> bool {
        self.table.insert(key, value, false, true).is_new
    }
    /// Inserts only when absent; `map.try_bind(1,42)`.
    pub fn try_bind(&mut self, key: K, value: V) -> bool {
        self.table.insert(key, value, false, false).is_new
    }
    /// Inserts or replaces and returns the original key's slot; `map.bound(1,42)`.
    pub fn bound(&mut self, key: K, value: V) -> &mut V {
        let result = self.table.insert(key, value, false, true);
        self.value_at(result.original)
    }
    /// Returns an existing value or inserts it without replacement; `map.try_bound(1,42)`.
    pub fn try_bound(&mut self, key: K, value: V) -> &mut V {
        let result = self.table.insert(key, value, false, false);
        self.value_at(result.original)
    }
    /// Constructs a replacement value and reports insertion; `map.emplace(1,||42)`.
    pub fn emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> bool {
        self.place(key, construct, true).is_new
    }
    /// Constructs a replacement and returns its slot; `map.emplaced(1,||42)`.
    pub fn emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let result = self.place(key, construct, true);
        self.value_at(result.original)
    }
    /// Constructs only for absent keys; `map.try_emplace(1,||42)`.
    pub fn try_emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> bool {
        self.place(key, construct, false).is_new
    }
    /// Borrows an existing value or constructs it lazily; `map.try_emplaced(1,||42)`.
    pub fn try_emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let result = self.place(key, construct, false);
        self.value_at(result.original)
    }
    fn place(&mut self, key: K, construct: impl FnOnce() -> V, replace: bool) -> Inserted {
        let hash: usize = self.table.reserve_then_hash(&key);
        self.place_hashed(key, hash, construct, replace)
    }
    pub(super) fn place_hashed(
        &mut self,
        key: K,
        hash: usize,
        construct: impl FnOnce() -> V,
        replace: bool,
    ) -> Inserted {
        if let Some(index) = self.table.find_cached_hash_slot(&key, hash) {
            if replace {
                *self.value_at(index) = construct();
            }
            return Inserted::occupied_slot(index);
        }
        self.table.insert_prehashed(key, construct(), hash)
    }
    pub(super) fn value_at(&mut self, index: usize) -> &mut V {
        &mut self.table.slots[index]
            .as_mut()
            .expect("inserted slot is occupied")
            .value
    }
}
