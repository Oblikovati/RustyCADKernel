//! OCCT's signed index and bucket overloads reject negative arguments.
use super::super::hash_policy::buckets_from_int;
use super::{BaseAllocator, Debug, Failure, HashPolicy, IndexedDataMap};

impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Creates a map after checking a signed bucket count; `IndexedDataMap::with_int_buckets(policy, 3, arena)?`.
    pub fn with_int_buckets(
        hasher: H,
        buckets: i32,
        resource: BaseAllocator,
    ) -> Result<Self, Failure> {
        Ok(Self::with_allocator(
            hasher,
            buckets_from_int(buckets)?,
            resource,
        ))
    }

    /// Rejects negative requests before resizing; `map.resize_int(101)?`.
    pub fn resize_int(&mut self, requested: i32) -> Result<(), Failure> {
        self.resize(buckets_from_int(requested)?);
        Ok(())
    }

    /// Borrows a key through the legacy int overload; `map.find_key_int(1)?`.
    pub fn find_key_int(&self, index: i32) -> Result<&K, Failure> {
        self.keys.find_key_int(index)
    }

    /// Borrows an item through the legacy int overload; `map.find_from_index_int(1)?`.
    pub fn find_from_index_int(&self, index: i32) -> Result<&V, Failure> {
        self.keys.find_key_int(index)?;
        self.find_from_index(index as usize)
    }

    /// Mutably borrows an item through the legacy int overload; `*map.change_from_index_int(1)? = 7`.
    pub fn change_from_index_int(&mut self, index: i32) -> Result<&mut V, Failure> {
        self.keys.find_key_int(index)?;
        self.change_from_index(index as usize)
    }

    /// Replaces an indexed binding after checking its signed index; `map.substitute_int(1, 7, 42)?`.
    pub fn substitute_int(&mut self, index: i32, key: K, value: V) -> Result<(), Failure> {
        self.keys.find_key_int(index)?;
        self.substitute(index as usize, key, value)
    }

    /// Swaps bindings after validating both signed indices; `map.swap_int(1, 2)?`.
    pub fn swap_int(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.keys.find_key_int(first)?;
        self.keys.find_key_int(second)?;
        self.swap(first as usize, second as usize)
    }

    /// Removes a binding using the signed swap-last overload; `map.remove_from_index_int(1)?`.
    pub fn remove_from_index_int(&mut self, index: i32) -> Result<(), Failure> {
        self.keys.find_key_int(index)?;
        self.remove_from_index(index as usize)
    }
}
