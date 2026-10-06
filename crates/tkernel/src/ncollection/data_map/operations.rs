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
        self.place_borrowed(key, || value.clone(), true).0
    }
}
