use super::{Debug, Failure, HashPolicy, IndexedMap};
impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Replaces a key while rejecting duplicates at another index; `map.substitute(2,25)?`.
    pub fn substitute(&mut self, index: usize, key: K) -> Result<(), Failure> {
        self.check_index(index)?;
        let (existing, bucket) = self.lookup_with_bucket(&key);
        self.check_substitute(index, existing, &key)?;
        let node = self.storage.node(index);
        if existing == 0 {
            self.storage
                .unlink(self.hash_bucket(self.storage.key(index)), node);
        }
        self.replace_substitute(index, bucket, existing == 0, |stored| *stored = key);
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
        let node = self.storage.node(self.len());
        self.storage.set_last_slot(std::ptr::null_mut());
        let hashed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.hash_bucket(unsafe { (&*node.cast::<super::IndexedMapNode<K>>()).value() })
        }));
        let bucket = match hashed {
            Ok(bucket) => bucket,
            Err(panic) => {
                self.storage.set_last_slot(node);
                std::panic::resume_unwind(panic)
            }
        };
        self.storage.destroy_last(bucket, node);
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

impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    fn check_substitute(&self, index: usize, existing: usize, key: &K) -> Result<(), Failure> {
        if existing == 0 || existing == index {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_DomainError",
            &format!(
                "IndexedMap key {key:?} already at {existing}; expected an absent key or index {index}"
            ),
            "",
        ))
    }
}
impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    fn replace_substitute(
        &mut self,
        index: usize,
        bucket: usize,
        detached: bool,
        assign: impl FnOnce(&mut K),
    ) {
        let node = self.storage.node(index);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assign(self.storage.key_mut(index))
        }));
        if detached {
            self.storage.prepend(bucket, node);
        }
        if let Err(panic) = result {
            self.clear_failed_assignment(panic);
        }
    }
    fn clear_failed_assignment(&mut self, panic: Box<dyn std::any::Any + Send>) {
        // A throwing clone_from can change the hash key partially. Clearing is a Rust safety
        // extension; the nonthrowing native assignment/unlink/relink order stays unchanged.
        let mut panics = crate::ncollection::destructor_unwind::DeferredDestructorPanic::default();
        panics.capture(|| std::panic::resume_unwind(panic));
        panics.capture(|| self.clear(false));
        panics.finish(Ok(()));
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Copies into the existing key with native assignment semantics; `map.substitute_ref(1, &7)?`.
    pub fn substitute_ref(&mut self, index: usize, key: &K) -> Result<(), Failure> {
        self.check_index(index)?;
        let (existing, bucket) = self.lookup_with_bucket(key);
        self.check_substitute(index, existing, key)?;
        let node = self.storage.node(index);
        if existing == 0 {
            self.storage
                .unlink(self.hash_bucket(self.storage.key(index)), node);
        }
        self.replace_substitute(index, bucket, existing == 0, |stored| {
            stored.clone_from(key)
        });
        Ok(())
    }
}
