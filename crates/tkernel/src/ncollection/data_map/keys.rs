// Set keys share the DataMap bucket engine without exposing mutable keys publicly.

use super::{DataMap, Debug, Entry, HashPolicy};
impl<K: Debug, H: HashPolicy<K>> DataMap<K, (), H> {
    pub(in crate::ncollection) fn stored_key(&self, key: &K) -> Option<&K> {
        let (bucket, index) = self.lookup(key)?;
        Some(&self.buckets[bucket][index].key)
    }
    pub(in crate::ncollection) fn place_key(&mut self, key: K, replace: bool) -> (bool, &K) {
        if self.is_empty() || self.count > self.bucket_count {
            self.resize(self.count);
        }
        if let Some((bucket, index)) = self.lookup(&key) {
            let stored = &mut self.buckets[bucket][index].key;
            if replace {
                *stored = key;
            }
            return (false, stored);
        }
        let bucket = self.hasher.hash(&key) % self.bucket_count + 1;
        let node = self.resource.allocate_value(Entry { key, value: () });
        self.buckets[bucket].insert(0, node);
        self.count += 1;
        (true, &self.buckets[bucket][0].key)
    }
}
