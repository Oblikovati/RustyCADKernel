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
impl<K: Debug + Clone, V: Clone, H: HashPolicy<K>> DataMap<K, V, H> {
    /// Binds borrowed values, cloning only after lookup; `map.bind_ref(&key,&value)`.
    ///
    /// This preserves source values and the existing slot address. It follows OCCT's
    /// construct-then-replace value path; user-defined C++ move-assignment hooks are not modeled.
    ///
    /// # Example
    /// ```
    /// use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// assert!(map.bind_ref(&1, &42));
    /// assert!(!map.bind_ref(&1, &43));
    /// assert_eq!(*map.find(&1).unwrap(), 43);
    /// ```
    pub fn bind_ref(&mut self, key: &K, value: &V) -> bool {
        if self.count == 0 || self.count > self.bucket_count {
            self.resize(self.count);
        }
        let (bucket, index) = self.lookup_with_bucket(key);
        self.replace_or_insert_borrowed(key, value, bucket, index)
    }
    fn replace_or_insert_borrowed(
        &mut self,
        key: &K,
        value: &V,
        bucket: usize,
        index: Option<usize>,
    ) -> bool {
        let Some(index) = index else {
            return self.insert_borrowed(key, value, bucket);
        };
        self.buckets[bucket][index].value = value.clone();
        false
    }
    fn insert_borrowed(&mut self, key: &K, value: &V, bucket: usize) -> bool {
        let entry = self.resource.allocate_value_with(|| {
            let copied_value = value.clone();
            let copied_key = key.clone();
            super::Entry {
                key: copied_key,
                value: copied_value,
            }
        });
        self.buckets[bucket].insert(0, entry);
        self.count += 1;
        true
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
