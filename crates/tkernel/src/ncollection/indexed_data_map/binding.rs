use super::{Debug, HashPolicy, IndexedDataMap, IndexedValue};
impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Inserts without replacing existing values, returning the index; `map.add(1,42)`.
    pub fn add(&mut self, key: K, value: V) -> usize {
        self.place(key, || value, false)
    }
    /// Inserts or replaces a value, reporting new-key insertion; `map.bind(1,42)`.
    pub fn bind(&mut self, key: K, value: V) -> bool {
        let old_len = self.len();
        self.place(key, || value, true) > old_len
    }
    /// Inserts or replaces and returns stable value storage; `*map.bound(1,42)=7`.
    pub fn bound(&mut self, key: K, value: V) -> &mut V {
        let index = self.place(key, || value, true);
        &mut self.values[index - 1].value
    }
    /// Returns an existing value or inserts without replacement; `map.try_bound(1,42)`.
    pub fn try_bound(&mut self, key: K, value: V) -> &mut V {
        let index = self.place(key, || value, false);
        &mut self.values[index - 1].value
    }
    /// Constructs a replacement value in place, returning the index; `map.emplace(1,||42)`.
    pub fn emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> usize {
        self.place(key, construct, true)
    }
    /// Constructs and returns a replacement value; `map.emplaced(1,||42)`.
    pub fn emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let index = self.place(key, construct, true);
        &mut self.values[index - 1].value
    }
    /// Constructs only for absent keys, returning the index; `map.try_emplace(1,||42)`.
    pub fn try_emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> usize {
        self.place(key, construct, false)
    }
    /// Borrows an existing value or constructs it lazily; `map.try_emplaced(1,||42)`.
    pub fn try_emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let index = self.place(key, construct, false);
        &mut self.values[index - 1].value
    }
    pub(super) fn place(&mut self, key: K, construct: impl FnOnce() -> V, replace: bool) -> usize {
        if self.is_empty() || self.len() > self.bucket_count() {
            self.resize(self.len());
        }
        let index = self.find_index(&key);
        if index != 0 {
            if replace {
                self.values[index - 1].value = construct();
            }
            return index;
        }
        let value = self.allocator().allocate_value(IndexedValue {
            value: construct(),
            _identity: 0,
        });
        let index = self.keys.add(key);
        self.values.push(value);
        index
    }
}
