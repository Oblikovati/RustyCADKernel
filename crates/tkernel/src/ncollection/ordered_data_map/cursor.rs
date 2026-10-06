use super::{Failure, HashPolicy, OrderedDataCursor, OrderedDataMap};
impl<K, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Creates a detached legacy cursor for explicit reborrowing; `map.cursor()`.
    pub fn cursor(&self) -> OrderedDataCursor {
        OrderedDataCursor {
            slot: self.table.first,
            owner: std::ptr::from_ref(self).cast(),
            node: self.cursor_node_identity(self.table.first),
        }
    }
    fn cursor_node_identity(&self, slot: Option<usize>) -> *const () {
        self.checked_node_slot(slot)
            .map_or(std::ptr::null(), |slot| {
                self.table.node_pointer(slot).cast_const().cast()
            })
    }
}
impl OrderedDataCursor {
    /// Rebinds the cursor to `map`'s first binding, native `Initialize`; `cursor.initialize(&map)`.
    pub fn initialize<K, V, H: HashPolicy<K>>(&mut self, map: &OrderedDataMap<K, V, H>) {
        *self = map.cursor();
    }
    /// Native `Reset`: a bound cursor restarts at the current first binding of `map` (the map
    /// it was created on), a default cursor stays exhausted; `cursor.reset(&map)`.
    pub fn reset<K, V, H: HashPolicy<K>>(&mut self, map: &OrderedDataMap<K, V, H>) {
        if self.owner.is_null() {
            return;
        }
        assert_eq!(
            self.owner,
            std::ptr::from_ref(map).cast(),
            "OrderedDataCursor reset map; expected original owner"
        );
        *self = map.cursor();
    }
    /// Checks whether the cursor is exhausted; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.slot.is_some()
    }
    /// Borrows the current key from its map; `cursor.key(&map)?`.
    pub fn key<'a, K, V, H: HashPolicy<K>>(
        &self,
        map: &'a OrderedDataMap<K, V, H>,
    ) -> Result<&'a K, Failure> {
        Ok(map
            .table
            .key(map.checked_node_slot(self.checked_slot(map)?)?))
    }
    /// Borrows the current value from its map; `cursor.value(&map)?`.
    pub fn value<'a, K, V, H: HashPolicy<K>>(
        &self,
        map: &'a OrderedDataMap<K, V, H>,
    ) -> Result<&'a V, Failure> {
        Ok(map
            .table
            .value(map.checked_node_slot(self.checked_slot(map)?)?))
    }
    /// Borrows the current value exclusively; `*cursor.change_value(&mut map)?=42`.
    pub fn change_value<'a, K, V, H: HashPolicy<K>>(
        &self,
        map: &'a mut OrderedDataMap<K, V, H>,
    ) -> Result<&'a mut V, Failure> {
        let slot = self.checked_slot(map)?;
        let slot = map.checked_node_slot(slot)?;
        Ok(map.table.value_mut(slot))
    }
    /// Advances through insertion-order links; `cursor.advance(&map)`.
    pub fn advance<K, V, H: HashPolicy<K>>(&mut self, map: &OrderedDataMap<K, V, H>) {
        self.slot = self
            .checked_slot(map)
            .ok()
            .and_then(|slot| map.table.next_slot(map.checked_node_slot(slot).ok()?));
        self.node = map.cursor_node_identity(self.slot);
    }

    fn checked_slot<K, V, H: HashPolicy<K>>(
        &self,
        map: &OrderedDataMap<K, V, H>,
    ) -> Result<Option<usize>, Failure> {
        let owner = std::ptr::from_ref(map).cast();
        let node = map.cursor_node_identity(self.slot);
        if owner == self.owner && !node.is_null() && node == self.node {
            return Ok(self.slot);
        }
        Err(Failure::with_type(
            "Standard_NoSuchObject",
            &format!(
                "OrderedDataCursor owner {owner:p}, node {node:p}; expected owner {:p}, live node {:p}",
                self.owner, self.node
            ),
            "",
        ))
    }

    /// Copies cursor state and returns the destination; `cursor.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        self.clone_from(other);
        self
    }

    /// Preserves self-assignment safely; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }

    /// Compares native node identity, treating exhausted cursors as equal; `cursor.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }

    /// Advances and returns this cursor, matching prefix increment; `cursor.advance_returning(&map)`.
    pub fn advance_returning<K, V, H: HashPolicy<K>>(
        &mut self,
        map: &OrderedDataMap<K, V, H>,
    ) -> &mut Self {
        self.advance(map);
        self
    }

    /// Advances while returning the old state, matching postfix increment; `cursor.post_advance(&map)`.
    pub fn post_advance<K, V, H: HashPolicy<K>>(&mut self, map: &OrderedDataMap<K, V, H>) -> Self {
        let previous = self.clone();
        self.advance(map);
        previous
    }
}
