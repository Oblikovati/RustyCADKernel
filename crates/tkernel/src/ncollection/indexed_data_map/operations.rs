use super::{Debug, Failure, HashPolicy, IndexedDataMap};
impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Replaces an indexed pair, rejecting keys bound elsewhere; `map.substitute(2,25,2.5)?`.
    pub fn substitute(&mut self, index: usize, key: K, value: V) -> Result<(), Failure> {
        self.keys.substitute(index, key)?;
        self.values[index - 1].value = value;
        Ok(())
    }
    /// Swaps two indices while keeping key/value pairing; `map.swap(1,3)?`.
    pub fn swap(&mut self, first: usize, second: usize) -> Result<(), Failure> {
        self.keys.swap(first, second)?;
        self.values.swap(first - 1, second - 1);
        Ok(())
    }
    /// Removes the last binding; empty maps report OutOfRange; `map.remove_last()?`.
    pub fn remove_last(&mut self) -> Result<(), Failure> {
        self.keys.remove_last()?;
        self.values.pop();
        Ok(())
    }
    /// Removes a binding and moves the last binding into its index; `map.remove_from_index(2)?`.
    pub fn remove_from_index(&mut self, index: usize) -> Result<(), Failure> {
        self.keys.remove_from_index(index)?;
        self.values.swap_remove(index - 1);
        Ok(())
    }
    /// Removes a key without affecting absent keys; `map.remove_key(&42)`.
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
impl<K: Debug, V: Clone, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Copies a found value into the output, preserving it when absent; `map.find_copy(&42,&mut value)`.
    pub fn find_copy(&self, key: &K, output: &mut V) -> bool {
        let Some(value) = self.seek(key) else {
            return false;
        };
        output.clone_from(value);
        true
    }
}
impl<K: Debug + Clone, V: Clone, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Copies bindings while retaining destination policy and allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.clear(false);
        if source.is_empty() {
            return self;
        }
        self.resize(source.len() - 1);
        for (key, value) in source.items() {
            // Native Assign copies each item before its key; it does not deduplicate.
            let bucket = self.hasher().hash(key) % self.bucket_count() + 1;
            self.insert_borrowed(key, value, bucket);
        }
        self
    }
}
impl<K: Debug + Clone, V: Clone, H: HashPolicy<K> + Clone> Clone for IndexedDataMap<K, V, H> {
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(
            self.hasher().clone(),
            self.bucket_count(),
            self.allocator().clone(),
        );
        copy.assign(self);
        copy
    }
}
