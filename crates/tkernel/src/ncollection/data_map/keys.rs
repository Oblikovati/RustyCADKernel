// Set keys share the DataMap bucket engine without exposing mutable keys publicly.

use super::{DataMap, Debug, Entry, HashPolicy};
impl<K: Debug, H: HashPolicy<K>> DataMap<K, (), H> {
    pub(in crate::ncollection) fn stored_key(&self, key: &K) -> Option<&K> {
        let (bucket, index) = self.lookup(key)?;
        Some(&self.buckets[bucket][index].key)
    }
    pub(in crate::ncollection) fn place_key(&mut self, key: K, replace: bool) -> (bool, &K) {
        self.prepare_key_insertion();
        let (bucket, index) = self.lookup_with_bucket(&key);
        if let Some(index) = index {
            let stored = &mut self.buckets[bucket][index].key;
            if replace {
                *stored = key;
            }
            return (false, stored);
        }
        self.insert_key_with(bucket, || key)
    }
    fn prepare_key_insertion(&mut self) {
        if self.is_empty() || self.count > self.bucket_count {
            self.resize(self.count);
        }
    }
    fn insert_key_with(&mut self, bucket: usize, construct: impl FnOnce() -> K) -> (bool, &K) {
        // OCCT placement new obtains node storage before copying the borrowed key.
        let node = self.resource.allocate_value_with(|| Entry {
            key: construct(),
            value: (),
        });
        self.buckets[bucket].insert(0, node);
        self.count += 1;
        (true, &self.buckets[bucket][0].key)
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> DataMap<K, (), H> {
    pub(in crate::ncollection) fn place_key_ref(&mut self, key: &K) -> (bool, &K) {
        self.prepare_key_insertion();
        let (bucket, index) = self.lookup_with_bucket(key);
        if let Some(index) = index {
            return (false, &self.buckets[bucket][index].key);
        }
        self.insert_key_with(bucket, || key.clone())
    }
}
