use super::{Debug, HashPolicy, IndexedDataMap, IndexedDataMapNode};
impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Inserts without replacing existing values, returning the index; `map.add(1,42)`.
    pub fn add(&mut self, key: K, value: V) -> usize {
        self.place(key, || value, false)
    }
}
impl<K: Debug + Clone, V: Clone, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Adds borrowed values only when the key is absent; `map.add_ref(&key,&value)`.
    ///
    /// This hashes and looks up the original key once. Existing bindings are returned without
    /// cloning either argument; a new binding clones the item before the key, matching OCCT's
    /// node construction order for value-semantic types.
    ///
    /// # Example
    /// ```
    /// use rustycad_tkernel::ncollection::IndexedDataMap;
    /// let mut map = IndexedDataMap::<i32, i32>::default();
    /// assert_eq!(map.add_ref(&1, &42), 1);
    /// assert_eq!(map.add_ref(&1, &99), 1);
    /// assert_eq!(map.find_from_index(1), Ok(&42));
    /// ```
    pub fn add_ref(&mut self, key: &K, value: &V) -> usize {
        self.prepare_borrowed_add();
        let (index, bucket) = self.storage.lookup(key, &self.hasher);
        if index != 0 {
            return index;
        }
        self.insert_borrowed(key, value, bucket)
    }
    pub(super) fn prepare_borrowed_add(&mut self) {
        if self.is_empty() || self.len() > self.bucket_count() {
            self.resize(self.len());
        }
    }
    pub(super) fn insert_borrowed(&mut self, key: &K, value: &V, bucket: usize) -> usize {
        self.storage.append(bucket, |index, next| {
            IndexedDataMapNode::construct_parts(index, next, || key.clone(), || value.clone())
        })
    }
}
impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Inserts or replaces a value, reporting new-key insertion; `map.bind(1,42)`.
    pub fn bind(&mut self, key: K, value: V) -> bool {
        let old_len = self.len();
        self.place(key, || value, true) > old_len
    }
    /// Inserts or replaces and returns stable value storage; `*map.bound(1,42)=7`.
    pub fn bound(&mut self, key: K, value: V) -> &mut V {
        let index = self.place(key, || value, true);
        self.storage.value_mut(index)
    }
    /// Returns an existing value or inserts without replacement; `map.try_bound(1,42)`.
    pub fn try_bound(&mut self, key: K, value: V) -> &mut V {
        let index = self.place(key, || value, false);
        self.storage.value_mut(index)
    }
    /// Constructs a replacement value in place, returning the index; `map.emplace(1,||42)`.
    pub fn emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> usize {
        self.place(key, construct, true)
    }
    /// Constructs and returns a replacement value; `map.emplaced(1,||42)`.
    pub fn emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let index = self.place(key, construct, true);
        self.storage.value_mut(index)
    }
    /// Constructs only for absent keys, returning the index; `map.try_emplace(1,||42)`.
    pub fn try_emplace(&mut self, key: K, construct: impl FnOnce() -> V) -> usize {
        self.place(key, construct, false)
    }
    /// Borrows an existing value or constructs it lazily; `map.try_emplaced(1,||42)`.
    pub fn try_emplaced(&mut self, key: K, construct: impl FnOnce() -> V) -> &mut V {
        let index = self.place(key, construct, false);
        self.storage.value_mut(index)
    }
    pub(super) fn place(&mut self, key: K, construct: impl FnOnce() -> V, replace: bool) -> usize {
        if self.is_empty() || self.len() > self.bucket_count() {
            self.resize(self.len());
        }
        let (index, bucket) = self.storage.lookup(&key, &self.hasher);
        if index != 0 {
            if replace {
                *self.storage.value_mut(index) = construct();
            }
            return index;
        }
        self.storage.append(bucket, |index, next| {
            IndexedDataMapNode::construct(key, index, next, construct)
        })
    }
}
