// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Views of insertion-order nodes: the Rust spelling of `FirstNode()` and `OrderNext()`.
//!
//! Native maps hand out raw node pointers; here a view borrows the map's node slots and
//! follows the same insertion-order links, so traversal cannot outlive the map.
use super::OrderedNode;
use crate::ncollection::AllocatedBox;

type NodeSlot<K, V> = Option<AllocatedBox<OrderedNode<K, V>>>;

/// A shared view of one ordered-map node; `map.first_node().unwrap().key()`.
pub struct OrderedNodeRef<'a, K, V> {
    slots: &'a [NodeSlot<K, V>],
    slot: usize,
}

/// An exclusive view of one ordered-data-map node; `*map.first_node_mut().unwrap().change_value() = 7`.
pub struct OrderedNodeMut<'a, K, V> {
    slots: &'a mut [NodeSlot<K, V>],
    slot: usize,
}

impl<'a, K, V> OrderedNodeRef<'a, K, V> {
    /// A view of `slot`, or `None` for the null link that ends the order.
    pub(in crate::ncollection) fn at(
        slots: &'a [NodeSlot<K, V>],
        slot: Option<usize>,
    ) -> Option<Self> {
        Some(Self { slots, slot: slot? })
    }
    /// Borrows the node's key; `node.key()`.
    pub fn key(&self) -> &'a K {
        &self.node().key
    }
    /// Borrows the node's item (`()` for a key-only map); `node.value()`.
    pub fn value(&self) -> &'a V {
        &self.node().value
    }
    /// The next node in insertion order, `None` after the last; `node.order_next()`.
    pub fn order_next(&self) -> Option<Self> {
        Self::at(self.slots, self.node().next)
    }
    fn node(&self) -> &'a OrderedNode<K, V> {
        // Only live slots are ever linked into the insertion order.
        self.slots[self.slot]
            .as_deref()
            .expect("an insertion-order link names a live node")
    }
}

impl<'a, K, V> OrderedNodeMut<'a, K, V> {
    /// A view of `slot`, or `None` for the null link that ends the order.
    pub(in crate::ncollection) fn at(
        slots: &'a mut [NodeSlot<K, V>],
        slot: Option<usize>,
    ) -> Option<Self> {
        Some(Self { slots, slot: slot? })
    }
    /// Borrows the node's key; keys stay immutable as in native `Key() const`; `node.key()`.
    pub fn key(&self) -> &K {
        &self.node().key
    }
    /// Borrows the node's item; `node.value()`.
    pub fn value(&self) -> &V {
        &self.node().value
    }
    /// Borrows the node's item exclusively; `*node.change_value() = 7`.
    pub fn change_value(&mut self) -> &mut V {
        let slot = self.slot;
        &mut self.slots[slot]
            .as_deref_mut()
            .expect("an insertion-order link names a live node")
            .value
    }
    /// Moves the exclusive view to the next node in insertion order; `node.order_next()`.
    pub fn order_next(self) -> Option<Self> {
        let next = self.node().next;
        Self::at(self.slots, next)
    }
    fn node(&self) -> &OrderedNode<K, V> {
        self.slots[self.slot]
            .as_deref()
            .expect("an insertion-order link names a live node")
    }
}
