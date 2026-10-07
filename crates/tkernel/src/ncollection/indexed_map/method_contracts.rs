//! OCCT assignment identity, storage exchange and the public bucket table.
use super::{Debug, HashPolicy, IndexedMap};
use crate::ncollection::HashBucketTable;

impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Native `Buckets()`: the key table (chains yield `(&key, &())`), or `None` where native
    /// returns null. The index table `myData2` is a protected member (plan D1 class B6).
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedMap;
    /// let mut map = IndexedMap::<i32>::default();
    /// assert!(map.buckets().is_none());
    /// map.add(7);
    /// let table = map.buckets().unwrap();
    /// assert_eq!(table.chain(8)?.map(|(key, _)| *key).collect::<Vec<_>>(), [7]);
    /// # Ok::<(), rustycad_tkernel::standard::Failure>(())
    /// ```
    pub fn buckets(&self) -> Option<HashBucketTable<'_, K, ()>> {
        self.storage.buckets()
    }
    /// Exchanges both maps as native move assignment does; `target.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) -> &mut Self {
        self.exchange(source);
        self
    }

    /// Preserves self-assignment without creating aliased references; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}
