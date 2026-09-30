//! `NCollection_Map::Iterator` and the allocator-replacing `Clear`.
use super::super::base_map::exhausted_iterator;
use super::super::{BaseAllocator, DataMapIterator};
use super::{Debug, HashPolicy, Map};
use crate::standard::Failure;

/// A read-only OCCT `Map::Iterator` over bucket order; `let mut it = map.cursor();`.
pub struct MapIterator<'a, K> {
    keys: DataMapIterator<'a, K, ()>,
}

impl<K: Debug, H: HashPolicy<K>> Map<K, H> {
    /// `Iterator(const NCollection_Map&)`: a cursor on the first key in bucket order.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::Map;
    /// let mut map = Map::<i32>::default();
    /// map.add(3);
    /// assert_eq!(map.cursor().key().unwrap(), &3);
    /// ```
    pub fn cursor(&self) -> MapIterator<'_, K> {
        MapIterator {
            keys: self.bindings.cursor(),
        }
    }

    /// `Clear(const Handle(NCollection_BaseAllocator)&)`: clears, releasing buckets only when
    /// the allocator changes; `None` (a null handle) selects the common allocator.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::{BaseAllocator, Map};
    /// let mut map = Map::<i32>::default();
    /// map.add(1);
    /// map.clear_with_allocator(None);
    /// assert!(map.is_empty() && map.allocator() == &BaseAllocator::common());
    /// ```
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        self.bindings.clear_with_allocator(resource);
    }
}

impl<'a, K> MapIterator<'a, K> {
    /// `More`: true while the cursor is on a key.
    pub fn more(&self) -> bool {
        self.keys.more()
    }
    /// `Next`: moves to the following key in bucket order.
    pub fn next(&mut self) {
        self.keys.next();
    }
    /// `Value`: the current key, or Standard_NoSuchObject past the end.
    pub fn value(&self) -> Result<&'a K, Failure> {
        self.current("NCollection_Map::Iterator::Value")
    }
    /// `Key`: the current key, or Standard_NoSuchObject past the end.
    pub fn key(&self) -> Result<&'a K, Failure> {
        self.current("NCollection_Map::Iterator::Key")
    }
    /// `Initialize(const NCollection_Map&)`: restarts on another map.
    pub fn initialize<H: HashPolicy<K>>(&mut self, map: &'a Map<K, H>)
    where
        K: Debug,
    {
        *self = map.cursor();
    }
    /// `Reset`: restarts on the map the cursor was created for.
    pub fn reset(&mut self) {
        self.keys.reset();
    }
    /// `IsEqual`: both cursors are on the same key, or both are past the end.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.keys.is_equal(&other.keys)
    }
    fn current(&self, method: &str) -> Result<&'a K, Failure> {
        self.keys.key().map_err(|_| exhausted_iterator(method))
    }
}

/// `Iterator()`: a cursor attached to no map, whose `more()` is false.
impl<K> Default for MapIterator<'_, K> {
    fn default() -> Self {
        Self {
            keys: DataMapIterator::default(),
        }
    }
}
