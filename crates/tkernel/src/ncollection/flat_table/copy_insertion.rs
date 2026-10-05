//! Native temporary copies follow capacity preparation and hashing, even on hits.
use super::{FlatTable, HashPolicy, Inserted};

impl<K: Clone, V: Clone, H: HashPolicy<K>> FlatTable<K, V, H> {
    pub fn insert_copied_pair(&mut self, key: &K, value: &V, replace: bool) -> Inserted {
        let hash: usize = self.reserve_then_hash(key);
        let copied_key: K = key.clone();
        let copied_value: V = value.clone();
        self.insert_prepared(copied_key, copied_value, hash, false, replace)
    }
}
impl<K: Clone, V, H: HashPolicy<K>> FlatTable<K, V, H> {
    pub fn insert_copied_key(&mut self, key: &K, value: V, replace: bool) -> Inserted {
        let hash: usize = self.reserve_then_hash(key);
        let copied_key: K = key.clone();
        self.insert_prepared(copied_key, value, hash, false, replace)
    }
}
impl<K, V: Clone, H: HashPolicy<K>> FlatTable<K, V, H> {
    pub fn insert_copied_value(&mut self, key: K, value: &V, replace: bool) -> Inserted {
        let hash: usize = self.reserve_then_hash(&key);
        let copied_value: V = value.clone();
        self.insert_prepared(key, copied_value, hash, false, replace)
    }
}
