// Set keys share the DataMap bucket engine without exposing mutable keys publicly.

use super::{DataMap, Debug, HashPolicy};
impl<K: Debug, H: HashPolicy<K>> DataMap<K, (), H> {
    pub(in crate::ncollection) fn stored_key(&self, key: &K) -> Option<&K> {
        let (_, node) = self.lookup(key)?;
        Some(unsafe { self.storage.node(node).key() })
    }
    pub(in crate::ncollection) fn place_key(&mut self, key: K, replace: bool) -> (bool, &K) {
        self.prepare_key_insertion();
        let (bucket, index) = self.lookup_with_bucket(&key);
        if let Some(index) = index {
            let stored = unsafe { self.storage.node_mut(index).key_mut() };
            if replace {
                *stored = key;
            }
            return (false, stored);
        }
        self.insert_key_with(bucket, || key)
    }
    fn prepare_key_insertion(&mut self) {
        if self.is_empty() || self.len() > self.bucket_count() {
            self.resize(self.len());
        }
    }
    fn insert_key_with(&mut self, bucket: usize, construct: impl FnOnce() -> K) -> (bool, &K) {
        // OCCT placement new obtains node storage before copying the borrowed key.
        let node = self.storage.insert(bucket, construct, || ());
        (true, unsafe { self.storage.node(node).key() })
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> DataMap<K, (), H> {
    pub(in crate::ncollection) fn place_key_ref(&mut self, key: &K) -> (bool, &K) {
        self.prepare_key_insertion();
        let (bucket, index) = self.lookup_with_bucket(key);
        if let Some(index) = index {
            return (false, unsafe { self.storage.node(index).key() });
        }
        self.insert_key_with(bucket, || key.clone())
    }
}
