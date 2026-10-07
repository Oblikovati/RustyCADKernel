//! Native `lookup`: `Hasher(key) % NbBuckets + 1` selects the bucket, then the chain is compared.
use super::ChainedSlots;
use crate::ncollection::{BaseMap, HashPolicy, slot_chains::SlotLink};

impl<K, V> ChainedSlots<K, V> {
    /// Finds a bound key; an empty map skips the user hasher, as native `IsEmpty()` checks do.
    pub(in crate::ncollection) fn lookup(
        &self,
        key: &K,
        hasher: &impl HashPolicy<K>,
    ) -> Option<(usize, usize)> {
        if self.size() == 0 {
            return None;
        }
        let (bucket, slot) = self.lookup_with_bucket(key, hasher);
        slot.map(|slot| (bucket, slot))
    }
    /// Hashes once and returns the bucket with the matching slot; callers prepared a table.
    pub(in crate::ncollection) fn lookup_with_bucket(
        &self,
        key: &K,
        hasher: &impl HashPolicy<K>,
    ) -> (usize, SlotLink) {
        let bucket = hasher.hash(key) % self.bucket_count + 1;
        let slot = self
            .chains
            .find(bucket, |slot| hasher.equal(self.payload.key(slot), key));
        (bucket, slot)
    }
}
