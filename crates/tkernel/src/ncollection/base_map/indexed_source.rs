use super::{Debug, HashPolicy, IndexedDataMap, IndexedMap};
use crate::standard::Failure;

/// Supplies the operations used by OCCT's BaseMap::IndexedIterator.
/// Example: `IndexedCursorSource::indexed_size(&map)`.
pub trait IndexedCursorSource {
    /// Stored key type; e.g. `i32` for `IndexedMap<i32>`.
    type Key;
    /// Indexed value type; e.g. `String` for `IndexedDataMap<i32, String>`.
    type Value;
    /// Returns the live size used by More; `map.indexed_size()`.
    fn indexed_size(&self) -> usize;
    /// Reads a one-based key; `map.indexed_key(1)?`.
    fn indexed_key(&self, index: usize) -> Result<&Self::Key, Failure>;
    /// Reads a one-based value; `map.indexed_value(1)?`.
    fn indexed_value(&self, index: usize) -> Result<&Self::Value, Failure>;
}

/// Supports native mutable indexed values; keys in IndexedMap remain read-only.
/// Example: `*map.indexed_value_mut(1)? = value`.
pub trait IndexedCursorSourceMut: IndexedCursorSource {
    /// Borrows one indexed value exclusively; `map.indexed_value_mut(1)?`.
    fn indexed_value_mut(&mut self, index: usize) -> Result<&mut Self::Value, Failure>;
}

impl<K: Debug, H: HashPolicy<K>> IndexedCursorSource for IndexedMap<K, H> {
    type Key = K;
    type Value = K;
    fn indexed_size(&self) -> usize {
        self.len()
    }
    fn indexed_key(&self, index: usize) -> Result<&K, Failure> {
        self.find_key(index)
    }
    fn indexed_value(&self, index: usize) -> Result<&K, Failure> {
        self.find_key(index)
    }
}

impl<K: Debug, V, H: HashPolicy<K>> IndexedCursorSource for IndexedDataMap<K, V, H> {
    type Key = K;
    type Value = V;
    fn indexed_size(&self) -> usize {
        self.len()
    }
    fn indexed_key(&self, index: usize) -> Result<&K, Failure> {
        self.find_key(index)
    }
    fn indexed_value(&self, index: usize) -> Result<&V, Failure> {
        self.find_from_index(index)
    }
}

impl<K: Debug, V, H: HashPolicy<K>> IndexedCursorSourceMut for IndexedDataMap<K, V, H> {
    fn indexed_value_mut(&mut self, index: usize) -> Result<&mut V, Failure> {
        self.change_from_index(index)
    }
}
