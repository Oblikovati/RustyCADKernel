//! Copy and move assignment preserve OCCT return identity and allocator rules.
use super::{DataMap, Debug, HashPolicy};

impl<K: Debug + Clone, V: Clone, H: HashPolicy<K> + Clone> DataMap<K, V, H> {
    /// Copies bindings while preserving the destination hasher and allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        self.assign_returning(source);
    }
    /// Copies bindings and returns the destination, preserving its allocator and hasher.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::DataMap;
    /// let mut source = DataMap::<i32, i32>::default();
    /// source.bind(1, 7);
    /// let mut target = DataMap::default();
    /// assert_eq!(target.assign_returning(&source).find(&1), Ok(&7));
    /// ```
    pub fn assign_returning(&mut self, source: &Self) -> &mut Self {
        // OCCT Clear() keeps bucket storage and the destination allocator.
        self.clear(false);
        if source.is_empty() {
            return self;
        }
        self.resize(source.len() - 1);
        for (key, value) in source.items() {
            // OCCT Assign delegates to borrowed Bind: allocate, then copy value before key.
            self.bind_ref(key, value);
        }
        self
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

impl<K: Debug, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// Models OCCT copy/move self-assignment without creating aliased references.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(1, 7);
    /// assert_eq!(map.assign_self().find(&1), Ok(&7));
    /// ```
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Exchanges both maps and returns the destination, as OCCT move assignment does.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::DataMap;
    /// let mut source = DataMap::<i32, i32>::default();
    /// source.bind(1, 7);
    /// let mut target = DataMap::default();
    /// assert_eq!(target.move_from_returning(&mut source).find(&1), Ok(&7));
    /// ```
    pub fn move_from_returning(&mut self, source: &mut Self) -> &mut Self {
        self.exchange(source);
        self
    }
}
