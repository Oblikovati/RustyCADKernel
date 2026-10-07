//! Ranges borrow the map and lend its entries in native bucket/chain order.
use super::{ChainPosition, ChainSlot, ChainedSlots};

/// Shared traversal; cursors compare by entry identity (map and slot), as native nodes do.
pub(in crate::ncollection) struct SlotRange<'a, K, V> {
    source: Option<&'a ChainedSlots<K, V>>,
    position: ChainPosition,
}

impl<'a, K, V> SlotRange<'a, K, V> {
    pub(in crate::ncollection) fn new(source: &'a ChainedSlots<K, V>) -> Self {
        Self::from_position(Some(source), ChainPosition::first(source))
    }
    pub(in crate::ncollection) fn from_position(
        source: Option<&'a ChainedSlots<K, V>>,
        position: ChainPosition,
    ) -> Self {
        Self { source, position }
    }
    /// The entry under the cursor; `None` when exhausted or unbound.
    pub(in crate::ncollection) fn current(&self) -> Option<&'a ChainSlot<K, V>> {
        Some(self.source?.slot(self.position.slot()?))
    }
    /// Restarts on the bound map; an unbound cursor stays exhausted.
    pub(in crate::ncollection) fn reset(&mut self) {
        if let Some(source) = self.source {
            self.position = ChainPosition::first(source);
        }
    }
    // Native node identity: the owning map plus the slot, or nothing once exhausted.
    fn identity(&self) -> Option<(*const ChainedSlots<K, V>, usize)> {
        Some((std::ptr::from_ref(self.source?), self.position.slot()?))
    }
}

impl<'a, K, V> Iterator for SlotRange<'a, K, V> {
    type Item = &'a ChainSlot<K, V>;
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.current()?;
        self.position.advance(self.source?);
        Some(entry)
    }
}

impl<K, V> Default for SlotRange<'_, K, V> {
    fn default() -> Self {
        Self::from_position(None, ChainPosition::default())
    }
}

impl<K, V> Clone for SlotRange<'_, K, V> {
    fn clone(&self) -> Self {
        Self::from_position(self.source, self.position)
    }
}

impl<K, V> PartialEq for SlotRange<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}

impl<K, V> Eq for SlotRange<'_, K, V> {}

/// Exclusive traversal lending each value once; the order is fixed when the range starts.
pub(in crate::ncollection) struct SlotRangeMut<'a, K, V> {
    entries: Vec<Option<&'a mut ChainSlot<K, V>>>,
    order: std::vec::IntoIter<usize>,
}

impl<'a, K, V> SlotRangeMut<'a, K, V> {
    pub(in crate::ncollection) fn new(source: &'a mut ChainedSlots<K, V>) -> Self {
        let order = source.chain_order().into_iter();
        Self {
            entries: source.slots.iter_mut().map(Some).collect(),
            order,
        }
    }
}

impl<'a, K, V> Iterator for SlotRangeMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.order.next()?;
        self.entries[slot].take().map(ChainSlot::key_value_mut)
    }
}
