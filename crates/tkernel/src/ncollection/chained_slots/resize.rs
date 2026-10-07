//! Native `ReSize`: a new table of the next prime modulus, filled by prepending in old order.
use super::{ChainedSlots, SlotLink};
use crate::ncollection::{
    HashPolicy,
    base_map::{planned_bucket_count, table_slot_count},
};
use crate::standard::Failure;

impl<K, V> ChainedSlots<K, V> {
    /// Grows to the native prime step for `extent`; a no-op keeps the table and skips hashing.
    /// Every key is hashed before any link changes, so a hasher panic leaves the map intact.
    pub(in crate::ncollection) fn resize(&mut self, extent: usize, hasher: &impl HashPolicy<K>) {
        let Some(count) = planned_bucket_count(extent, self.bucket_count, self.heads.is_some())
        else {
            return;
        };
        // Pre-existing D3 site: ReSize has no failure channel; an unaddressable table panics
        // with the native Standard_OutOfMemory failure.
        let mut heads = zeroed_heads(count).unwrap_or_else(|failure| panic!("{failure}"));
        for (slot, bucket) in self.rehash_destinations(count, hasher) {
            let entry = &mut self.slots[slot];
            entry.next = heads[bucket].replace(slot);
            entry.bucket = bucket;
        }
        // The new table exists before the old one is freed, so their identities differ.
        self.heads = Some(heads);
        self.bucket_count = count;
    }
    fn rehash_destinations(
        &self,
        count: usize,
        hasher: &impl HashPolicy<K>,
    ) -> Vec<(usize, usize)> {
        self.chain_order()
            .into_iter()
            .map(|slot| (slot, hasher.hash(&self.slots[slot].key) % count + 1))
            .collect()
    }
}

fn zeroed_heads(bucket_count: usize) -> Result<Box<[SlotLink]>, Failure> {
    let length = table_slot_count(bucket_count, 1)?;
    let mut heads = Vec::new();
    heads.try_reserve_exact(length).map_err(|_| {
        Failure::with_message(
            "Standard_OutOfMemory",
            &format!(
                "BaseMap bucket allocation of {length} heads failed; expected addressable table"
            ),
        )
    })?;
    heads.resize(length, None);
    Ok(heads.into_boxed_slice())
}
