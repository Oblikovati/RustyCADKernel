//! Keys in dense index order: the slot vector is the native index table.
use super::{Debug, HashPolicy, IndexedMap, IndexedMapItems, IndexedMapIter};
impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Traverses keys in dense index order; `map.iter().min()`.
    pub fn iter(&self) -> IndexedMapIter<'_, K> {
        IndexedMapIter {
            keys: self.storage.payload().keys(),
            position: 0,
        }
    }
    /// Returns an exhausted key iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> IndexedMapIter<'_, K> {
        IndexedMapIter {
            keys: self.storage.payload().keys(),
            position: self.len(),
        }
    }
    /// Traverses keys paired with one-based indices; `for (key,index) in map.indexed_items() {}`.
    pub fn indexed_items(&self) -> IndexedMapItems<'_, K> {
        IndexedMapItems { inner: self.iter() }
    }
    /// Returns the exhausted indexed range; `map.indexed_items()!=map.indexed_end()`.
    pub fn indexed_end(&self) -> IndexedMapItems<'_, K> {
        IndexedMapItems { inner: self.end() }
    }
}
impl<'a, K> Iterator for IndexedMapIter<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys.get(self.position)?;
        self.position += 1;
        Some(key)
    }
}
impl<K> Clone for IndexedMapIter<'_, K> {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys,
            position: self.position,
        }
    }
}
impl<K> PartialEq for IndexedMapIter<'_, K> {
    fn eq(&self, other: &Self) -> bool {
        (self.position >= self.keys.len() && other.position >= other.keys.len())
            || (self.keys.as_ptr() == other.keys.as_ptr() && self.position == other.position)
    }
}
impl<K> Eq for IndexedMapIter<'_, K> {}
impl<'a, K> Iterator for IndexedMapItems<'a, K> {
    type Item = (&'a K, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.inner.position + 1;
        self.inner.next().map(|key| (key, index))
    }
}
impl<K> Clone for IndexedMapItems<'_, K> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<K> PartialEq for IndexedMapItems<'_, K> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<K> Eq for IndexedMapItems<'_, K> {}
impl<'a, K: Debug, H: HashPolicy<K>> IntoIterator for &'a IndexedMap<K, H> {
    type Item = &'a K;
    type IntoIter = IndexedMapIter<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
