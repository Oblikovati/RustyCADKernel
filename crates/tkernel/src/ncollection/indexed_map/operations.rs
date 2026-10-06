use super::{Debug, Failure, HashPolicy, IndexedMap};
impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Replaces a key while rejecting duplicates at another index; `map.substitute(2,25)?`.
    pub fn substitute(&mut self, index: usize, key: K) -> Result<(), Failure> {
        let (bucket, detached) =
            self.storage
                .prepare_substitute(index, &key, &self.hasher, "IndexedMap")?;
        self.storage
            .replace_substitute(index, bucket, detached, |node| unsafe {
                *(*node).key1() = key;
            });
        Ok(())
    }
    /// Exchanges two indices without moving key storage; `map.swap(1,3)?`.
    pub fn swap(&mut self, first: usize, second: usize) -> Result<(), Failure> {
        self.check_index(first)?;
        self.check_index(second)?;
        if first != second {
            self.storage.swap(first, second);
        }
        Ok(())
    }
    /// Removes the final index; empty maps report OutOfRange; `map.remove_last()?`.
    pub fn remove_last(&mut self) -> Result<(), Failure> {
        self.check_index(self.len())?;
        self.storage.remove_last(&self.hasher);
        Ok(())
    }
    /// Removes an index and moves the last index into its position; `map.remove_from_index(2)?`.
    pub fn remove_from_index(&mut self, index: usize) -> Result<(), Failure> {
        self.swap(index, self.len())?;
        self.remove_last()
    }
    /// Removes a key using swap-last indexing; `map.remove_key(&42)`.
    pub fn remove_key(&mut self, key: &K) -> bool {
        let index = self.find_index(key);
        if index == 0 {
            return false;
        }
        self.remove_from_index(index)
            .expect("lookup returned a valid index");
        true
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Copies index order while retaining destination policy and allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        // Native Assign retains the destination policy, allocator, and bucket table.
        self.clear(false);
        if source.is_empty() {
            return self;
        }
        self.resize(source.len() - 1);
        for key in source {
            let bucket = self.hash_bucket(key);
            self.append_new_at_bucket_with(bucket, || key.clone());
        }
        self
    }
}
impl<K: Debug + Clone, H: HashPolicy<K> + Clone> Clone for IndexedMap<K, H> {
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(
            self.hasher.clone(),
            self.bucket_count(),
            self.allocator().clone(),
        );
        copy.assign(self);
        copy
    }
}

impl<K: Debug + Clone, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Copies into the existing key with native assignment semantics; `map.substitute_ref(1, &7)?`.
    pub fn substitute_ref(&mut self, index: usize, key: &K) -> Result<(), Failure> {
        let (bucket, detached) =
            self.storage
                .prepare_substitute(index, key, &self.hasher, "IndexedMap")?;
        self.storage
            .replace_substitute(index, bucket, detached, |node| unsafe {
                (*node).key1().clone_from(key);
            });
        Ok(())
    }
}
