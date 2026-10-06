//! Checked legacy int overloads preserve OCCT's negative-input behavior.
use super::super::hash_policy::buckets_from_int;
use super::{BaseAllocator, Debug, Failure, HashPolicy, IndexedMap};

impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Checks a signed initial count before creating the map; `IndexedMap::with_int_buckets(policy, 3, arena)?`.
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

    /// Treats negative resize requests as zero, as OCCT does; `map.resize_int(-1)`.
    pub fn resize_int(&mut self, requested: i32) {
        self.resize(requested.max(0) as usize);
    }

    /// Borrows a key through the signed index overload; `map.find_key_int(1)?`.
    pub fn find_key_int(&self, index: i32) -> Result<&K, Failure> {
        self.find_key(self.checked_signed_index(index)?)
    }

    /// Replaces a signed index after validating its range; `map.substitute_int(1, 42)?`.
    pub fn substitute_int(&mut self, index: i32, key: K) -> Result<(), Failure> {
        self.substitute(self.checked_signed_index(index)?, key)
    }

    /// Exchanges two checked signed indices; `map.swap_int(1, 2)?`.
    pub fn swap_int(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.swap(
            self.checked_signed_index(first)?,
            self.checked_signed_index(second)?,
        )
    }

    /// Removes a signed index with native swap-last behavior; `map.remove_from_index_int(1)?`.
    pub fn remove_from_index_int(&mut self, index: i32) -> Result<(), Failure> {
        self.remove_from_index(self.checked_signed_index(index)?)
    }

    fn checked_signed_index(&self, index: i32) -> Result<usize, Failure> {
        let converted = usize::try_from(index).map_err(|_| {
            Failure::with_type(
                "Standard_OutOfRange",
                &format!("IndexedMap index {index}; expected 1..={}", self.len()),
                "",
            )
        })?;
        self.check_index(converted)?;
        Ok(converted)
    }
}
