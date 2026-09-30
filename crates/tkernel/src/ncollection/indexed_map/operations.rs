use super::{Debug, Failure, HashPolicy, IndexedMap};
impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Replaces a key while rejecting duplicates at another index; `map.substitute(2,25)?`.
    pub fn substitute(&mut self, index: usize, key: K) -> Result<(), Failure> {
        self.check_index(index)?;
        let existing = self.find_index(&key);
        if existing != 0 && existing != index {
            return Err(Failure::with_type(
                "Standard_DomainError",
                &format!(
                    "IndexedMap key {key:?} already at {existing}; expected an absent key or index {index}"
                ),
                "",
            ));
        }
        if existing == 0 {
            self.unlink_index(index - 1);
            let bucket = self.hash_bucket(&key);
            self.buckets[bucket].insert(0, index - 1);
        }
        self.nodes[index - 1].key = key;
        Ok(())
    }
    /// Exchanges two indices without moving key storage; `map.swap(1,3)?`.
    pub fn swap(&mut self, first: usize, second: usize) -> Result<(), Failure> {
        self.check_index(first)?;
        self.check_index(second)?;
        if first == second {
            return Ok(());
        }
        let a = self.hash_bucket(&self.nodes[first - 1].key);
        let b = self.hash_bucket(&self.nodes[second - 1].key);
        self.swap_bucket_indices(a, first - 1, second - 1);
        if a != b {
            self.swap_bucket_indices(b, first - 1, second - 1);
        }
        self.nodes.swap(first - 1, second - 1);
        Ok(())
    }
    /// Removes the final index; empty maps report OutOfRange; `map.remove_last()?`.
    pub fn remove_last(&mut self) -> Result<(), Failure> {
        self.check_index(self.len())?;
        self.unlink_index(self.len() - 1);
        self.nodes.pop();
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
    fn unlink_index(&mut self, index: usize) {
        let bucket = self.hash_bucket(&self.nodes[index].key);
        self.buckets[bucket].retain(|&stored| stored != index);
    }
    fn swap_bucket_indices(&mut self, bucket: usize, first: usize, second: usize) {
        for index in &mut self.buckets[bucket] {
            *index = if *index == first {
                second
            } else if *index == second {
                first
            } else {
                *index
            };
        }
    }
}
impl<K: Debug + Clone, H: HashPolicy<K> + Clone> IndexedMap<K, H> {
    /// Copies index order while retaining destination policy and allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        self.clear(true);
        if source.is_empty() {
            return;
        }
        self.resize(source.len() - 1);
        for key in source {
            self.append_new(key.clone());
        }
    }
}
impl<K: Debug + Clone, H: HashPolicy<K> + Clone> Clone for IndexedMap<K, H> {
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
