//! `NCollection_IndexedDataMap::Iterator`, the remaining lookups and the allocator-replacing `Clear`.
use super::super::base_map::exhausted_iterator;
use super::super::{BaseAllocator, DefaultHashPolicy};
use super::{Debug, HashPolicy, IndexedDataMap};
use crate::standard::Failure;

/// A read-only OCCT `IndexedDataMap::Iterator` walking indices `1..=Extent()`.
pub struct IndexedDataMapIterator<'a, K, V, H = DefaultHashPolicy> {
    map: Option<&'a IndexedDataMap<K, V, H>>,
    index: usize,
}

/// A mutable OCCT `IndexedDataMap::Iterator` whose `change_value` edits items in place.
pub struct IndexedDataMapIteratorMut<'a, K, V, H = DefaultHashPolicy> {
    map: &'a mut IndexedDataMap<K, V, H>,
    index: usize,
}

impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// `TryBind`: binds an absent key only; true when a binding was added.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedDataMap;
    /// let mut map = IndexedDataMap::<i32, i32>::default();
    /// assert!(map.try_bind(1, 10) && !map.try_bind(1, 20));
    /// assert_eq!(map.seek(&1), Some(&10));
    /// ```
    pub fn try_bind(&mut self, key: K, value: V) -> bool {
        let old_len = self.len();
        self.place(key, || value, false) > old_len
    }

    /// `Contained() const`: the stored key and item of an equivalent key, if bound.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedDataMap;
    /// let mut map = IndexedDataMap::<i32, i32>::default();
    /// map.add(1, 10);
    /// assert_eq!(map.contained(&1), Some((&1, &10)));
    /// ```
    pub fn contained(&self, key: &K) -> Option<(&K, &V)> {
        let index = self.find_index(key).checked_sub(1)?;
        Some((
            self.keys.find_key(index + 1).ok()?,
            &self.values[index].value,
        ))
    }

    /// `Contained()`: the stored key and a mutable item of an equivalent key, if bound.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedDataMap;
    /// let mut map = IndexedDataMap::<i32, i32>::default();
    /// map.add(1, 10);
    /// *map.contained_mut(&1).unwrap().1 = 3;
    /// assert_eq!(map.seek(&1), Some(&3));
    /// ```
    pub fn contained_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        let index = self.find_index(key).checked_sub(1)?;
        let stored = self.keys.find_key(index + 1).ok()?;
        Some((stored, &mut self.values[index].value))
    }

    /// `Clear(const Handle(NCollection_BaseAllocator)&)`: clears, releasing storage only when
    /// the allocator changes; `None` (a null handle) selects the common allocator.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedDataMap;
    /// let mut map = IndexedDataMap::<i32, i32>::default();
    /// map.add(1, 10);
    /// map.clear_with_allocator(None);
    /// assert!(map.is_empty());
    /// ```
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        let release = resource.as_ref() != Some(self.allocator());
        self.keys.clear_with_allocator(resource);
        self.values.clear();
        if release {
            self.values.shrink_to_fit();
        }
    }

    /// `IndexedItems()`: keys, mutable items and one-based indices in index order.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedDataMap;
    /// let mut map = IndexedDataMap::<i32, i32>::default();
    /// map.add(1, 10);
    /// for (_, value, index) in map.indexed_items_mut() { *value += index as i32; }
    /// assert_eq!(map.seek(&1), Some(&11));
    /// ```
    pub fn indexed_items_mut(&mut self) -> impl Iterator<Item = (&K, &mut V, usize)> {
        self.items_mut()
            .enumerate()
            .map(|(index, (key, value))| (key, value, index + 1))
    }

    /// `Iterator(const NCollection_IndexedDataMap&)`: a cursor on index 1.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedDataMap;
    /// let mut map = IndexedDataMap::<i32, i32>::default();
    /// map.add(1, 10);
    /// assert_eq!(map.cursor().value().unwrap(), &10);
    /// ```
    pub fn cursor(&self) -> IndexedDataMapIterator<'_, K, V, H> {
        IndexedDataMapIterator {
            map: Some(self),
            index: 1,
        }
    }

    /// `Iterator(NCollection_IndexedDataMap&)`: a cursor on index 1 that may change items.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedDataMap;
    /// let mut map = IndexedDataMap::<i32, i32>::default();
    /// map.add(1, 10);
    /// *map.cursor_mut().change_value().unwrap() = 4;
    /// assert_eq!(map.seek(&1), Some(&4));
    /// ```
    pub fn cursor_mut(&mut self) -> IndexedDataMapIteratorMut<'_, K, V, H> {
        IndexedDataMapIteratorMut {
            map: self,
            index: 1,
        }
    }
}

