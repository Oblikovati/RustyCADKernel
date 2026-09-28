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

use super::{
    Debug, Failure, HashPolicy, OrderedDataMap, OrderedNode, OrderedNodeMut, OrderedNodeRef,
    missing_ordered_node,
};
impl<K: Debug, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// The head of the insertion order as a node view, `None` when empty; `map.first_node()`.
    pub fn first_node(&self) -> Option<OrderedNodeRef<'_, K, V>> {
        OrderedNodeRef::at(&self.table.slots, self.table.first)
    }
    /// The head node with an exclusive item, `None` when empty; `map.first_node_mut()`.
    pub fn first_node_mut(&mut self) -> Option<OrderedNodeMut<'_, K, V>> {
        OrderedNodeMut::at(&mut self.table.slots, self.table.first)
    }
    /// Borrows the first inserted remaining key; `map.first()?`.
    pub fn first(&self) -> Result<&K, Failure> {
        Ok(&self.node(self.table.first)?.key)
    }
    /// Borrows the last inserted remaining key; `map.last()?`.
    pub fn last(&self) -> Result<&K, Failure> {
        Ok(&self.node(self.table.last)?.key)
    }
    /// Borrows the first remaining value; `map.first_value()?`.
    pub fn first_value(&self) -> Result<&V, Failure> {
        Ok(&self.node(self.table.first)?.value)
    }
    /// Borrows the last remaining value; `map.last_value()?`.
    pub fn last_value(&self) -> Result<&V, Failure> {
        Ok(&self.node(self.table.last)?.value)
    }
    /// Borrows the first remaining value exclusively; `*map.change_first_value()?=42`.
    pub fn change_first_value(&mut self) -> Result<&mut V, Failure> {
        Ok(&mut self.node_mut(self.table.first)?.value)
    }
    /// Borrows the last remaining value exclusively; `*map.change_last_value()?=42`.
    pub fn change_last_value(&mut self) -> Result<&mut V, Failure> {
        Ok(&mut self.node_mut(self.table.last)?.value)
    }
    pub(super) fn node(&self, slot: Option<usize>) -> Result<&OrderedNode<K, V>, Failure> {
        slot.and_then(|index| self.table.slots.get(index)?.as_deref())
            .ok_or_else(|| missing_ordered_node(slot))
    }
    pub(super) fn node_mut(
        &mut self,
        slot: Option<usize>,
    ) -> Result<&mut OrderedNode<K, V>, Failure> {
        slot.and_then(|index| self.table.slots.get_mut(index)?.as_deref_mut())
            .ok_or_else(|| missing_ordered_node(slot))
    }
}
