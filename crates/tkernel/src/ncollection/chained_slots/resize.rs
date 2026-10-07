//! Native `ReSize`: a new table of the next prime modulus, filled by prepending in old order.
use super::ChainedSlots;
use crate::ncollection::{HashPolicy, base_map::planned_bucket_count};

impl<K, V> ChainedSlots<K, V> {
    /// Grows to the native prime step for `extent`; a no-op keeps the table and skips hashing.
    /// Every key is hashed before any link changes, so a hasher panic leaves the map intact.
    pub(in crate::ncollection) fn resize(&mut self, extent: usize, hasher: &impl HashPolicy<K>) {
        let allocated = self.chains.is_allocated();
        let Some(count) = planned_bucket_count(extent, self.bucket_count, allocated) else {
            return;
        };
        let routes: Vec<(usize, usize)> = self
            .chains
            .order()
            .into_iter()
            .map(|slot| (slot, hasher.hash(self.slots[slot].key()) % count + 1))
            .collect();
        self.chains.rebuild(count, &routes);
        self.bucket_count = count;
    }
}
