use super::{BaseAllocator, DataMap, Debug, HashPolicy};
use std::ops::{Index, IndexMut};

impl<K: Debug, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// `Bound`: binds or replaces like `Bind` and returns the stored item.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// *map.bound(1, 10) += 1;
    /// assert_eq!(*map.find(&1).unwrap(), 11);
    /// ```
    pub fn bound(&mut self, key: K, value: V) -> &mut V {
        self.place(key, || value, true).1
    }

    /// `Contained() const`: the stored key and item of an equivalent key, if bound.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(1, 10);
    /// assert_eq!(map.contained(&1), Some((&1, &10)));
    /// ```
    pub fn contained(&self, key: &K) -> Option<(&K, &V)> {
        let (_, node) = self.lookup(key)?;
        let entry = unsafe { self.storage.node(node) };
        Some((entry.key(), entry.value()))
    }

    /// `Contained()`: the stored key and a mutable item of an equivalent key, if bound.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(1, 10);
    /// *map.contained_mut(&1).unwrap().1 = 5;
    /// assert_eq!(map.seek(&1), Some(&5));
    /// ```
    pub fn contained_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        let (_, node) = self.lookup(key)?;
        Some(unsafe { self.storage.node_mut(node).key_value_mut() })
    }

    /// `Seek`: the bound item or `None` (the native null pointer).
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let map = DataMap::<i32, i32>::default();
    /// assert_eq!(map.seek(&1), None);
    /// ```
    pub fn seek(&self, key: &K) -> Option<&V> {
        self.contained(key).map(|(_, value)| value)
    }

    /// `ChangeSeek`: the bound item for modification or `None`.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(2, 1);
    /// *map.change_seek(&2).unwrap() = 3;
    /// assert_eq!(map.seek(&2), Some(&3));
    /// ```
    pub fn change_seek(&mut self, key: &K) -> Option<&mut V> {
        self.contained_mut(key).map(|(_, value)| value)
    }

    /// `Exchange`: swaps contents, hash policy and allocator with `other`.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let (mut a, mut b) = (DataMap::<i32, i32>::default(), DataMap::<i32, i32>::default());
    /// a.bind(1, 1);
    /// a.exchange(&mut b);
    /// assert!(a.is_empty() && b.is_bound(&1));
    /// ```
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }

    /// `Clear(const Handle(NCollection_BaseAllocator)&)`: clears, releasing buckets only when
    /// the allocator changes; `None` (a null handle) selects the common allocator.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::{BaseAllocator, DataMap};
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(1, 1);
    /// let arena = BaseAllocator::incremental(1024);
    /// map.clear_with_allocator(Some(arena.clone()));
    /// assert!(map.is_empty() && map.allocator() == &arena);
    /// ```
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        self.storage.clear_with_allocator(resource);
    }
}

impl<K: Debug, V: Clone, H: HashPolicy<K>> DataMap<K, V, H> {
    /// `Find(key, item&)`: copies a bound item into `output`, leaving it untouched otherwise.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(1, 10);
    /// let mut out = 0;
    /// assert!(map.find_into(&1, &mut out) && out == 10);
    /// ```
    pub fn find_into(&self, key: &K, output: &mut V) -> bool {
        let Some(value) = self.seek(key) else {
            return false;
        };
        output.clone_from(value);
        true
    }
}

/// `operator()(key) const`: `Find`, which raises Standard_NoSuchObject for an unbound key.
impl<K: Debug, V, H: HashPolicy<K>> Index<&K> for DataMap<K, V, H> {
    type Output = V;
    fn index(&self, key: &K) -> &V {
        self.find(key).unwrap_or_else(|failure| panic!("{failure}"))
    }
}

/// `operator()(key)`: `ChangeFind`, which raises Standard_NoSuchObject for an unbound key.
impl<K: Debug, V, H: HashPolicy<K>> IndexMut<&K> for DataMap<K, V, H> {
    fn index_mut(&mut self, key: &K) -> &mut V {
        self.change_find(key)
            .unwrap_or_else(|failure| panic!("{failure}"))
    }
}
