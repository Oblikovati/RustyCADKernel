//! Head insertion, removal and destruction keep the native chain and destructor order.
use super::ChainedSlots;
use crate::ncollection::map_payload::construct_value_then_key;

impl<K, V> ChainedSlots<K, V> {
    /// Constructs the value, then the key, and prepends the entry to `bucket`; returns its slot.
    /// A constructor panic leaves the map unchanged. The caller prepared a table.
    pub(in crate::ncollection) fn insert(
        &mut self,
        bucket: usize,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> usize {
        let (value, key) = construct_value_then_key(construct_value, construct_key);
        self.payload.push(key, value);
        self.chains.push_front(bucket)
    }
    /// Unlinks `target` and returns its pair for the caller to drop afterwards, so a panicking
    /// destructor finds the map already consistent (native `UnBind` order).
    pub(in crate::ncollection) fn remove(&mut self, target: usize) -> (K, V) {
        self.chains.unlink(target);
        self.chains.swap_remove(target);
        self.payload.swap_remove(target)
    }
    /// Native `destroy`: empties the map in bucket/chain order; `release` frees the table.
    pub(in crate::ncollection) fn clear(&mut self, release: bool) {
        let order = self.chains.order();
        let payload = std::mem::take(&mut self.payload);
        self.chains.reset(release);
        payload.drop_in_order(order);
    }
}
