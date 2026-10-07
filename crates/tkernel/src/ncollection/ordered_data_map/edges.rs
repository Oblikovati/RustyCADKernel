use super::{
    Failure, HashPolicy, OrderedDataMap, OrderedNodeMut, OrderedNodeRef, empty_ordered_data,
};
use crate::ncollection::HashBucketTable;
impl<K, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Native `Buckets()`: the key table with each chain's `(key, item)` pairs, or `None`
    /// where native returns null; the insertion-order links stay private.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::OrderedDataMap;
    /// let mut map = OrderedDataMap::<i32, i32>::default();
    /// map.bind(7, 70);
    /// assert_eq!(map.buckets().unwrap().chain(8)?.collect::<Vec<_>>(), [(&7, &70)]);
    /// # Ok::<(), rustycad_tkernel::standard::Failure>(())
    /// ```
    pub fn buckets(&self) -> Option<HashBucketTable<'_, K, V>> {
        self.table.buckets()
    }
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
        Ok(self.table.key(self.checked_node_slot(self.table.first())?))
    }
    /// Borrows the last inserted remaining key; `map.last()?`.
    pub fn last(&self) -> Result<&K, Failure> {
        Ok(self.table.key(self.checked_node_slot(self.table.last())?))
    }
    /// Borrows the first remaining value; `map.first_value()?`.
    pub fn first_value(&self) -> Result<&V, Failure> {
        Ok(self
            .table
            .value(self.checked_node_slot(self.table.first())?))
    }
    /// Borrows the last remaining value; `map.last_value()?`.
    pub fn last_value(&self) -> Result<&V, Failure> {
        Ok(self.table.value(self.checked_node_slot(self.table.last())?))
    }
    /// Borrows the first remaining value exclusively; `*map.change_first_value()?=42`.
    pub fn change_first_value(&mut self) -> Result<&mut V, Failure> {
        let slot = self.checked_node_slot(self.table.first())?;
        Ok(self.table.value_mut(slot))
    }
    /// Borrows the last remaining value exclusively; `*map.change_last_value()?=42`.
    pub fn change_last_value(&mut self) -> Result<&mut V, Failure> {
        let slot = self.checked_node_slot(self.table.last())?;
        Ok(self.table.value_mut(slot))
    }
    // Native First()/Last() raise Standard_NoSuchObject on an empty map.
    fn checked_node_slot(&self, slot: Option<usize>) -> Result<usize, Failure> {
        slot.ok_or_else(empty_ordered_data)
    }
}
