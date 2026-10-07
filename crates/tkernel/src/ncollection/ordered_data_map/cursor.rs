//! `NCollection_OrderedDataMap::Iterator` as a detached cursor: it holds the map identity and
//! the never-reused identity of its current entry, and reborrows the map for every access, so a
//! cursor whose entry was removed reports Standard_NoSuchObject instead of dangling.
use super::{Failure, HashPolicy, OrderedDataCursor, OrderedDataMap};
impl<K, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Creates a detached legacy cursor for explicit reborrowing; `map.cursor()`.
    pub fn cursor(&self) -> OrderedDataCursor {
        OrderedDataCursor {
            owner: std::ptr::from_ref(self).cast(),
            entry: self.table.first().map(|slot| self.table.serial(slot)),
        }
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
        self.entry.is_some()
    }
    /// Borrows the current key from its map; `cursor.key(&map)?`.
    pub fn key<'a, K, V, H: HashPolicy<K>>(
        &self,
        map: &'a OrderedDataMap<K, V, H>,
    ) -> Result<&'a K, Failure> {
        Ok(map.table.key(self.checked_slot(map)?))
    }
    /// Borrows the current value from its map; `cursor.value(&map)?`.
    pub fn value<'a, K, V, H: HashPolicy<K>>(
        &self,
        map: &'a OrderedDataMap<K, V, H>,
    ) -> Result<&'a V, Failure> {
        Ok(map.table.value(self.checked_slot(map)?))
    }
    /// Borrows the current value exclusively; `*cursor.change_value(&mut map)?=42`.
    pub fn change_value<'a, K, V, H: HashPolicy<K>>(
        &self,
        map: &'a mut OrderedDataMap<K, V, H>,
    ) -> Result<&'a mut V, Failure> {
        let slot = self.checked_slot(map)?;
        Ok(map.table.value_mut(slot))
    }
    /// Advances through insertion-order links; a cursor off its map's live entries becomes
    /// exhausted; `cursor.advance(&map)`.
    pub fn advance<K, V, H: HashPolicy<K>>(&mut self, map: &OrderedDataMap<K, V, H>) {
        let next = self
            .checked_slot(map)
            .ok()
            .and_then(|slot| map.table.next_slot(slot));
        self.entry = next.map(|slot| map.table.serial(slot));
    }
    // The live slot of the current entry of `map`, or Standard_NoSuchObject.
    fn checked_slot<K, V, H: HashPolicy<K>>(
        &self,
        map: &OrderedDataMap<K, V, H>,
    ) -> Result<usize, Failure> {
        let owner: *const () = std::ptr::from_ref(map).cast();
        let slot = (owner == self.owner)
            .then_some(self.entry)
            .flatten()
            .and_then(|entry| map.table.slot_of(entry));
        slot.ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                &format!(
                    "OrderedDataCursor owner {owner:p}, entry {:?}; expected owner {:p} and a live entry",
                    self.entry, self.owner
                ),
                "",
            )
        })
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
