//! Ranges borrow a map and lend its entries in native bucket/chain order.
use super::{ChainPosition, ChainSlot, SlotChains};

/// One bucket table together with the slots its chains index; the shared view of a map.
pub(in crate::ncollection) struct ChainView<'a, K, V> {
    chains: &'a SlotChains,
    slots: &'a [ChainSlot<K, V>],
}

impl<'a, K, V> ChainView<'a, K, V> {
    pub(in crate::ncollection) fn new(
        chains: &'a SlotChains,
        slots: &'a [ChainSlot<K, V>],
    ) -> Self {
        Self { chains, slots }
    }
    pub(in crate::ncollection) fn chains(&self) -> &'a SlotChains {
        self.chains
    }
    pub(in crate::ncollection) fn slot(&self, index: usize) -> &'a ChainSlot<K, V> {
        &self.slots[index]
    }
}

impl<K, V> Clone for ChainView<'_, K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for ChainView<'_, K, V> {}

/// Shared traversal; cursors compare by entry identity (map and slot), as native nodes do.
pub(in crate::ncollection) struct SlotRange<'a, K, V> {
    view: Option<ChainView<'a, K, V>>,
    position: ChainPosition,
}

impl<'a, K, V> SlotRange<'a, K, V> {
    pub(in crate::ncollection) fn new(view: ChainView<'a, K, V>) -> Self {
        Self::from_position(Some(view), ChainPosition::first(view.chains))
    }
    pub(in crate::ncollection) fn from_position(
        view: Option<ChainView<'a, K, V>>,
        position: ChainPosition,
    ) -> Self {
        Self { view, position }
    }
    /// The entry under the cursor; `None` when exhausted or unbound.
    pub(in crate::ncollection) fn current(&self) -> Option<&'a ChainSlot<K, V>> {
        Some(self.view?.slot(self.position.slot()?))
    }
    /// Restarts on the bound map; an unbound cursor stays exhausted.
    pub(in crate::ncollection) fn reset(&mut self) {
        if let Some(view) = self.view {
            self.position = ChainPosition::first(view.chains);
        }
    }
    // Native node identity: the owning table plus the slot, or nothing once exhausted.
    fn identity(&self) -> Option<(*const SlotChains, usize)> {
        Some((std::ptr::from_ref(self.view?.chains), self.position.slot()?))
    }
}

impl<'a, K, V> Iterator for SlotRange<'a, K, V> {
    type Item = &'a ChainSlot<K, V>;
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.current()?;
        self.position.advance(self.view?.chains);
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
        Self::from_position(self.view, self.position)
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
    /// Lends `slots` in `order` (a permutation of their indices).
    pub(in crate::ncollection) fn new(order: Vec<usize>, slots: &'a mut [ChainSlot<K, V>]) -> Self {
        Self {
            entries: slots.iter_mut().map(Some).collect(),
            order: order.into_iter(),
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
