//! Borrowed binding, returning assignment contracts and the public bucket table.
use super::{Debug, HashPolicy, IndexedDataMap};
use crate::ncollection::HashBucketTable;

impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Native `Buckets()`: the key table with each chain's `(key, item)` pairs, or `None` where
    /// native returns null. The index table `myData2` is a protected member (plan D1 class B6).
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedDataMap;
    /// let mut map = IndexedDataMap::<i32, i32>::default();
    /// map.add(7, 70);
    /// let table = map.buckets().unwrap();
    /// assert_eq!(table.chain(8)?.collect::<Vec<_>>(), [(&7, &70)]);
    /// # Ok::<(), rustycad_tkernel::standard::Failure>(())
    /// ```
    pub fn buckets(&self) -> Option<HashBucketTable<'_, K, V>> {
        self.storage.buckets()
    }
    /// Exchanges entries, allocators and policies like native move assignment; `target.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) -> &mut Self {
        self.exchange(source);
        self
    }

    /// Keeps self-assignment safe without aliasing mutable references; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}

impl<K: Debug + Clone, V: Clone, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Copies a binding, assigning only the item on a hit; `map.bind_ref(&1, &42)`.
    pub fn bind_ref(&mut self, key: &K, value: &V) -> bool {
        let count = self.len();
        self.place_borrowed(key, value, true) > count
    }

    /// Copies a binding and borrows its stored item; `*map.bound_ref(&1, &42) = 7`.
    pub fn bound_ref(&mut self, key: &K, value: &V) -> &mut V {
        let index = self.place_borrowed(key, value, true);
        self.storage.value_at_mut(index)
    }

    /// Copies arguments only if no binding exists; `map.try_bind_ref(&1, &42)`.
    pub fn try_bind_ref(&mut self, key: &K, value: &V) -> bool {
        let count = self.len();
        self.place_borrowed(key, value, false) > count
    }

    /// Borrows a stored item, copying arguments only on a miss; `map.try_bound_ref(&1, &42)`.
    pub fn try_bound_ref(&mut self, key: &K, value: &V) -> &mut V {
        let index = self.place_borrowed(key, value, false);
        self.storage.value_at_mut(index)
    }

    fn place_borrowed(&mut self, key: &K, value: &V, replace: bool) -> usize {
        self.prepare_borrowed_add();
        let (index, bucket) = self.storage.find_index(key, &self.hasher);
        if index == 0 {
            return self.insert_borrowed(key, value, bucket);
        }
        if replace {
            self.storage.value_at_mut(index).clone_from(value);
        }
        index
    }
}
