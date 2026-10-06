//! `NCollection_IndexedMap::Iterator`, the remaining lookups and the allocator-replacing `Clear`.
use super::super::base_map::exhausted_iterator;
use super::super::{BaseAllocator, DefaultHashPolicy};
use super::{Debug, HashPolicy, IndexedMap};
use crate::standard::Failure;

/// A read-only OCCT `IndexedMap::Iterator` walking indices `1..=Extent()`.
pub struct IndexedMapIterator<'a, K, H = DefaultHashPolicy> {
    map: Option<&'a IndexedMap<K, H>>,
    index: usize,
}

impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// `TryEmplace`: adds an absent key, keeping an equivalent stored key; returns its index.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedMap;
    /// let mut map = IndexedMap::<i32>::default();
    /// assert_eq!((map.try_emplace(5), map.try_emplace(5)), (1, 1));
    /// ```
    pub fn try_emplace(&mut self, key: K) -> usize {
        self.place(key, false)
    }

    /// `TryEmplaced`: like `try_emplace`, returning the stored key.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedMap;
    /// let mut map = IndexedMap::<i32>::default();
    /// assert_eq!(map.try_emplaced(5), &5);
    /// ```
    pub fn try_emplaced(&mut self, key: K) -> &K {
        let index = self.place(key, false);
        self.storage.key(index)
    }

    /// `Contained`: the stored key equivalent to `key`, if any.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedMap;
    /// let mut map = IndexedMap::<i32>::default();
    /// map.add(5);
    /// assert_eq!(map.contained(&5), Some(&5));
    /// ```
    pub fn contained(&self, key: &K) -> Option<&K> {
        let index = self.find_index(key).checked_sub(1)?;
        Some(self.storage.key(index + 1))
    }

    /// `Clear(const Handle(NCollection_BaseAllocator)&)`: clears, releasing storage only when
    /// the allocator changes; `None` (a null handle) selects the common allocator.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedMap;
    /// let mut map = IndexedMap::<i32>::default();
    /// map.add(5);
    /// map.clear_with_allocator(None);
    /// assert!(map.is_empty());
    /// ```
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        self.storage.clear_with_allocator(resource);
    }

    /// `Iterator(const NCollection_IndexedMap&)`: a cursor on index 1.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::IndexedMap;
    /// let mut map = IndexedMap::<i32>::default();
    /// map.add(5);
    /// let it = map.cursor();
    /// assert_eq!((it.value().unwrap(), it.index()), (&5, 1));
    /// ```
    pub fn cursor(&self) -> IndexedMapIterator<'_, K, H> {
        IndexedMapIterator {
            map: Some(self),
            index: 1,
        }
    }
}

impl<'a, K: Debug, H: HashPolicy<K>> IndexedMapIterator<'a, K, H> {
    /// `More`: true while the index is within `1..=Extent()`.
    pub fn more(&self) -> bool {
        self.map.is_some_and(|map| self.index <= map.len())
    }
    /// `Next`: moves to the following index.
    pub fn next(&mut self) {
        self.index = self.index.wrapping_add(1);
    }
    /// `Value`: the key at the current index, or Standard_NoSuchObject past the end.
    pub fn value(&self) -> Result<&'a K, Failure> {
        let method = "NCollection_IndexedMap::Iterator::Value";
        let map = self
            .map
            .filter(|_| self.more())
            .ok_or_else(|| exhausted_iterator(method))?;
        map.find_key(self.index)
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
}

/// `Iterator()`: a cursor attached to no map, whose `more()` is false.
impl<K, H> Default for IndexedMapIterator<'_, K, H> {
    fn default() -> Self {
        Self {
            map: None,
            index: 1,
        }
    }
}