impl<'a, K: Debug, V, H: HashPolicy<K>> IndexedDataMapIterator<'a, K, V, H> {
    /// `More`: true while the index is within `1..=Extent()`.
    pub fn more(&self) -> bool {
        self.map.is_some_and(|map| self.index <= map.len())
    }
    /// `Next`: moves to the following index.
    pub fn next(&mut self) {
        self.index = self.index.wrapping_add(1);
    }
    /// `Value`: the item at the current index, or Standard_NoSuchObject past the end.
    pub fn value(&self) -> Result<&'a V, Failure> {
        self.live("NCollection_IndexedDataMap::Iterator::Value")?
            .find_from_index(self.index)
    }
    /// `Key`: the key at the current index, or Standard_NoSuchObject past the end.
    pub fn key(&self) -> Result<&'a K, Failure> {
        self.live("NCollection_IndexedDataMap::Iterator::Key")?
            .find_key(self.index)
    }
    /// `Index`: the current one-based index narrowed to `int` (1 for a default cursor).
    pub fn index(&self) -> i32 {
        self.index as i32
    }
    /// `IsEqual`: both cursors are past the end, or on the same map and index.
    pub fn is_equal(&self, other: &Self) -> bool {
        let same_map = match (self.map, other.map) {
            (Some(a), Some(b)) => std::ptr::eq(a, b),
            (None, None) => true,
            _ => false,
        };
        (!self.more() && !other.more()) || (same_map && self.index == other.index)
    }
    fn live(&self, method: &str) -> Result<&'a IndexedDataMap<K, V, H>, Failure> {
        self.map
            .filter(|_| self.more())
            .ok_or_else(|| exhausted_iterator(method))
    }
}

/// `Iterator()`: a cursor attached to no map, whose `more()` is false.
impl<K, V, H> Default for IndexedDataMapIterator<'_, K, V, H> {
    fn default() -> Self {
        Self {
            map: None,
            index: 1,
        }
    }
}

impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMapIteratorMut<'_, K, V, H> {
    /// `More`: true while the index is within `1..=Extent()`.
    pub fn more(&self) -> bool {
        self.index <= self.map.len()
    }
    /// `Next`: moves to the following index.
    pub fn next(&mut self) {
        self.index = self.index.wrapping_add(1);
    }
    /// `Value`: the item at the current index, or Standard_NoSuchObject past the end.
    pub fn value(&self) -> Result<&V, Failure> {
        self.check("NCollection_IndexedDataMap::Iterator::Value")?;
        self.map.find_from_index(self.index)
    }
    /// `ChangeValue`: the item at the current index for modification.
    pub fn change_value(&mut self) -> Result<&mut V, Failure> {
        self.check("NCollection_IndexedDataMap::Iterator::ChangeValue")?;
        self.map.change_from_index(self.index)
    }
    /// `Key`: the key at the current index, or Standard_NoSuchObject past the end.
    pub fn key(&self) -> Result<&K, Failure> {
        self.check("NCollection_IndexedDataMap::Iterator::Key")?;
        self.map.find_key(self.index)
    }
    /// `Index`: the current one-based index narrowed to `int`.
    pub fn index(&self) -> i32 {
        self.index as i32
    }
    fn check(&self, method: &str) -> Result<(), Failure> {
        if self.more() {
            return Ok(());
        }
        Err(exhausted_iterator(method))
    }
}
