//! Borrowed traversal along the insertion-order links.
use super::{OrderSlot, OrderedTable};
use crate::ncollection::slot_chains::{SlotLink, SlotRangeMut};

/// The order links and payload of one table, without its policy.
pub(in crate::ncollection) struct OrderView<'a, K, V> {
    order: &'a [OrderSlot],
    keys: &'a [K],
    values: &'a [V],
    first: SlotLink,
}

impl<K, V> Clone for OrderView<'_, K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for OrderView<'_, K, V> {}

/// Shared traversal in insertion order; cursors compare by entry identity, as native nodes do.
pub(in crate::ncollection) struct OrderedItems<'a, K, V> {
    view: Option<OrderView<'a, K, V>>,
    current: SlotLink,
}

/// Exclusive traversal in insertion order, lending each value once.
pub(in crate::ncollection) type OrderedItemsMut<'a, K, V> = SlotRangeMut<'a, K, V>;

impl<K, V, H> OrderedTable<K, V, H> {
    pub fn items(&self) -> OrderedItems<'_, K, V> {
        let view = OrderView {
            order: &self.order,
            keys: self.storage.payload().keys(),
            values: self.storage.payload().values(),
            first: self.first,
        };
        OrderedItems {
            view: Some(view),
            current: self.first,
        }
    }
    pub fn items_mut(&mut self) -> OrderedItemsMut<'_, K, V> {
        let order = self.order_slots();
        SlotRangeMut::new(order, self.storage.payload_mut())
    }
    // Every slot from the head along `myOrderNext`.
    fn order_slots(&self) -> Vec<usize> {
        let mut slots = Vec::with_capacity(self.order.len());
        let mut link = self.first;
        while let Some(slot) = link {
            slots.push(slot);
            link = self.order[slot].next;
        }
        slots
    }
}

impl<'a, K, V> OrderedItems<'a, K, V> {
    /// Native `More()`: the cursor is on an entry.
    pub fn more(&self) -> bool {
        self.current.is_some()
    }
    pub fn current_pair(&self) -> Option<(&'a K, &'a V)> {
        let (view, slot) = (self.view?, self.current?);
        Some((&view.keys[slot], &view.values[slot]))
    }
    /// Native `Reset`: back to the head the cursor was created on; a default cursor stays empty.
    pub fn restart(&mut self) {
        self.current = self.view.and_then(|view| view.first);
    }
    /// Native `Next()`: follow `myOrderNext`.
    pub fn advance(&mut self) {
        if let (Some(view), Some(slot)) = (self.view, self.current) {
            self.current = view.order[slot].next;
        }
    }
    /// Native `myOrderPrev`: step back to the entry inserted before the current one.
    pub fn retreat(&mut self) {
        if let (Some(view), Some(slot)) = (self.view, self.current) {
            self.current = view.order[slot].previous;
        }
    }
    // Native node identity: the owning table plus the slot, or nothing once exhausted.
    fn identity(&self) -> Option<(*const OrderSlot, usize)> {
        Some((self.view?.order.as_ptr(), self.current?))
    }
}

impl<'a, K, V> Iterator for OrderedItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let pair = self.current_pair()?;
        self.advance();
        Some(pair)
    }
}

impl<K, V> Clone for OrderedItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            view: self.view,
            current: self.current,
        }
    }
}

impl<K, V> Default for OrderedItems<'_, K, V> {
    fn default() -> Self {
        Self {
            view: None,
            current: None,
        }
    }
}

impl<K, V> PartialEq for OrderedItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}

impl<K, V> Eq for OrderedItems<'_, K, V> {}
