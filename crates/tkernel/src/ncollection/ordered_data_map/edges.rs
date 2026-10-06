use super::{
    Failure, HashPolicy, OrderedDataMap, OrderedNodeMut, OrderedNodeRef, missing_ordered_node,
};
impl<K, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// The head of the insertion order as a node view, `None` when empty; `map.first_node()`.
    pub fn first_node(&self) -> Option<OrderedNodeRef<'_, K, V>> {
        self.table.first_node()
    }
    /// The head node with an exclusive item, `None` when empty; `map.first_node_mut()`.
    pub fn first_node_mut(&mut self) -> Option<OrderedNodeMut<'_, K, V>> {
        self.table.first_node_mut()
    }
    /// Borrows the first inserted remaining key; `map.first()?`.
    pub fn first(&self) -> Result<&K, Failure> {
        Ok(self.table.key(self.checked_node_slot(self.table.first)?))
    }
    /// Borrows the last inserted remaining key; `map.last()?`.
    pub fn last(&self) -> Result<&K, Failure> {
        Ok(self.table.key(self.checked_node_slot(self.table.last)?))
    }
    /// Borrows the first remaining value; `map.first_value()?`.
    pub fn first_value(&self) -> Result<&V, Failure> {
        Ok(self.table.value(self.checked_node_slot(self.table.first)?))
    }
    /// Borrows the last remaining value; `map.last_value()?`.
    pub fn last_value(&self) -> Result<&V, Failure> {
        Ok(self.table.value(self.checked_node_slot(self.table.last)?))
    }
    /// Borrows the first remaining value exclusively; `*map.change_first_value()?=42`.
    pub fn change_first_value(&mut self) -> Result<&mut V, Failure> {
        let slot = self.checked_node_slot(self.table.first)?;
        Ok(self.table.value_mut(slot))
    }
    /// Borrows the last remaining value exclusively; `*map.change_last_value()?=42`.
    pub fn change_last_value(&mut self) -> Result<&mut V, Failure> {
        let slot = self.checked_node_slot(self.table.last)?;
        Ok(self.table.value_mut(slot))
    }
    pub(super) fn checked_node_slot(&self, slot: Option<usize>) -> Result<usize, Failure> {
        slot.filter(|&index| self.table.slots.get(index).is_some_and(Option::is_some))
            .ok_or_else(|| missing_ordered_node(slot))
    }
}
