//! Native `ReSize`: a new table of the next prime modulus, filled by prepending in old order.
use super::ChainedSlots;
use crate::ncollection::HashPolicy;

impl<K, V> ChainedSlots<K, V> {
    /// Grows to the native prime step for `extent`; a no-op keeps the table and skips hashing.
    /// Every key is hashed before any link changes, so a hasher panic leaves the map intact.
    pub(in crate::ncollection) fn resize(&mut self, extent: usize, hasher: &impl HashPolicy<K>) {
        let payload = &self.payload;
        self.chains.rehash(extent, &mut self.bucket_count, |slot| {
            hasher.hash(payload.key(slot))
        });
    }
}
