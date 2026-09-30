use super::{DataMap, Debug, HashPolicy};
impl<K: Debug, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// Inserts or replaces a value, returning whether the key is new; `map.bind(1,42)`.
    pub fn bind(&mut self, key: K, value: V) -> bool {
        self.place(key, || value, true).0
    }
    /// Inserts only when absent; `map.try_bind(1,42)`.
    pub fn try_bind(&mut self, key: K, value: V) -> bool {
        self.place(key, || value, false).0
    }
    /// Returns an existing value or inserts the provided value; `map.try_bound(1,42)`.
    pub fn try_bound(&mut self, key: K, value: V) -> &mut V {
        self.place(key, || value, false).1
    }
    /// Constructs or replaces an item, returning insertion status; `map.emplace(1,||42)`.
    pub fn emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> bool {
        self.place(key, construct, true).0
    }
    /// Constructs or replaces an item and returns it; `map.emplaced(1,||42)`.
    pub fn emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        self.place(key, construct, true).1
    }
    /// Constructs only for an absent key; `map.try_emplace(1,||42)`.
    pub fn try_emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> bool {
        self.place(key, construct, false).0
    }
    /// Returns an existing item or constructs it lazily; `map.try_emplaced(1,||42)`.
    pub fn try_emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        self.place(key, construct, false).1
    }
}
impl<K: Debug + Clone, V: Clone, H: HashPolicy<K> + Clone> DataMap<K, V, H> {
    /// Copies bindings while preserving the destination hasher and allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        self.clear(true);
        if source.is_empty() {
            return;
        }
        self.resize(source.len() - 1);
        for (key, value) in source.items() {
            self.bind(key.clone(), value.clone());
        }
    }
}
impl<K: Debug + Clone, V: Clone, H: HashPolicy<K> + Clone> Clone for DataMap<K, V, H> {
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(
            self.hasher.clone(),
            self.bucket_count,
            self.resource.clone(),
        );
        copy.assign(self);
        copy
    }
}
