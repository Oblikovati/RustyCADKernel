//! IndexedMap and IndexedDataMap keep the entry of index `i` in slot `i - 1`. They remove only
//! the last slot (`RemoveLast`; `RemoveFromIndex` swaps first) and trade two slots on `Swap`, so
//! the slot vector itself is the native index table `myData2`, in index order. The bucket chains
//! and the hashing order stay those of native `NCollection_IndexedMap`.
use super::ChainedSlots;
use crate::{
    ncollection::{BaseMap, HashPolicy},
    standard::Failure,
};
use std::fmt::Debug;

impl<K, V> ChainedSlots<K, V> {
    /// Standard_OutOfRange unless `index` is in `1..=Extent()`; `kind` names the map.
    pub(in crate::ncollection) fn check_index(
        &self,
        index: usize,
        kind: &str,
    ) -> Result<(), Failure> {
        if index == 0 || index > self.size() {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!("{kind} index {index}; expected 1..={}", self.size()),
                "",
            ));
        }
        Ok(())
    }
    /// Native `lookup(key, node, hash)`: always hashes; returns the bound index (0 when absent)
    /// and the key's bucket. Callers prepared a table.
    pub(in crate::ncollection) fn find_index(
        &self,
        key: &K,
        hasher: &impl HashPolicy<K>,
    ) -> (usize, usize) {
        let (bucket, slot) = self.lookup_with_bucket(key, hasher);
        (slot.map_or(0, |slot| slot + 1), bucket)
    }
    /// Appends a new last index in `bucket`, constructing the value before the key.
    pub(in crate::ncollection) fn append(
        &mut self,
        bucket: usize,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> usize {
        self.insert(bucket, construct_key, construct_value) + 1
    }
    /// Native `Swap` of two valid, distinct indices: entries trade places, chains keep them.
    pub(in crate::ncollection) fn swap_indices(&mut self, first: usize, second: usize) {
        self.chains.swap_slots(first - 1, second - 1);
        self.payload.swap(first - 1, second - 1);
    }
    /// Native `RemoveLast` of a nonempty map: hashes the last key to find its chain (the slot
    /// records it, so the call keeps only the native hasher sequence), unlinks it, then drops
    /// the key and value once the map is consistent.
    pub(in crate::ncollection) fn remove_last(&mut self, hasher: &impl HashPolicy<K>) {
        let last = self.size() - 1;
        let _ = hasher.hash(self.payload.key(last));
        drop(self.remove(last));
    }
    /// The key, value and exclusive value of a valid index.
    pub(in crate::ncollection) fn key_at(&self, index: usize) -> &K {
        self.payload.key(index - 1)
    }
    pub(in crate::ncollection) fn value_at(&self, index: usize) -> &V {
        self.payload.value(index - 1)
    }
    pub(in crate::ncollection) fn value_at_mut(&mut self, index: usize) -> &mut V {
        self.payload.value_mut(index - 1)
    }
}

impl<K: Debug, V> ChainedSlots<K, V> {
    /// Native `Substitute` up to the assignment: hashes the new key; a key bound at another
    /// index is Standard_DomainError. When the key is new, hashes the old key and unlinks the
    /// slot. Returns the new key's bucket and whether the slot was unlinked.
    pub(in crate::ncollection) fn prepare_substitute(
        &mut self,
        index: usize,
        key: &K,
        hasher: &impl HashPolicy<K>,
        kind: &str,
    ) -> Result<(usize, bool), Failure> {
        self.check_index(index, kind)?;
        let (existing, bucket) = self.find_index(key, hasher);
        check_substitute(index, existing, key, kind)?;
        if existing == 0 {
            let _ = hasher.hash(self.key_at(index));
            self.chains.unlink(index - 1);
        }
        Ok((bucket, existing == 0))
    }
    /// Relinks an unlinked slot at the head of `bucket`, then assigns its new payload. Native
    /// assigns first; relinking first keeps a panicking assignment from leaving the slot
    /// unchained, and no caller can observe the order in between.
    pub(in crate::ncollection) fn replace_substitute(
        &mut self,
        index: usize,
        (bucket, detached): (usize, bool),
        assign: impl FnOnce(&mut K, &mut V),
    ) {
        if detached {
            self.chains.relink_front(index - 1, bucket);
        }
        let (key, value) = self.payload.pair_mut(index - 1);
        assign(key, value);
    }
}

fn check_substitute<K: Debug>(
    index: usize,
    existing: usize,
    key: &K,
    kind: &str,
) -> Result<(), Failure> {
    if existing == 0 || existing == index {
        return Ok(());
    }
    Err(Failure::with_type(
        "Standard_DomainError",
        &format!(
            "{kind} key {key:?} already at {existing}; expected an absent key or index {index}"
        ),
        "",
    ))
}
