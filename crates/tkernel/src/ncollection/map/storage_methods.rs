//! Map shares the safe bucket-chain storage of DataMap with a unit value per key.
use super::{Debug, HashPolicy, Map};
use crate::ncollection::HashBucketTable;
impl<K: Debug, H: HashPolicy<K>> Map<K, H> {
    /// Native `Buckets()`: the allocated table (key-only chains yield `(&key, &())`), or `None`
    /// where native returns null.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::Map;
    /// let mut map = Map::<i32>::default();
    /// assert!(map.buckets().is_none());
    /// map.add(7);
    /// assert_eq!(map.buckets().map(|table| table.len()), Some(102));
    /// ```
    pub fn buckets(&self) -> Option<HashBucketTable<'_, K, ()>> {
        self.storage.buckets()
    }
    pub(super) fn place_key(&mut self, key: K, replace: bool) -> (bool, &K) {
        self.prepare_emplacement();
        let (bucket, slot) = self.storage.lookup_with_bucket(&key, &self.hasher);
        let Some(slot) = slot else {
            return self.insert_key(bucket, || key);
        };
        if replace {
            // Native Emplace destroys the stored key and reconstructs it in place.
            self.storage.slot_mut(slot).replace_key(key);
        }
        (false, self.storage.slot(slot).key())
    }
    fn insert_key(&mut self, bucket: usize, construct: impl FnOnce() -> K) -> (bool, &K) {
        let slot = self.storage.insert(bucket, construct, || ());
        (true, self.storage.slot(slot).key())
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> Map<K, H> {
    pub(super) fn place_key_ref(&mut self, key: &K) -> (bool, &K) {
        self.prepare_emplacement();
        let (bucket, slot) = self.storage.lookup_with_bucket(key, &self.hasher);
        match slot {
            Some(slot) => (false, self.storage.slot(slot).key()),
            None => self.insert_key(bucket, || key.clone()),
        }
    }
}
