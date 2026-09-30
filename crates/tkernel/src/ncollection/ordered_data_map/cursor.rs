use super::{Debug, Failure, HashPolicy, OrderedDataCursor, OrderedDataMap};
impl<K: Debug, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Creates a detached legacy cursor for explicit reborrowing; `map.cursor()`.
    pub fn cursor(&self) -> OrderedDataCursor {
        OrderedDataCursor {
            slot: self.table.first,
            bound: true,
        }
    }
}
impl OrderedDataCursor {
    /// Rebinds the cursor to `map`'s first binding, native `Initialize`; `cursor.initialize(&map)`.
    pub fn initialize<K: Debug, V, H: HashPolicy<K>>(&mut self, map: &OrderedDataMap<K, V, H>) {
        *self = map.cursor();
    }
    /// Native `Reset`: a bound cursor restarts at the current first binding of `map` (the map
    /// it was created on), a default cursor stays exhausted; `cursor.reset(&map)`.
    pub fn reset<K: Debug, V, H: HashPolicy<K>>(&mut self, map: &OrderedDataMap<K, V, H>) {
        if self.bound {
            self.slot = map.table.first;
        }
    }
    /// Checks whether the cursor is exhausted; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.slot.is_some()
    }
    /// Borrows the current key from its map; `cursor.key(&map)?`.
    pub fn key<'a, K: Debug, V, H: HashPolicy<K>>(
        &self,
        map: &'a OrderedDataMap<K, V, H>,
    ) -> Result<&'a K, Failure> {
        Ok(&map.node(self.slot)?.key)
    }
    /// Borrows the current value from its map; `cursor.value(&map)?`.
    pub fn value<'a, K: Debug, V, H: HashPolicy<K>>(
        &self,
        map: &'a OrderedDataMap<K, V, H>,
    ) -> Result<&'a V, Failure> {
        Ok(&map.node(self.slot)?.value)
    }
    /// Borrows the current value exclusively; `*cursor.change_value(&mut map)?=42`.
    pub fn change_value<'a, K: Debug, V, H: HashPolicy<K>>(
        &self,
        map: &'a mut OrderedDataMap<K, V, H>,
    ) -> Result<&'a mut V, Failure> {
        Ok(&mut map.node_mut(self.slot)?.value)
    }
    /// Advances through insertion-order links; `cursor.advance(&map)`.
    pub fn advance<K: Debug, V, H: HashPolicy<K>>(&mut self, map: &OrderedDataMap<K, V, H>) {
        self.slot = map.node(self.slot).ok().and_then(|node| node.next);
    }
}
